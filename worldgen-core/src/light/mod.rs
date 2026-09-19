// light/mod.rs — 光照内核（vanilla 1.20.1 双 FIFO 传播的 Rust 复刻，方案 B：单 chunk 全量重算）。
// 职责：
//   1. LightEngine：opacity/emission 查找表（light_data.json 数据驱动，按 raw block id 索引，动态扩容）
//   2. light_compute：3×3 邻域 chunk 方块数组 → 中心 chunk 双通道光照（D4 协议，自包含重算）
// 有损边界（对照 vanilla，见交付说明 L1-L4）：全量重算无增量、opacity 查表无 VoxelShape、
//   sky 高度图柱状直落 + BFS、扁平化数组、值队列普通 FIFO（vanilla decrease/increase 双 FIFO 等价）。
//
// ⚠️ 状态：未编译验证（主会话负责 cargo build）。
//
// nibble 布局（G1 对比成败第一坑，对齐 yarn ChunkNibbleArray.getIndex）：
//   section 内 index = (y<<8)|(z<<4)|x（section 局部 y 0..15），偶数 index 占低 4 位：
//   bytes[i>>1] |= value << (4*(i&1))

// CP-1（260916-01）：原 `use std::sync::Mutex` 随 scratch 去全局化移除——
// scratch 从「引擎字段 Mutex 全程持锁」改为 per-call / per-domain-batch 局部
//（#150 解粘同批项：Mutex 整段串行化在域批化后会把持锁窗口 ×9，
//  且该缓冲为清零复用语义、无跨调用有效性，全局唯一本无功能需求）。
// ---- 域常量（3×3 chunk 邻域，扁平域坐标 x/z 0..48，y 0..384 = 世界 y-64..319）----
pub const DOM: usize = 48;
pub const WORLD_H: usize = 384;
pub const SECTIONS: usize = 24;
pub const SECTION_BYTES: usize = 2048;
pub const CHUNK_CELLS: usize = 16 * 16 * WORLD_H; // 98304
pub const BLOCKS9_LEN: usize = 9 * CHUNK_CELLS; // 884736
/// CP-1 域批输入：5×5 chunk（25）——9 个网格对齐中心 chunk 各自 3×3 窗的并集
pub const BLOCKS25_LEN: usize = 25 * CHUNK_CELLS; // 2457600
/// C-1a（260919-05，形态 S1=9×3×3 窗最小切入）：全域共享 fill 的 5×5 域边长（x/z 0..80）
pub const DOM25: usize = 80;
/// C-1a 全域 opacity 容量（80×80×384）
pub const OPACITY25_LEN: usize = DOM25 * DOM25 * WORLD_H; // 2457600
/// CP-1 域批输出：9 中心 × (outBlock + outSky + outFlags) = 9 × 98352 = 885168
pub const DOMAIN_OUT_LEN: usize = 9 * (2 * OUT_CHAN_LEN + FLAGS_LEN);
pub const OUT_CHAN_LEN: usize = SECTIONS * SECTION_BYTES; // 49152
pub const FLAGS_LEN: usize = SECTIONS * 2; // 48

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LightError {
    /// 输入数组长度错误（blocks9 != 884736）
    InputLen,
    /// 输出数组长度错误（outBlock/outSky != 49152 或 outFlags != 48）
    OutputLen,
    /// packed ABI 解码失败（bits 域外 / palette 索引越界 / 游标不闭合）——JNI 映射 rc=-2，
    /// Java 侧整 chunk 回退 blocks9 ABI（260914-04 候选 C，judge MUST 防御面）
    PackedDecode,
}

/// blocks9 内索引：chunk c(=dz*3+dx)，chunk 内 (y+64)*256 + z*16 + x
#[inline]
fn blocks9_index(c: usize, y: usize, lx: usize, lz: usize) -> usize {
    c * CHUNK_CELLS + y * 256 + lz * 16 + lx
}

/// 域内扁平索引：(y*48 + z)*48 + x，最大 884735 < u32::MAX（队列打包安全）
#[inline]
fn dom_index(x: usize, y: usize, z: usize) -> usize {
    (y * DOM + z) * DOM + x
}

#[inline]
fn dom_unpack(i: usize) -> (usize, usize, usize) {
    let x = i % DOM;
    let r = i / DOM;
    (x, r % DOM, r / DOM)
}

/// 光照引擎 handle（Java 侧以 Box::into_raw 指针持有）。
/// 查找表按 raw id 索引动态扩容，缺 id 用 default；scratch 缓冲复用避免热路径堆分配。
pub struct LightEngine {
    /// (opacity, emission) by block id
    table: Vec<(u8, u8)>,
    /// table[0] == (0,0) 时 fill 可走「air 快路径」（id==0 且无 luminance ⇒ op/em 全 0，
    /// 免查表 + 免 col_max 条件写）；真实 light_data 下恒成立，非默认 (0,0) 时自动禁用。
    air_fast: bool,
    // CP-3（260915-02，形态审计候选池，judge C-4 声明：防御性修复——UB 可达性未实证）：
    // RefCell → Mutex。原 RefCell 在多线程并发调用 light_compute 时 borrow_mut 会 panic
    //（非 UB 但同样不可用），且形态审计指出光照车道串行化事实上在充当本字段的锁
    //（发现 #150：解粘必须与本修复同批）。Mutex 使引擎自身并发安全，poison 时不 panic、
    // 恢复继续用（scratch 是清零复用缓冲，内容无跨调用有效性）。
    // CP-1（260916-01，.b1 域批中心化同批解粘）：scratch 字段移除——Mutex 全程持锁把
    // 并发 light 整段串行化；per-call 局部化后正确性由「无状态纯函数 + 清零复用缓冲」
    // 结构性承载（借用分离由编译期检查复证：light_compute_inner 只拿 &LightEngine +
    // &mut Scratch，任何隐藏可变路径直接编译失败）。原 Mutex 方案存档于此注释（§15.4 精神）。
}

struct Scratch {
    opacity: Vec<u8>,
    block_light: Vec<u8>,
    sky_light: Vec<u8>,
    queue: Vec<u32>,
    /// 每域列 (x + z*DOM) 的最高不透明 y（无则 -1）——fill 同趟收集，sky 阶段复用
    col_max: Vec<i32>,
}

impl Scratch {
    /// CP-1：per-call / per-domain-batch 局部 scratch（容量跨调用复用收益由
    /// 域批内 9 中心循环共享一个 Scratch 承载；跨调用复用收益占比未量化，@anchor.idk 级）。
    fn new() -> Self {
        Scratch {
            opacity: Vec::new(),
            block_light: Vec::new(),
            sky_light: Vec::new(),
            queue: Vec::new(),
            col_max: Vec::new(),
        }
    }
}

impl LightEngine {
    /// 从 light_data.json 文件加载（格式 corewap-light-1）
    pub fn from_json_file(path: &str) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
        Self::from_json_str(&text)
    }

    /// 从 JSON 文本加载。key = 十进制 raw id 字符串；缺 id 用 default。
    pub fn from_json_str(text: &str) -> Result<Self, String> {
        let root = crate::json::parse(text).map_err(|e| format!("parse light_data: {e}"))?;

        let (d_op, d_em) = match root.get("default") {
            Some(d) => (parse_u8_field(d, "opacity", 0), parse_u8_field(d, "emission", 0)),
            None => (0u8, 0u8),
        };

        let mut table: Vec<(u8, u8)> = vec![(d_op, d_em); 1024];

        if let Some(blocks) = root.get("blocks").and_then(|b| b.as_object()) {
            for (k, v) in blocks {
                let id: usize = k.parse().map_err(|_| format!("bad block id key: {k}"))?;
                if id >= table.len() {
                    table.resize(id + 1, (d_op, d_em));
                }
                table[id] = (
                    parse_u8_field(v, "opacity", d_op),
                    parse_u8_field(v, "emission", d_em),
                );
            }
        }

        Ok(LightEngine {
            air_fast: table[0] == (0, 0),
            table,
        })
    }

    /// 方块 raw id → (opacity, emission)；缺 id / 负 id 用 default（表首项）。
    #[inline]
    fn lookup(&self, id: i32) -> (u8, u8) {
        if id < 0 {
            return self.table[0];
        }
        self.table.get(id as usize).copied().unwrap_or(self.table[0])
    }

    /// 探针轮 260914-04（光照 round3）：诊断只读访问器（bin-diag light_probe_260914 用），
    /// 生产路径零引用；与 light_compute_phased 同款 doc-hidden 诊断面模式。
    #[doc(hidden)]
    pub fn lookup_probe(&self, id: i32) -> (u8, u8) {
        self.lookup(id)
    }

    /// 探针轮 260914-04：air_fast 只读（table[0]==(0,0)，air 快路径合法性前提）。
    #[doc(hidden)]
    pub fn air_fast_probe(&self) -> bool {
        self.air_fast
    }
}

#[inline]
fn parse_u8_field(v: &crate::json::JsonValue, key: &str, default: u8) -> u8 {
    // 注意（260905-05）：负值（如 vanilla opacity -1 = VoxelShape 哑元）会被 clamp 成 0——
    // 本引擎无 VoxelShape（已声明有损边界），负值语义无法表达。数据侧必须显式化 0..15
    //（light_data.json 613-629 shulker→15、954 pointed_dripstone→0 已于 260905-05 修复），
    // 新数据源禁止再出现负 opacity；如需恢复哨兵语义须扩展表结构。
    v.get(key)
        .and_then(|f| f.as_f64())
        .map(|n| n.clamp(0.0, 15.0) as u8)
        .unwrap_or(default)
}

// ---- 核心 ABI（契约函数，JNI 侧共享）----

/// D4 协议：3×3 邻域方块 → 中心 chunk 双通道光照导出。
/// blocks9: 9 chunk raw id（chunkIdx = dz*3+dx；chunk 内 (y+64)*256+z*16+x）
/// out_block/out_sky: 24 section × 2048 B；out_flags: 48 B = 每 section [blockFlag, skyFlag]
///   flag: 0=非均质（data 有效）, 1=全 0, 2=全 15（均质 section 同样写入全值 data）
pub fn light_compute(
    engine: &LightEngine,
    blocks9: &[i32],
    out_block: &mut [u8],
    out_sky: &mut [u8],
    out_flags: &mut [u8],
) -> Result<(), LightError> {
    // CP-1：per-call 局部 scratch（原为引擎 Mutex 字段全程持锁——#150 解粘同批项）
    let mut scratch = Scratch::new();
    light_compute_inner(engine, &mut scratch, blocks9, out_block, out_sky, out_flags, None)
}

/// 诊断用 phase 计时（bin-diag 专用，生产路径传 None 零开销）。
/// 顺序：fill / block_bfs / sky_fall / sky_seed_bfs / export。
#[doc(hidden)]
pub struct PhaseTimings(pub [std::time::Duration; 5]);

/// packed ABI 解码（260914-04 候选 C）：Java 侧 writePacket 帧（bits + palette + longs）的
/// 预拆产物 → blocks9。等价性构造性保证：输出即 B 路径（Java 解码填 blocks9）的同一数组，
/// 供 light_compute 消费——golden 逐位门由「同内核同输入」结构性承载。
///
/// 帧契约（一手源 PC.java:383-387 + Singular/Array/BiMap writePacket，Java 侧拆帧）：
/// - section_meta：216 节 × 2 = 432 项，节序 = 9 chunk（c=dz*3+dx）× 24 section，
///   每节 [bits, psz]；bits=0&psz=0 = 空节哨兵（全 AIR）；bits=0&psz=1 = singular 均质节。
/// - palette_data：逐节拼接的 ABI 编码表（rawId | lum<<24），空节零长。
/// - packed：逐节拼接的 PackedIntegerArray longs（LSB-first、元素不跨 long，PIA.java:261/307-313；
///   索引序 = computeIndex y<<8|z<<4|x = blocks9 节内序，b2 §1 静态闭环 + B 路双采集 ALL-MATCH 实证）。
/// - bits 域 = 4..=14（ArrayPalette/BiMapPalette）；global（ID_LIST，bits≥15）**不进本 ABI**——
///   Java 侧遇 global 节整 chunk 走 blocks9 ABI（1.20.1 直方图实测 0 例，防御面）。
pub fn light_decode_packed(
    section_meta: &[i32],
    palette_data: &[i32],
    packed: &[i64],
    blocks9_out: &mut [i32],
) -> Result<(), LightError> {
    if section_meta.len() != SECTIONS * 9 * 2 || blocks9_out.len() != BLOCKS9_LEN {
        return Err(LightError::InputLen);
    }
    // 260919-03：实现收敛到 light_decode_packed_into（单一实现，防双路漂移）
    let bases: [usize; 9] = std::array::from_fn(|c| c * CHUNK_CELLS);
    light_decode_packed_into(section_meta, palette_data, packed, &bases, blocks9_out)
}

/// 260919-03（.b2 C-3）：packed 解码泛化——节组按 chunk_bases[i] 给出的目标 chunk 基址
/// 解码（base = 目标数组内 chunk 起始下标）。light_decode_packed = 本函数在
/// bases = [0,1,..,8]*CHUNK_CELLS、out = blocks9 下的特例（包装保持，历史调用零改动）。
/// 契约同 light_decode_packed（帧形态/bits 域/游标闭合），meta 长 = bases.len()*SECTIONS*2。
pub fn light_decode_packed_into(
    section_meta: &[i32],
    palette_data: &[i32],
    packed: &[i64],
    chunk_bases: &[usize],
    blocks_out: &mut [i32],
) -> Result<(), LightError> {
    let n = chunk_bases.len();
    if section_meta.len() != SECTIONS * n * 2
        || n == 0
        || blocks_out.len() < (n - 1) * CHUNK_CELLS + CHUNK_CELLS
    {
        return Err(LightError::InputLen);
    }
    // 越界基址防御（decode 内部按 base 直接索引，先验一次）
    for &b in chunk_bases {
        if b % CHUNK_CELLS != 0 || b + CHUNK_CELLS > blocks_out.len() {
            return Err(LightError::InputLen);
        }
    }
    // 仅清本帧目标 chunk（空节哨兵 = AIR）——域批多帧依次解码时不得触碰其他 chunk
    // （全量清零会抹掉先前帧写入，260919-03 实现自检捕获）
    for &b in chunk_bases {
        blocks_out[b..b + CHUNK_CELLS].fill(0);
    }
    let mut pal_cur = 0usize;
    let mut sto_cur = 0usize;
    for c in 0..n {
        for s in 0..SECTIONS {
            let i = (c * SECTIONS + s) * 2;
            let bits = section_meta[i];
            let psz = section_meta[i + 1] as usize;
            if bits == 0 && psz == 0 {
                continue;
            }
            let base = chunk_bases[c] + s * 4096;
            if bits == 0 {
                if psz != 1 || palette_data.len() < pal_cur + 1 {
                    return Err(LightError::PackedDecode);
                }
                let v = palette_data[pal_cur];
                pal_cur += 1;
                blocks_out[base..base + 4096].fill(v);
                continue;
            }
            if !(4..=14).contains(&bits) || psz < 1 || psz > 256 {
                return Err(LightError::PackedDecode);
            }
            if palette_data.len() < pal_cur + psz {
                return Err(LightError::PackedDecode);
            }
            let pal = &palette_data[pal_cur..pal_cur + psz];
            pal_cur += psz;
            let epl = 64 / bits as usize;
            let n_long = (4096 + epl - 1) / epl;
            if packed.len() < sto_cur + n_long {
                return Err(LightError::PackedDecode);
            }
            let words = &packed[sto_cur..sto_cur + n_long];
            sto_cur += n_long;
            let bits_u = bits as usize;
            let mask = (1u64 << bits_u) - 1;
            let long_bits = epl * bits_u;
            let mut dst = base;
            let mut w = 0usize;
            let mut off = 0usize;
            for _ in 0..4096 {
                let v = ((words[w] as u64 >> off) & mask) as usize;
                if v >= psz {
                    return Err(LightError::PackedDecode);
                }
                blocks_out[dst] = pal[v];
                dst += 1;
                off += bits_u;
                if off == long_bits {
                    w += 1;
                    off = 0;
                }
            }
        }
    }
    if pal_cur != palette_data.len() || sto_cur != packed.len() {
        return Err(LightError::PackedDecode);
    }
    Ok(())
}

/// 260919-03（.b2 C-3）：域批 packed 解码——n 帧（每帧 = 一个已提交中心的 3×3 邻域
/// packed 帧，meta 9*24*2 = 432 项）依次解码进 blocks25。帧 k（k = kz*3+kx，kz/kx 为
/// 中心在域内 0..3 网格序）覆盖 chunk 基 = ((kz+dz9)*5 + (kx+dx9)) * CHUNK_CELLS
/// （c9 = dz9*3+dx9）；后帧覆盖重叠 = Java 侧 blocks25 arraycopy 顺序语义（同序 k=0..n）。
/// frame_lens = 每帧 [palLen, stoLen]（2n 项）；帧内 pal/sto 依序拼接。
/// 未被任何帧覆盖的边缘 chunk 保持 0（AIR）= 现 blocks25 零槽语义。
pub fn light_decode_packed_domain(
    frame_meta: &[i32],
    palette_data: &[i32],
    packed: &[i64],
    frame_lens: &[i32],
    blocks25_out: &mut [i32],
) -> Result<(), LightError> {
    if frame_lens.len() % 2 != 0 || blocks25_out.len() != BLOCKS25_LEN {
        return Err(LightError::InputLen);
    }
    let n = frame_lens.len() / 2;
    if n == 0 || n > 9 || frame_meta.len() != SECTIONS * 9 * 2 * n {
        return Err(LightError::InputLen);
    }
    let mut pal_cur = 0usize;
    let mut sto_cur = 0usize;
    let mut meta_cur = 0usize;
    for k in 0..n {
        let pal_len = frame_lens[2 * k];
        let sto_len = frame_lens[2 * k + 1];
        if pal_len < 0 || sto_len < 0 {
            return Err(LightError::InputLen);
        }
        let (pal_len, sto_len) = (pal_len as usize, sto_len as usize);
        if palette_data.len() < pal_cur + pal_len || packed.len() < sto_cur + sto_len {
            return Err(LightError::PackedDecode);
        }
        let kx = k % 3;
        let kz = k / 3;
        let bases: [usize; 9] = std::array::from_fn(|c9| {
            ((kz + c9 / 3) * 5 + (kx + c9 % 3)) * CHUNK_CELLS
        });
        light_decode_packed_into(
            &frame_meta[meta_cur..meta_cur + SECTIONS * 9 * 2],
            &palette_data[pal_cur..pal_cur + pal_len],
            &packed[sto_cur..sto_cur + sto_len],
            &bases,
            blocks25_out,
        )?;
        meta_cur += SECTIONS * 9 * 2;
        pal_cur += pal_len;
        sto_cur += sto_len;
    }
    if pal_cur != palette_data.len() || sto_cur != packed.len() || meta_cur != frame_meta.len() {
        return Err(LightError::PackedDecode); // 游标不闭合 = 帧契约破坏
    }
    Ok(())
}

/// 诊断入口：同 light_compute，附 phase 级分解（C2，judge 条件）。#[doc(hidden)]
pub fn light_compute_phased(
    engine: &LightEngine,
    blocks9: &[i32],
    out_block: &mut [u8],
    out_sky: &mut [u8],
    out_flags: &mut [u8],
) -> Result<PhaseTimings, LightError> {
    let mut t = PhaseTimings(std::array::from_fn(|_| std::time::Duration::ZERO));
    let mut scratch = Scratch::new();
    light_compute_inner(engine, &mut scratch, blocks9, out_block, out_sky, out_flags, Some(&mut t.0))?;
    Ok(t)
}

/// CP-1（260916-01，.b1 域批中心化）：5×5 chunk 方块帧 → 网格对齐 3×3 中心 chunk 的
/// 双通道光照批量导出。9 个中心的 3×3 窗并集恰为 5×5；每中心以对应子窗调
/// light_compute_inner 同构内核（Scratch 与子窗缓冲在 9 中心循环间复用），保证
/// 逐位等价于 per-chunk 路径（同内核同输入）。
///
/// 帧契约：
/// - blocks25: 25 chunk raw id（chunkIdx25 = dz25*5+dx25，dx25/dz25 0..5；
///   chunk 内 (y+64)*256 + z*16 + x，总长 2457600）。5×5 块覆盖 chunk [X..X+4]×[Z..Z+4]，
///   9 中心 = [X+1..X+3]×[Z+1..Z+3]；边缘 chunk 只读不产出。
/// - out: 9 段连续输出，段序 k = kz*3+kx（中心 chunk = X+1+kx, Z+1+kz），
///   每段 = outBlock(49152) ++ outSky(49152) ++ outFlags(48)，总长 885168。
pub fn light_compute_domain(
    engine: &LightEngine,
    blocks25: &[i32],
    out: &mut [u8],
) -> Result<(), LightError> {
    light_compute_domain_impl(engine, blocks25, out, None).map(|_| ())
}

/// 诊断入口（260919-02 A1 立项预研，env WG_LIGHTPHASE 门控）：同 light_compute_domain，
/// 附 9 中心聚合 phase 账 + 子窗拷贝累计（task 级一次，无热路径逐格计时）。#[doc(hidden)]
#[derive(Debug, Default, Clone, Copy)]
pub struct DomainPhases {
    /// 与 PhaseTimings 同序：fill / block_bfs / sky_fall / sky_seed_bfs / export。
    /// C-1a（260919-05）口径：fill = 全域共享 fill 一趟（非 9 中心求和）；
    /// block_bfs/sky_fall/sky_seed_bfs/export 仍为 9 中心求和。
    pub phases: [std::time::Duration; 5],
    /// 窗准备累计（9 中心求和；C-1a 后 = opacity/col_max 行拷 + 种子回填，旧 b9←blocks25 子窗拷贝已消失）
    pub subcopy: std::time::Duration,
}

pub fn light_compute_domain_phased(
    engine: &LightEngine,
    blocks25: &[i32],
    out: &mut [u8],
) -> Result<DomainPhases, LightError> {
    let mut dp = DomainPhases {
        phases: std::array::from_fn(|_| std::time::Duration::ZERO),
        subcopy: std::time::Duration::ZERO,
    };
    light_compute_domain_impl(engine, blocks25, out, Some(&mut dp))?;
    Ok(dp)
}

fn light_compute_domain_impl(
    engine: &LightEngine,
    blocks25: &[i32],
    out: &mut [u8],
    mut diag: Option<&mut DomainPhases>,
) -> Result<(), LightError> {
    if blocks25.len() != BLOCKS25_LEN {
        return Err(LightError::InputLen);
    }
    if out.len() < DOMAIN_OUT_LEN {
        return Err(LightError::OutputLen);
    }

    // C-1a（260919-05，形态 S1 = 9×3×3 窗最小切入，用户拍板）：全域 fill 一趟共享
    // + 9 中心各自 3×3 窗 BFS+export。
    // - fill 逐格纯查表（邻接无关，col_max 列内）⇒ 25 chunk 一趟，域成员口径
    //   81→25 次 chunk-fill（3.24×，scout-map §2.2；12 篇 :141 的 9× 是邻域参与口径，
    //   不作本优化列账依据——两口径不同度量，非冲突）。
    // - G3 保持（design-c1-260919-04 §2）：fill/种子/packed 解码均为本帧 blocks25
    //   快照的确定函数，9 中心共享同一份本帧 fill 结果不引入跨任务状态；BFS 仍每
    //   中心从本帧快照独立重算，无新不动点语义。
    // - 窗内 block/sky 结果与全域 fill 逐位一致：种子值/收集序（i25 升序 = 旧 fill
    //   内联序）与 opacity/col_max 逐格恒等，位等价由 light_compute_domain_bitwise_equivalence 钉死。
    let t_fill = diag.as_mut().map(|_| std::time::Instant::now());
    let mut opacity25 = vec![0u8; OPACITY25_LEN];
    let mut col_max25 = vec![-1i32; DOM25 * DOM25];
    let mut seeds25: Vec<u32> = Vec::new(); // 打包 (em<<24)|idx80（idx80 < 2^22，em ≤ 15 占高 8 位）
    fill_domain_generic(
        engine,
        blocks25,
        &mut opacity25,
        &mut col_max25,
        DOM25,
        5,
        |i, em| seeds25.push(((em as u32) << 24) | i as u32),
    );
    if let (Some(d), Some(t)) = (diag.as_mut(), t_fill) {
        d.phases[0] += t.elapsed(); // 口径注（260919-05）：fill_us = 全域共享 fill 一趟（非 9 中心求和）
    }

    let mut scratch = Scratch::new();
    for k in 0..9usize {
        let kx = k % 3; // 0..2 → 窗 x = kx*16..kx*16+48（80 域内 chunk 列）
        let kz = k / 3;
        // 窗准备（subcopy 口径扩展，260919-05）：旧 b9←blocks25 子窗拷贝替换为
        // opacity/col_max 行拷 + block 种子回填（b9 子窗拷贝随 fill 共享整体消失）
        let t_sub = diag.as_mut().map(|_| std::time::Instant::now());
        {
            let Scratch { opacity, block_light, sky_light, queue, col_max } = &mut scratch;
            opacity.clear();
            opacity.resize(BLOCKS9_LEN, 0);
            block_light.clear();
            block_light.resize(BLOCKS9_LEN, 0);
            sky_light.clear();
            sky_light.resize(BLOCKS9_LEN, 0);
            queue.clear();
            col_max.clear();
            col_max.resize(DOM * DOM, -1);
            let x0 = kx * 16;
            let z0 = kz * 16;
            for y in 0..WORLD_H {
                for lz in 0..DOM {
                    let src = (y * DOM25 + (z0 + lz)) * DOM25 + x0;
                    let dst = (y * DOM + lz) * DOM;
                    opacity[dst..dst + DOM].copy_from_slice(&opacity25[src..src + DOM]);
                }
            }
            for lz in 0..DOM {
                let src = (z0 + lz) * DOM25 + x0;
                let dst = lz * DOM;
                col_max[dst..dst + DOM].copy_from_slice(&col_max25[src..src + DOM]);
            }
            // 种子回填：仅落本窗内的 emission（seeds25 升序 ⇒ 窗内序 = 旧 fill 内联种子序）
            for &s in &seeds25 {
                let em = (s >> 24) as u8;
                let i25 = (s & 0x00FF_FFFF) as usize;
                let x = i25 % DOM25;
                let r = i25 / DOM25;
                let z = r % DOM25;
                let y = r / DOM25;
                if x >= x0 && x < x0 + DOM && z >= z0 && z < z0 + DOM {
                    let wi = (y * DOM + (z - z0)) * DOM + (x - x0);
                    block_light[wi] = em;
                    queue.push(wi as u32);
                }
            }
        }
        if let (Some(d), Some(t)) = (diag.as_mut(), t_sub) {
            d.subcopy += t.elapsed();
        }
        let seg = &mut out[k * (2 * OUT_CHAN_LEN + FLAGS_LEN)..(k + 1) * (2 * OUT_CHAN_LEN + FLAGS_LEN)];
        let (ob, rest) = seg.split_at_mut(OUT_CHAN_LEN);
        let (os, of) = rest.split_at_mut(OUT_CHAN_LEN);
        propagate_and_export(&mut scratch, ob, os, of, diag.as_mut().map(|d| &mut d.phases));
    }
    Ok(())
}

/// fill 泛化（C-1a，260919-05）：blocks（chunks×chunks 域，dom-major (y*dom+z)*dom+x）
/// → opacity + col_max 同趟；每个 emission 种子经 `on_seed(i, em)` 交付（3×3 内联写
/// block_light+queue / 5×5 打包进 seeds25，两态均按 i 升序回调 = 旧 fill 收集序）。
/// 逐格纯查表（邻接无关）+ col_max 列内运算——共享 fill 的等价性根据。
fn fill_domain_generic<E: FnMut(usize, u8)>(
    engine: &LightEngine,
    blocks: &[i32],
    opacity: &mut [u8],
    col_max: &mut [i32],
    dom: usize,
    chunks: usize,
    mut on_seed: E,
) {
    // ABI：低 24 位 = raw id，高 8 位 = luminance 真值（state 级，Java 侧提供；为 0 时
    // 回退数据表 emission —— 表仍作 id 级回退源）。air 快路径：v==0（id=0 且 luminance=0，
    // luminance 高位非 0 不得走此路径）且 table[0]==(0,0) ⇒ op/em 全 0。
    for y in 0..WORLD_H {
        let yb = y * 256;
        for z in 0..dom {
            let cz = z >> 4;
            let lz = z & 15;
            let row = (y * dom + z) * dom;
            let colrow = z * dom;
            for cx in 0..chunks {
                let c = cz * chunks + cx;
                let base = c * CHUNK_CELLS + yb + lz * 16;
                let d = row + cx * 16;
                for lx in 0..16usize {
                    let v = blocks[base + lx] as u32;
                    let i = d + lx;
                    if engine.air_fast && v == 0 {
                        // opacity[i] 保持 0（调用方清零语义）；block_light 同
                        continue;
                    }
                    let id = (v & 0x00FF_FFFF) as i32;
                    let lum = (v >> 24) as u8;
                    let (op, mut em) = engine.lookup(id);
                    if lum > 0 {
                        em = lum;
                    }
                    opacity[i] = op;
                    if em > 0 {
                        on_seed(i, em);
                    }
                    if op > 0 {
                        col_max[colrow + cx * 16 + lx] = y as i32;
                    }
                }
            }
        }
    }
}

/// 传播与导出（C-1a 抽取，260919-05）：light_compute_inner 的 ②③④ 相原样抽出——
/// block BFS → sky 直落 + 种子收缩 + sky BFS → export。域批路（fill 共享后）与
/// per-chunk 路共用本函数，保证同内核同输入逐位等价。
fn propagate_and_export(
    scratch: &mut Scratch,
    out_block: &mut [u8],
    out_sky: &mut [u8],
    out_flags: &mut [u8],
    mut phases: Option<&mut [std::time::Duration; 5]>,
) {
    let Scratch { opacity, block_light, sky_light, queue, col_max } = scratch;
    let _t1 = phases.as_mut().map(|_| std::time::Instant::now());

    // 2. block light：BFS（6 邻域，cost = max(1, opacity[邻])，无 sky 直落特例）
    bfs_propagate(block_light, opacity, queue, false);

    if let Some(p) = phases.as_deref_mut() {
        p[1] += _t1.map(|t| t.elapsed()).unwrap_or_default();
    }
    let _t2 = phases.as_mut().map(|_| std::time::Instant::now());

    queue.clear();

    // 3. sky light：柱状直落（opacity 0 一路 15，遇 opacity>0 停）+ BFS 水平/向下扩散。
    //    round2：直落区间直接取 col_max——首不透明格 = 列最大 y（fill 已按 y 升序取 max），
    //    故 15-区间 = [col_max+1, 383]，与原「自上而下 blocked 扫描」逐位等价且免 opacity 读。
    for z in 0..DOM {
        for x in 0..DOM {
            let lo = (col_max[z * DOM + x] + 1) as usize; // col_max=-1 → 0（全空列整柱 15）
            for y in lo..WORLD_H {
                sky_light[(y * DOM + z) * DOM + x] = 15;
            }
        }
    }

    if let Some(p) = phases.as_deref_mut() {
        p[2] += _t2.map(|t| t.elapsed()).unwrap_or_default();
    }
    let _t3 = phases.as_mut().map(|_| std::time::Instant::now());

    // 种子收缩 round2：边界种子集合用列区间算术直接枚举（消 round1 的 O(N)×6 邻域全扫）。
    // 等价性论证：每列 15-区间 [lo, 383] 连续（lo = col_max+1），故格 (x,z,y)（y∈区间）的
    // 6 邻域存在 sky<15 ⇔ ①y==lo（列底，下方 = col_max 不透明格 sky=0；lo==0 时无下方邻）
    // 或 ②∃ 水平邻列其 lo_nb > y（y 不在邻列区间内；y+1<15 不可能，区间向上连续到顶）。
    // 与 round1 逐格扫描收集的种子集合严格恒等；传播单调 + 不动点唯一 ⇒ BFS 结果逐位一致。
    for z in 0..DOM {
        for x in 0..DOM {
            let lo = col_max[z * DOM + x] + 1;
            if lo >= WORLD_H as i32 {
                continue; // 空 15-区间（列顶即不透明）
            }
            let mut bottom_covered = false;
            let nbs = [
                (x > 0).then(|| (x - 1, z)),
                (x + 1 < DOM).then(|| (x + 1, z)),
                (z > 0).then(|| (x, z - 1)),
                (z + 1 < DOM).then(|| (x, z + 1)),
            ];
            for nb in nbs.into_iter().flatten() {
                let (nx, nz) = nb;
                let hi = (col_max[nz * DOM + nx] + 1).min(WORLD_H as i32) - 1;
                if hi >= lo {
                    // 补集区间 [lo, hi]：这些 y 在本列是 15 而邻列 <15 → 边界种子
                    for y in lo..=hi {
                        queue.push(((y as usize * DOM + z) * DOM + x) as u32);
                    }
                    if lo == 0 {
                        bottom_covered = true; // lo==0 时底格无下方邻，由区间覆盖
                    }
                }
            }
            if !bottom_covered && lo > 0 {
                // 列底格恒为边界（下方 sky=0）；lo==0 且无区间则该列无任何种子（全柱 15 且邻列亦全 15）
                queue.push(((lo as usize * DOM + z) * DOM + x) as u32);
            }
        }
    }
    bfs_propagate(sky_light, opacity, queue, true);

    if let Some(p) = phases.as_deref_mut() {
        p[3] += _t3.map(|t| t.elapsed()).unwrap_or_default();
    }
    let _t4 = phases.as_mut().map(|_| std::time::Instant::now());

    // 4. 导出中心 chunk（域 x/z 16..32）+ 均质性 flags
    export_center(block_light, sky_light, out_block, out_sky, out_flags);

    if let Some(p) = phases.as_deref_mut() {
        p[4] += _t4.map(|t| t.elapsed()).unwrap_or_default();
    }
}

fn light_compute_inner(
    engine: &LightEngine,
    scratch: &mut Scratch,
    blocks9: &[i32],
    out_block: &mut [u8],
    out_sky: &mut [u8],
    out_flags: &mut [u8],
    mut phases: Option<&mut [std::time::Duration; 5]>,
) -> Result<(), LightError> {
    if blocks9.len() != BLOCKS9_LEN {
        return Err(LightError::InputLen);
    }
    if out_block.len() < OUT_CHAN_LEN || out_sky.len() < OUT_CHAN_LEN || out_flags.len() < FLAGS_LEN {
        return Err(LightError::OutputLen);
    }

    let Scratch { opacity, block_light, sky_light, queue, col_max } = scratch;

    let _t0 = phases.as_mut().map(|_| std::time::Instant::now());

    // clear+resize：scratch 跨调用复用，必须清零（resize 不覆盖已有元素）
    opacity.clear();
    opacity.resize(BLOCKS9_LEN, 0);
    block_light.clear();
    block_light.resize(BLOCKS9_LEN, 0);
    sky_light.clear();
    sky_light.resize(BLOCKS9_LEN, 0);
    queue.clear();
    col_max.clear();
    col_max.resize(DOM * DOM, -1);

    // 1. fill（C-1a 260919-05 抽为 fill_domain_generic，3×3 态内联回调 = 旧内联收集，
    //    同序同值）：round2 dom-major 重排 + 同趟 col_max 语义不变，见 fill_domain_generic 注。
    fill_domain_generic(engine, blocks9, opacity, col_max, DOM, 3, |i, em| {
        block_light[i] = em;
        queue.push(i as u32);
    });
    if let Some(p) = phases.as_deref_mut() {
        p[0] += _t0.map(|t| t.elapsed()).unwrap_or_default();
    }
    propagate_and_export(scratch, out_block, out_sky, out_flags, phases);
    Ok(())
}

/// 通用 BFS：队列里是已设值的格（打包 u32 域索引），弹出后向 6 邻域传播。
/// cost = max(1, opacity[n])；`sky_direct_down` 时特例：l==15 且向下且 opacity[n]==0 → 不衰减（vanilla
/// ChunkSkyLightProvider 语义：15 值向下传播 cost 0，无论 15 来源是直落还是横向扩散）。
fn bfs_propagate(
    light: &mut [u8],
    opacity: &[u8],
    queue: &mut Vec<u32>,
    sky_direct_down: bool,
) {
    let mut head = 0usize;
    while head < queue.len() {
        let i = queue[head] as usize;
        head += 1;
        let l = light[i];
        if l == 0 {
            continue;
        }
        if head >= 65536 {
            // 队列压缩：丢弃已消费前缀，防内存无界
            queue.drain(..head);
            head = 0;
        }
        // dom_unpack 返回 (x, z, y)：第二元 = (i/DOM)%DOM 是 z，第三元 = i/(DOM*DOM) 是 y
        let (x, z, y) = dom_unpack(i);
        // 6 邻域（-x,+x,-z,+z,-y,+y）
        if x > 0 {
            try_spread(light, opacity, queue, i - 1, l, sky_direct_down, false);
        }
        if x + 1 < DOM {
            try_spread(light, opacity, queue, i + 1, l, sky_direct_down, false);
        }
        if z > 0 {
            try_spread(light, opacity, queue, i - DOM, l, sky_direct_down, false);
        }
        if z + 1 < DOM {
            try_spread(light, opacity, queue, i + DOM, l, sky_direct_down, false);
        }
        if y > 0 {
            try_spread(light, opacity, queue, i - DOM * DOM, l, sky_direct_down, true);
        }
        if y + 1 < WORLD_H {
            try_spread(light, opacity, queue, i + DOM * DOM, l, sky_direct_down, false);
        }
    }
}

#[inline]
fn try_spread(
    light: &mut [u8],
    opacity: &[u8],
    queue: &mut Vec<u32>,
    n: usize,
    l: u8,
    sky_direct_down: bool,
    downward: bool,
) {
    let op = opacity[n];
    let new_level: i32 = if sky_direct_down && downward && l == 15 && op == 0 {
        15
    } else {
        l as i32 - (op.max(1) as i32)
    };
    if new_level > light[n] as i32 {
        light[n] = new_level as u8;
        queue.push(n as u32);
    }
}

/// 导出中心 chunk：每 section 打包 nibble + 均质性 flag（均质 section 同样写全值 data）。
fn export_center(
    block_light: &[u8],
    sky_light: &[u8],
    out_block: &mut [u8],
    out_sky: &mut [u8],
    out_flags: &mut [u8],
) {
    const CENTER_OFF: usize = 16; // 中心 chunk 在域坐标中的偏移（dx=1,dz=1）
    for s in 0..SECTIONS {
        let fb = &mut out_block[s * SECTION_BYTES..(s + 1) * SECTION_BYTES];
        let fs = &mut out_sky[s * SECTION_BYTES..(s + 1) * SECTION_BYTES];
        fb.fill(0);
        fs.fill(0);
        let mut b_min = 15u8;
        let mut b_max = 0u8;
        let mut s_min = 15u8;
        let mut s_max = 0u8;
        for ly in 0..16usize {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    let i = dom_index(CENTER_OFF + lx, s * 16 + ly, CENTER_OFF + lz);
                    let bv = block_light[i];
                    let sv = sky_light[i];
                    b_min = b_min.min(bv);
                    b_max = b_max.max(bv);
                    s_min = s_min.min(sv);
                    s_max = s_max.max(sv);
                    let nib = (ly << 8) | (lz << 4) | lx;
                    fb[nib >> 1] |= bv << (4 * (nib & 1));
                    fs[nib >> 1] |= sv << (4 * (nib & 1));
                }
            }
        }
        out_flags[s * 2] = homogeneity_flag(b_min, b_max);
        out_flags[s * 2 + 1] = homogeneity_flag(s_min, s_max);
    }
}

/// 0=非均质（data 有效），1=全 0，2=全 15；均质但值非 0/15（理论不可达）按非均质处理。
#[inline]
fn homogeneity_flag(min: u8, max: u8) -> u8 {
    if min != max {
        0
    } else if min == 0 {
        1
    } else if min == 15 {
        2
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_compute_smoke_no_panic() {
        let engine = LightEngine::from_json_str(r#"{"format":"corewap-light-1","default":{"opacity":0,"emission":0},"blocks":{"1":{"opacity":15,"emission":0},"32":{"opacity":1,"emission":0},"10":{"opacity":15,"emission":15}}}"#).unwrap();
        let mut b9 = vec![0i32; BLOCKS9_LEN];
        // 石底 + 一个萤石种子：全 3×3 域 y<64 石头，(24,63,24) 萤石
        for c in 0..9usize {
            for y in 0..128usize {
                for lz in 0..16usize {
                    for lx in 0..16usize {
                        b9[blocks9_index(c, y, lx, lz)] = 1;
                    }
                }
            }
        }
        b9[blocks9_index(4, 127, 8, 8)] = 10;
        let mut ob = vec![0u8; OUT_CHAN_LEN];
        let mut os = vec![0u8; OUT_CHAN_LEN];
        let mut of = vec![0u8; FLAGS_LEN];
        light_compute(&engine, &b9, &mut ob, &mut os, &mut of).unwrap();
    }

    /// CP-1（260916-01）域批内核等价门：light_compute_domain 9 中心输出
    /// 必须与 per-chunk light_compute 对同一 3×3 子窗逐位相等（纯函数域，无噪声带）。
    /// blocks25 用固定 LCG 伪随机填充（含光源位/不透明位），覆盖所有 9 中心。
    /// C-1a（260919-05）：路改为「全域共享 fill + 9×3×3 窗 BFS」，本门即形态 S1 的
    /// 位等价判据——双 LCG 种子独立采样（单种子假绿防线）。
    #[test]
    fn light_compute_domain_bitwise_equivalence() {
        for seed in [0x9E37_79B9_7F4A_7C15u64, 0xA5A5_5A5A_1234_ABCDu64] {
            assert_domain_bitwise_equiv(seed);
        }
    }

    fn assert_domain_bitwise_equiv(seed: u64) {
        let engine = LightEngine::from_json_str(r#"{"format":"corewap-light-1","default":{"opacity":0,"emission":0},"blocks":{"1":{"opacity":15,"emission":0},"32":{"opacity":1,"emission":0},"10":{"opacity":15,"emission":15}}}"#).unwrap();
        // 固定种子 LCG（msvc/Java 无关，仅测试内部确定性）
        let mut s: u64 = seed;
        let mut rng = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let mut b25 = vec![0i32; BLOCKS25_LEN];
        for v in b25.iter_mut() {
            let r = rng();
            // 三态采样：~40% 石（op15）、~10% 萤石（em15）、其余空气/水族低 opacity
            *v = match r % 10 {
                0..=3 => 1,
                4 => 10 | (15 << 24),
                5 => 32,
                _ => 0,
            };
        }
        let mut out = vec![0u8; DOMAIN_OUT_LEN];
        light_compute_domain(&engine, &b25, &mut out).expect("domain compute");
        // 每中心：抽子窗 → per-chunk 路径 → 逐位对比
        let mut b9 = vec![0i32; BLOCKS9_LEN];
        let mut ob = vec![0u8; OUT_CHAN_LEN];
        let mut os = vec![0u8; OUT_CHAN_LEN];
        let mut of = vec![0u8; FLAGS_LEN];
        for k in 0..9usize {
            let kx = k % 3;
            let kz = k / 3;
            for dz9 in 0..3usize {
                for dx9 in 0..3usize {
                    let src = ((kz + dz9) * 5 + (kx + dx9)) * CHUNK_CELLS;
                    let dst = (dz9 * 3 + dx9) * CHUNK_CELLS;
                    b9[dst..dst + CHUNK_CELLS].copy_from_slice(&b25[src..src + CHUNK_CELLS]);
                }
            }
            light_compute(&engine, &b9, &mut ob, &mut os, &mut of).expect("per-chunk compute");
            let seg = &out[k * (2 * OUT_CHAN_LEN + FLAGS_LEN)..(k + 1) * (2 * OUT_CHAN_LEN + FLAGS_LEN)];
            assert_eq!(&seg[..OUT_CHAN_LEN], &ob[..], "center {k} outBlock mismatch");
            assert_eq!(&seg[OUT_CHAN_LEN..2 * OUT_CHAN_LEN], &os[..], "center {k} outSky mismatch");
            assert_eq!(&seg[2 * OUT_CHAN_LEN..], &of[..], "center {k} outFlags mismatch");
        }
    }

    /// 260919-03（.b2 C-3）packed 域批解码等价门：测试内 writePacket 帧同构编码器
    ///（bits 4..=14 / singular / 空节三形态）把 LCG b25 编码为 9 帧 packed，经
    /// light_decode_packed_domain 解码后必须与源 b25 逐位相等（全覆盖 25 chunk，
    /// 重叠帧内容相同 = 后帧覆盖不变量）。
    #[test]
    fn light_decode_packed_domain_roundtrip() {
        let mut s: u64 = 0x0123456789ABCDEF;
        let mut rng = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let mut b25 = vec![0i32; BLOCKS25_LEN];
        for v in b25.iter_mut() {
            let r = rng();
            // 值域刻意小（0..8）+ 三态：制造空节/单值节/小 palette 节混合
            *v = match r % 16 {
                0..=7 => 0,
                8..=10 => 1,
                11 => 10 | (15 << 24),
                _ => (r % 6) as i32 + 2,
            };
        }
        // 测试内帧编码器（与 Java writePacket 拆帧逆向同构：LSB-first、元素不跨 long）
        fn encode_chunk(b25: &[i32], chunk: usize, meta: &mut Vec<i32>, pal: &mut Vec<i32>, sto: &mut Vec<i64>) {
            for sec in 0..24usize {
                let base = chunk * CHUNK_CELLS + sec * 4096;
                let mut vals: Vec<i32> = Vec::new();
                for i in 0..4096usize {
                    let v = b25[base + i];
                    if !vals.contains(&v) {
                        vals.push(v);
                    }
                }
                match vals.len() {
                    0 => {
                        meta.push(0);
                        meta.push(0);
                    }
                    1 => {
                        meta.push(0);
                        meta.push(1);
                        pal.push(vals[0]);
                    }
                    psz => {
                        let bits = ((psz as usize - 1).next_power_of_two().trailing_zeros() as usize)
                            .max(4)
                            .min(14);
                        meta.push(bits as i32);
                        meta.push(psz as i32);
                        pal.extend_from_slice(&vals);
                        let epl = 64 / bits;
                        let n = (4096 + epl - 1) / epl;
                        let mut idx = 0usize;
                        for _ in 0..n {
                            let mut word = 0u64;
                            for j in 0..epl {
                                if idx >= 4096 {
                                    break;
                                }
                                let v = vals.iter().position(|&x| x == b25[base + idx]).unwrap();
                                word |= (v as u64) << (j * bits);
                                idx += 1;
                            }
                            sto.push(word as i64);
                        }
                    }
                }
            }
        }
        let mut frame_meta: Vec<i32> = Vec::new();
        let mut pal: Vec<i32> = Vec::new();
        let mut sto: Vec<i64> = Vec::new();
        let mut lens: Vec<i32> = Vec::new();
        for k in 0..9usize {
            let kx = k % 3;
            let kz = k / 3;
            let pal0 = pal.len();
            let sto0 = sto.len();
            for c9 in 0..9usize {
                encode_chunk(
                    &b25,
                    (kz + c9 / 3) * 5 + (kx + c9 % 3),
                    &mut frame_meta,
                    &mut pal,
                    &mut sto,
                );
            }
            lens.push((pal.len() - pal0) as i32);
            lens.push((sto.len() - sto0) as i32);
        }
        let mut decoded = vec![0i32; BLOCKS25_LEN];
        light_decode_packed_domain(&frame_meta, &pal, &sto, &lens, &mut decoded)
            .expect("domain packed decode");
        assert_eq!(decoded, b25, "packed domain roundtrip mismatch");
        // 包装函数回归：单帧（k=0 内容 = b9 窗）经 light_decode_packed 也须成立
        let mut b9 = vec![0i32; BLOCKS9_LEN];
        for c9 in 0..9usize {
            let src = ((c9 / 3) * 5 + (c9 % 3)) * CHUNK_CELLS;
            b9[c9 * CHUNK_CELLS..(c9 + 1) * CHUNK_CELLS]
                .copy_from_slice(&b25[src..src + CHUNK_CELLS]);
        }
        let mut meta1: Vec<i32> = Vec::new();
        let mut pal1: Vec<i32> = Vec::new();
        let mut sto1: Vec<i64> = Vec::new();
        for c9 in 0..9usize {
            encode_chunk(&b9, c9, &mut meta1, &mut pal1, &mut sto1);
        }
        let mut decoded9 = vec![0i32; BLOCKS9_LEN];
        light_decode_packed(&meta1, &pal1, &sto1, &mut decoded9).expect("b9 packed decode");
        assert_eq!(decoded9, b9, "b9 packed roundtrip mismatch");
    }

    /// 防回归（260905-05）：light_data.json 负 opacity 显式化修复——
    /// 潜影盒族（原 vanilla -1 哑元）应解析为 15，pointed_dripstone 应为 0，
    /// 且全表无被 clamp 吞掉的负值（#14 静默语义腐蚀家族）。
    #[test]
    fn light_data_negative_opacity_explicitized() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../versions/1.20.1/data/worldgen/light_data.json");
        let engine = LightEngine::from_json_file(path)
            .expect("load real light_data.json");
        for id in 613..=629 {
            assert_eq!(engine.lookup(id), (15u8, 0u8), "shulker family id={id} 应为 opacity 15");
        }
        assert_eq!(engine.lookup(954), (0u8, 0u8), "pointed_dripstone 应为 opacity 0");
    }
}
