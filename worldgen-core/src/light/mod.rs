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

use std::cell::RefCell;

// ---- 域常量（3×3 chunk 邻域，扁平域坐标 x/z 0..48，y 0..384 = 世界 y-64..319）----
pub const DOM: usize = 48;
pub const WORLD_H: usize = 384;
pub const SECTIONS: usize = 24;
pub const SECTION_BYTES: usize = 2048;
pub const CHUNK_CELLS: usize = 16 * 16 * WORLD_H; // 98304
pub const BLOCKS9_LEN: usize = 9 * CHUNK_CELLS; // 884736
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
    scratch: RefCell<Scratch>,
}

struct Scratch {
    opacity: Vec<u8>,
    block_light: Vec<u8>,
    sky_light: Vec<u8>,
    queue: Vec<u32>,
    /// 每域列 (x + z*DOM) 的最高不透明 y（无则 -1）——fill 同趟收集，sky 阶段复用
    col_max: Vec<i32>,
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
            scratch: RefCell::new(Scratch {
                opacity: Vec::new(),
                block_light: Vec::new(),
                sky_light: Vec::new(),
                queue: Vec::new(),
                col_max: Vec::new(),
            }),
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
    light_compute_inner(engine, blocks9, out_block, out_sky, out_flags, None)
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
    blocks9_out.fill(0); // 空节哨兵 = 保持 0（AIR）
    let mut pal_cur = 0usize;
    let mut sto_cur = 0usize;
    for c in 0..9 {
        for s in 0..SECTIONS {
            let i = (c * SECTIONS + s) * 2;
            let bits = section_meta[i];
            let psz = section_meta[i + 1] as usize;
            if bits == 0 && psz == 0 {
                continue; // 空节：已零填充
            }
            let base = c * CHUNK_CELLS + s * 4096;
            if bits == 0 {
                if psz != 1 || palette_data.len() < pal_cur + 1 {
                    return Err(LightError::PackedDecode);
                }
                let v = palette_data[pal_cur];
                pal_cur += 1;
                blocks9_out[base..base + 4096].fill(v);
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
            let n = (4096 + epl - 1) / epl;
            if packed.len() < sto_cur + n {
                return Err(LightError::PackedDecode);
            }
            let words = &packed[sto_cur..sto_cur + n];
            sto_cur += n;
            let bits_u = bits as usize;
            let mask = (1u64 << bits_u) - 1;
            let long_bits = epl * bits_u; // 元素不跨 long ⇒ off < 64 恒成立
            let mut dst = base;
            let mut w = 0usize;
            let mut off = 0usize;
            for _ in 0..4096 {
                let v = ((words[w] as u64 >> off) & mask) as usize;
                if v >= psz {
                    return Err(LightError::PackedDecode);
                }
                blocks9_out[dst] = pal[v];
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
    light_compute_inner(engine, blocks9, out_block, out_sky, out_flags, Some(&mut t.0))?;
    Ok(t)
}

fn light_compute_inner(
    engine: &LightEngine,
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

    let mut sc = engine.scratch.borrow_mut();
    let Scratch { opacity, block_light, sky_light, queue, col_max } = &mut *sc;

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

    // 1. blocks9 → 域 opacity/emission。ABI：低 24 位 = raw id，高 8 位 = luminance 真值
    //    （state 级，Java 侧提供；为 0 时回退数据表 emission —— 表仍作 id 级回退源）。
    //    同时收集 block light 种子（emission>0）。
    //    round2：dom-major 重排（写侧连续，消 dom_index 乘法散写），同趟收集 col_max
    //    （每列最高不透明 y，sky 阶段的 15-区间即 col_max+1..383，省一趟 O(N) opacity 读）。
    for y in 0..WORLD_H {
        let yb = y * 256;
        for z in 0..DOM {
            let cz = z >> 4;
            let lz = z & 15;
            let row = (y * DOM + z) * DOM;
            let colrow = z * DOM;
            for cx in 0..3usize {
                let c = cz * 3 + cx;
                let base = c * CHUNK_CELLS + yb + lz * 16;
                let d = row + cx * 16;
                for lx in 0..16usize {
                    let v = blocks9[base + lx] as u32;
                    let i = d + lx;
                    // air 快路径：v==0（id=0 且 luminance=0，luminance 高位非 0 不得走此路径）
                    // 且 table[0]==(0,0) ⇒ op/em 全 0。⚠️ id==0 但 lum>0 是合法光源，必须落查表。
                    if engine.air_fast && v == 0 {
                        // opacity[i] 保持 0（scratch 清零语义）；block_light 同
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
                        block_light[i] = em;
                        queue.push(i as u32);
                    }
                    if op > 0 {
                        col_max[colrow + cx * 16 + lx] = y as i32;
                    }
                }
            }
        }
    }
    if let Some(p) = phases.as_deref_mut() {
        p[0] += _t0.map(|t| t.elapsed()).unwrap_or_default();
    }
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
