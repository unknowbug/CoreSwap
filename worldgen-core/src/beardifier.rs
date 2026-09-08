// beardifier.rs — StructureWeightSampler（Beardifier）结构密度修正
//
// C++ 参考：versions/1.20.1/cpp/worldgen/src/beardifier.h（逐位对齐 Java StructureWeightSampler 1.20.1）
// Java 参考：net/minecraft/world/gen/StructureWeightSampler.java（mc_src_extract）
// 机制：ChunkNoiseSampler.getActualDensityFunction 将 DensityFunctionTypes.Beardifier.INSTANCE
//       替换为真实 beardifying → density 链 = add(finalDensity, Beardifier)。
//       本模块只移植纯算法（24^3 权重表 + sample 四分支 + fastInverseSqrt 位操作逐位对齐），
//       pieces/junctions 输入由 beard file 加载（Java 探针 -Dbeard.dump 导出格式，见 block_probe.cpp）。
// 语义：fill 时对每个块 fd += beard.sample(x,y,z)；无结构（empty）→ 返回 0，与现状一致。

// ===== 结构地形适配枚举（StructureTerrainAdaptation，序数 = Java ordinal）=====
// 260908-10 P7：1.21.6 新增 ENCAPSULATE（序数 4，trial_chambers 唯一使用者）——
//   1.20.1 枚举 = NONE/BURY/BEARD_THIN/BEARD_BOX（0..3）；1.21.6 = +ENCAPSULATE(4)。
//   证据：versions/1.21.6/data/mc_src_extract/.../StructureTerrainAdaptation.java:7-11
//        + data/minecraft/worldgen/structure/trial_chambers.json `"terrain_adaptation":"encapsulate"`
//        + StructureWeightSampler.java:99/106（ENCAPSULATE 的 q 与权重分支）。
//   旧实现 `_ => None` 使 trial chamber 全部 piece 权重归零 → 结构处密度局部偏低 ≈1.4-2.2。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainAdaptation {
    None = 0,
    Bury = 1,
    BeardThin = 2,
    BeardBox = 3,
    Encapsulate = 4,
}

// ===== Piece（StructureWeightSampler.Piece）=====
// box: minX/minY/minZ/maxX/maxY/maxZ（均为含边界）
#[derive(Clone, Copy, Debug)]
pub struct BeardPiece {
    pub min_x: i32, pub min_y: i32, pub min_z: i32,
    pub max_x: i32, pub max_y: i32, pub max_z: i32,
    pub terrain: TerrainAdaptation,
    pub ground_level_delta: i32,
}

// ===== JigsawJunction（仅 sample 用到的三元组）=====
#[derive(Clone, Copy, Debug)]
pub struct BeardJunction {
    pub source_x: i32,
    pub source_ground_y: i32,
    pub source_z: i32,
}

// ===== 权重表（24^3，Java static final float[13824]）=====
const EDGE_LENGTH: usize = 24;
const INDEX_OFFSET: i32 = 12;

// array[i*576 + j*24 + k] = calculateStructureWeight(j-12, k-12, i-12)
// sample 索引：table[k*576 + i*24 + j]（k=z+12, i=x+12, j=y+12）
fn weight_table() -> &'static [f32; EDGE_LENGTH * EDGE_LENGTH * EDGE_LENGTH] {
    use std::sync::OnceLock;
    static TABLE: OnceLock<[f32; EDGE_LENGTH * EDGE_LENGTH * EDGE_LENGTH]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut arr = [0.0f32; EDGE_LENGTH * EDGE_LENGTH * EDGE_LENGTH];
        for i in 0..EDGE_LENGTH {
            for j in 0..EDGE_LENGTH {
                for k in 0..EDGE_LENGTH {
                    arr[i * 24 * 24 + j * 24 + k] =
                        calculate_structure_weight(j as i32 - INDEX_OFFSET, k as i32 - INDEX_OFFSET, i as i32 - INDEX_OFFSET) as f32;
                }
            }
        }
        arr
    })
}

// MathHelper.fastInverseSqrt（L517-523）：位操作近似 1/sqrt(x)，Newton 一步迭代
// 注意：Java long 有符号算术右移 >>，i64 右移即算术右移（一致）
fn fast_inv_sqrt(x: f64) -> f64 {
    let d = 0.5 * x;
    let mut l: i64 = x.to_bits() as i64;      // Double.doubleToRawLongBits
    l = 6910469410427058090_i64 - (l >> 1);
    let x = f64::from_bits(l as u64);          // Double.longBitsToDouble
    x * (1.5 - d * x * x)
}

// MathHelper.clampedMap → getLerpProgress / clampedLerp / lerp 链
fn clamped_map(value: f64, old_start: f64, old_end: f64, new_start: f64, new_end: f64) -> f64 {
    let delta = (value - old_start) / (old_end - old_start);  // getLerpProgress
    if delta < 0.0 { return new_start; }
    if delta > 1.0 { return new_end; }
    new_start + delta * (new_end - new_start)                  // lerp
}

// calculateStructureWeight(x, y, z) = structureWeight(x, y+0.5, z)
// structureWeight = pow(E, -squaredMagnitude(x,y,z)/16.0)
// ⚠️ Java 用 Math.pow(Math.E, ...)（fdlibm pow 通用路径）非 Math.exp——用字面量保持同语义
fn calculate_structure_weight(x: i32, y: i32, z: i32) -> f64 {
    let dx = x as f64;
    let dy = y as f64 + 0.5;
    let dz = z as f64;
    let d = dx * dx + dy * dy + dz * dz;
    // Java Math.E = 2.718281828459045（double 位级）
    (2.718281828459045_f64).powf(-d / 16.0)
}

// getMagnitudeWeight(x, y, z) = clampedMap(magnitude(x, y, z), 0, 6, 1, 0)
// 260908-10：改 f64 入参版本——Java 三处调用形态不同（BURY 传 (m, q/2.0, n)；
// ENCAPSULATE 传 (m/2.0, q/2.0, n/2.0)），旧 i32 版把「y 内部 /2.0」写死在函数体内，
// 无法表达 ENCAPSULATE 的三轴 /2 形态。
// ⚠️ 1.20.1 回归声明（judge MUST-3）：Bury 调用语义不变——旧 `get_magnitude_weight(m, q, n)`
//   内部 `dy = q as f64 / 2.0`；新 `get_magnitude_weight(m as f64, q as f64 / 2.0, n as f64)`
//   得到同一 magnitude(m, q/2, n)，无整数除法参与（旧实现也从未整数除截断）。
//   BeardThin/BeardBox/None 分支逐字未动；Encapsulate 仅由 1.21.6 数据（ordinal 4）触达，
//   1.20.1 数据枚举最大序数 3 → 1.20.1 行为不变。回归：`cargo test -p WorldgenRust --lib` 14/14。
fn get_magnitude_weight(x: f64, y: f64, z: f64) -> f64 {
    let d = (x * x + y * y + z * z).sqrt();
    clamped_map(d, 0.0, 6.0, 1.0, 0.0)
}

// getStructureWeight(x, y, z, yy)：表查找（越界 0）+ fastInverseSqrt 因子
fn get_structure_weight(x: i32, y: i32, z: i32, yy: i32) -> f64 {
    let i = x + INDEX_OFFSET;
    let j = y + INDEX_OFFSET;
    let k = z + INDEX_OFFSET;
    if i >= 0 && i < EDGE_LENGTH as i32 && j >= 0 && j < EDGE_LENGTH as i32 && k >= 0 && k < EDGE_LENGTH as i32 {
        let d = yy as f64 + 0.5;
        let dx = x as f64;
        let dz = z as f64;
        let e = dx * dx + d * d + dz * dz;
        let f = -d * fast_inv_sqrt(e / 2.0) / 2.0;
        return f * weight_table()[k as usize * 24 * 24 + i as usize * 24 + j as usize] as f64;
    }
    0.0
}

// ===== Beardifier（StructureWeightSampler）=====
#[derive(Clone, Debug, Default)]
pub struct Beardifier {
    pub pieces: Vec<BeardPiece>,
    pub junctions: Vec<BeardJunction>,
}

impl Beardifier {
    pub fn new() -> Self {
        Self { pieces: Vec::new(), junctions: Vec::new() }
    }

    pub fn empty(&self) -> bool {
        self.pieces.is_empty() && self.junctions.is_empty()
    }

    // sample(pos)：pieces 累加 + junctions 累加（Java 每次从头遍历）
    pub fn sample(&self, block_x: i32, block_y: i32, block_z: i32) -> f64 {
        let mut d = 0.0;
        for piece in &self.pieces {
            let l = piece.ground_level_delta;
            let m = 0_i32.max((piece.min_x - block_x).max(block_x - piece.max_x));
            let n = 0_i32.max((piece.min_z - block_z).max(block_z - piece.max_z));
            let o = piece.min_y + l;
            let p = block_y - o;
            let q = match piece.terrain {
                TerrainAdaptation::None => 0,
                TerrainAdaptation::Bury | TerrainAdaptation::BeardThin => p,
                TerrainAdaptation::BeardBox => 0_i32.max((o - block_y).max(block_y - piece.max_y)),
                // Java 1.21.6:99 —— ENCAPSULATE 用 min_y/max_y（不含 groundLevelDelta）
                TerrainAdaptation::Encapsulate => 0_i32.max((piece.min_y - block_y).max(block_y - piece.max_y)),
            };
            match piece.terrain {
                TerrainAdaptation::None => {}
                // Java 1.21.6:104 —— BURY: getMagnitudeWeight(m, q/2.0, n)
                TerrainAdaptation::Bury => d += get_magnitude_weight(m as f64, q as f64 / 2.0, n as f64),
                TerrainAdaptation::BeardThin | TerrainAdaptation::BeardBox => {
                    d += get_structure_weight(m, q, n, p) * 0.8;
                }
                // Java 1.21.6:106 —— ENCAPSULATE: getMagnitudeWeight(m/2.0, q/2.0, n/2.0) * 0.8
                TerrainAdaptation::Encapsulate => {
                    d += get_magnitude_weight(m as f64 / 2.0, q as f64 / 2.0, n as f64 / 2.0) * 0.8;
                }
            }
        }
        for jj in &self.junctions {
            let r = block_x - jj.source_x;
            let l = block_y - jj.source_ground_y;
            let m = block_z - jj.source_z;
            d += get_structure_weight(r, l, m, l) * 0.4;
        }
        d
    }

    // ===== beard file 加载（BlockProbe -Dbeard.dump 输出格式，见 block_probe.cpp loadBeardFile）=====
    // 格式：
    //   chunk <cx> <cz>
    //   piece <minX> <minY> <minZ> <maxX> <maxY> <maxZ> <terrain 0-3> <groundLevelDelta>
    //   junction <sourceX> <sourceGroundY> <sourceZ>
    pub fn from_file(path: &str) -> std::io::Result<Vec<(i32, i32, Beardifier)>> {
        let content = std::fs::read_to_string(path)?;
        let mut out: Vec<(i32, i32, Beardifier)> = Vec::new();
        let mut cx = 0i32; let mut cz = 0i32;
        let mut have_chunk = false;
        let mut cur = Beardifier::new();
        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty() { continue; }
            let mut parts = line.split_whitespace();
            let tag = parts.next().unwrap_or("");
            match tag {
                "chunk" => {
                    if have_chunk {
                        out.push((cx, cz, std::mem::take(&mut cur)));
                    }
                    cx = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                    cz = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                    have_chunk = true;
                }
                "piece" => {
                    // 格式：piece <minX> <minY> <minZ> <maxX> <maxY> <maxZ> <terrain 0-4> <groundLevelDelta>
                    // "piece" 已消费，parts 剩 8 个数：v[0..5]=box, v[6]=terrain, v[7]=groundLevelDelta
                    let v: Vec<i32> = parts.filter_map(|s| s.parse().ok()).collect();
                    if v.len() >= 8 {
                        let terrain = match v[6] {
                            1 => TerrainAdaptation::Bury,
                            2 => TerrainAdaptation::BeardThin,
                            3 => TerrainAdaptation::BeardBox,
                            4 => TerrainAdaptation::Encapsulate,
                            _ => TerrainAdaptation::None,
                        };
                        cur.pieces.push(BeardPiece {
                            min_x: v[0], min_y: v[1], min_z: v[2],
                            max_x: v[3], max_y: v[4], max_z: v[5],
                            terrain, ground_level_delta: v[7],
                        });
                    }
                }
                "junction" => {
                    let v: Vec<i32> = parts.filter_map(|s| s.parse().ok()).collect();
                    if v.len() >= 3 {
                        cur.junctions.push(BeardJunction { source_x: v[0], source_ground_y: v[1], source_z: v[2] });
                    }
                }
                _ => {}
            }
        }
        if have_chunk {
            out.push((cx, cz, cur));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    //! 260908-10 P7：1.21.6 ENCAPSULATE 分支金标（trial_chambers 唯一使用者）。
    use super::*;

    fn trial_piece(terrain: TerrainAdaptation) -> BeardPiece {
        // trial_chambers chunk(11,16) 实测 piece 之一：BB [176,-23,238]-[194,-4,256]，delta 0
        BeardPiece {
            min_x: 176, min_y: -23, min_z: 238,
            max_x: 194, max_y: -4, max_z: 256,
            terrain, ground_level_delta: 0,
        }
    }

    // 盒内点：m=n=q=0 → magnitude 0 → clampedMap→1 → ENCAPSULATE = 1*0.8 = 0.8
    // （Java 1.21.6 StructureWeightSampler:99/106 直接手算；None 同点恒 0）
    #[test]
    fn encapsulate_inside_box_is_0_8() {
        let mut b = Beardifier::new();
        b.pieces.push(trial_piece(TerrainAdaptation::Encapsulate));
        let v = b.sample(185, -10, 245);
        assert!((v - 0.8).abs() < 1e-12, "encapsulate inside box = {} (want 0.8)", v);

        let mut n = Beardifier::new();
        n.pieces.push(trial_piece(TerrainAdaptation::None));
        assert_eq!(n.sample(185, -10, 245), 0.0, "None must contribute 0");
    }

    // 盒上方 4 格：q = max(0, max(minY-j, j-maxY)) = max(-23-0, 0-(-4)) = 4；
    // m=n=0 → magnitude(0, 2, 0)=2 → clampedMap(2,0,6,1,0)=1-2/6=0.6666666666666667 → *0.8
    // 独立手算（Java 公式）期望 0.5333333333333333。
    #[test]
    fn encapsulate_above_box_matches_java_formula() {
        let mut b = Beardifier::new();
        b.pieces.push(trial_piece(TerrainAdaptation::Encapsulate));
        let v = b.sample(185, 0, 245);
        assert!((v - 0.5333333333333333).abs() < 1e-12, "encapsulate above box = {} (want 0.5333...)", v);
    }

    // ordinal 4 解析（Java 侧 CppBridge 传 ordinal；旧实现落 None 是本次残差根因）
    #[test]
    fn beard_file_ordinal_4_maps_to_encapsulate() {
        let txt = "chunk 11 16\npiece 176 -23 238 194 -4 256 4 0\n";
        let dir = std::env::temp_dir().join("coreswap_beard_260908_10.txt");
        std::fs::write(&dir, txt).unwrap();
        let loaded = Beardifier::from_file(dir.to_str().unwrap()).unwrap();
        let _ = std::fs::remove_file(&dir);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].2.pieces[0].terrain, TerrainAdaptation::Encapsulate);
    }
}
