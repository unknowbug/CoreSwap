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
    scratch: RefCell<Scratch>,
}

struct Scratch {
    opacity: Vec<u8>,
    block_light: Vec<u8>,
    sky_light: Vec<u8>,
    queue: Vec<u32>,
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
            table,
            scratch: RefCell::new(Scratch {
                opacity: Vec::new(),
                block_light: Vec::new(),
                sky_light: Vec::new(),
                queue: Vec::new(),
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
}

#[inline]
fn parse_u8_field(v: &crate::json::JsonValue, key: &str, default: u8) -> u8 {
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
    if blocks9.len() != BLOCKS9_LEN {
        return Err(LightError::InputLen);
    }
    if out_block.len() < OUT_CHAN_LEN || out_sky.len() < OUT_CHAN_LEN || out_flags.len() < FLAGS_LEN {
        return Err(LightError::OutputLen);
    }

    let mut sc = engine.scratch.borrow_mut();
    let Scratch { opacity, block_light, sky_light, queue } = &mut *sc;

    // clear+resize：scratch 跨调用复用，必须清零（resize 不覆盖已有元素）
    opacity.clear();
    opacity.resize(BLOCKS9_LEN, 0);
    block_light.clear();
    block_light.resize(BLOCKS9_LEN, 0);
    sky_light.clear();
    sky_light.resize(BLOCKS9_LEN, 0);
    queue.clear();

    // 1. blocks9 → 域 opacity/emission。ABI：低 24 位 = raw id，高 8 位 = luminance 真值
    //    （state 级，Java 侧提供；为 0 时回退数据表 emission —— 表仍作 id 级回退源）。
    //    同时收集 block light 种子（emission>0）。
    for c in 0..9usize {
        for y in 0..WORLD_H {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    let v = blocks9[blocks9_index(c, y, lx, lz)] as u32;
                    let id = (v & 0x00FF_FFFF) as i32;
                    let lum = (v >> 24) as u8;
                    let (op, mut em) = engine.lookup(id);
                    if lum > 0 {
                        em = lum;
                    }
                    let i = dom_index((c % 3) * 16 + lx, y, (c / 3) * 16 + lz);
                    opacity[i] = op;
                    if em > 0 {
                        block_light[i] = em;
                        queue.push(i as u32);
                    }
                }
            }
        }
    }

    // 2. block light：BFS（6 邻域，cost = max(1, opacity[邻])，无 sky 直落特例）
    bfs_propagate(block_light, opacity, queue, false);

    queue.clear();

    // 3. sky light：柱状直落（opacity 0 一路 15，遇 opacity>0 停）+ BFS 水平/向下扩散
    for z in 0..DOM {
        for x in 0..DOM {
            let mut blocked = false;
            for y in (0..WORLD_H).rev() {
                let i = dom_index(x, y, z);
                if opacity[i] > 0 {
                    blocked = true;
                } else if !blocked {
                    sky_light[i] = 15;
                }
            }
        }
    }
    for i in 0..BLOCKS9_LEN {
        if sky_light[i] == 15 {
            queue.push(i as u32);
        }
    }
    bfs_propagate(sky_light, opacity, queue, true);

    // 4. 导出中心 chunk（域 x/z 16..32）+ 均质性 flags
    export_center(block_light, sky_light, out_block, out_sky, out_flags);

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
}
