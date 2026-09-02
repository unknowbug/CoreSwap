// 自动生成（gen_tables_rs.py，DFC Rust 后端数据表），勿手改。同源：dfc_gen.py gen_cpu/gen_df + _compute_val_layout
pub const SPLIT_TOTAL: usize = 8672;
pub const PERM_SIZE: usize = 356352;
pub const MIN_Y: i32 = -64; // overworld 维度 minY（interpolated cell 网格）
pub const N_INTERP: usize = 5;
pub const N_SHIFTS: usize = 1;
pub const N_NORMALS: usize = 192;
pub const N_OLDS: usize = 8;
pub const SPLINE_BIND_BASE: usize = 6;
pub const SPLINE_NODES: usize = 56;
pub const DF_NODES: usize = 163;
pub const TOP_ROOT: usize = 162;
pub const N_CLOSURE: usize = 5;
pub const CLOSURE_MAX_SLOTS: usize = 18;
pub const TOP_CLOSURE_LEN: usize = 21;
pub const VAL_SLOTS_TOP: usize = 8;
pub const TOP_ROOT_POS: usize = 20;
pub const NORMAL_INSTANCES: usize = 200;
pub const NOISE_SLOT_COUNT: usize = 25;
pub const COORD_TYPES: usize = 4;

#[derive(Clone, Copy)] pub struct NoiseInit { pub key: &'static str, pub first_octave: i32, pub amps: &'static [f64] }
#[derive(Clone, Copy)] pub struct OldInit { pub xz_scale: f64, pub y_scale: f64, pub xz_factor: f64, pub y_factor: f64, pub smear: f64 }

pub static SHIFT_INIT: [NoiseInit; N_SHIFTS] = [
    NoiseInit { key: "minecraft:offset", first_octave: -3, amps: &[1.0, 1.0, 1.0, 0.0] },
];
pub static NORMAL_INIT: [NoiseInit; N_NORMALS] = [
    NoiseInit { key: "minecraft:continentalness", first_octave: -9, amps: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:continentalness", first_octave: -9, amps: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:continentalness", first_octave: -9, amps: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:continentalness", first_octave: -9, amps: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:continentalness", first_octave: -9, amps: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:continentalness", first_octave: -9, amps: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:continentalness", first_octave: -9, amps: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:continentalness", first_octave: -9, amps: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:erosion", first_octave: -9, amps: &[1.0, 1.0, 0.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:erosion", first_octave: -9, amps: &[1.0, 1.0, 0.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:erosion", first_octave: -9, amps: &[1.0, 1.0, 0.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:erosion", first_octave: -9, amps: &[1.0, 1.0, 0.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:erosion", first_octave: -9, amps: &[1.0, 1.0, 0.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:erosion", first_octave: -9, amps: &[1.0, 1.0, 0.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:erosion", first_octave: -9, amps: &[1.0, 1.0, 0.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:erosion", first_octave: -9, amps: &[1.0, 1.0, 0.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:ridge", first_octave: -7, amps: &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0] },
    NoiseInit { key: "minecraft:ridge", first_octave: -7, amps: &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0] },
    NoiseInit { key: "minecraft:ridge", first_octave: -7, amps: &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0] },
    NoiseInit { key: "minecraft:ridge", first_octave: -7, amps: &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0] },
    NoiseInit { key: "minecraft:ridge", first_octave: -7, amps: &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0] },
    NoiseInit { key: "minecraft:ridge", first_octave: -7, amps: &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0] },
    NoiseInit { key: "minecraft:ridge", first_octave: -7, amps: &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0] },
    NoiseInit { key: "minecraft:ridge", first_octave: -7, amps: &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0] },
    NoiseInit { key: "minecraft:jagged", first_octave: -16, amps: &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:jagged", first_octave: -16, amps: &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:jagged", first_octave: -16, amps: &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:jagged", first_octave: -16, amps: &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:jagged", first_octave: -16, amps: &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:jagged", first_octave: -16, amps: &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:jagged", first_octave: -16, amps: &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:jagged", first_octave: -16, amps: &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0] },
    NoiseInit { key: "minecraft:cave_entrance", first_octave: -7, amps: &[0.40000000000000002, 0.5, 1.0] },
    NoiseInit { key: "minecraft:cave_entrance", first_octave: -7, amps: &[0.40000000000000002, 0.5, 1.0] },
    NoiseInit { key: "minecraft:cave_entrance", first_octave: -7, amps: &[0.40000000000000002, 0.5, 1.0] },
    NoiseInit { key: "minecraft:cave_entrance", first_octave: -7, amps: &[0.40000000000000002, 0.5, 1.0] },
    NoiseInit { key: "minecraft:cave_entrance", first_octave: -7, amps: &[0.40000000000000002, 0.5, 1.0] },
    NoiseInit { key: "minecraft:cave_entrance", first_octave: -7, amps: &[0.40000000000000002, 0.5, 1.0] },
    NoiseInit { key: "minecraft:cave_entrance", first_octave: -7, amps: &[0.40000000000000002, 0.5, 1.0] },
    NoiseInit { key: "minecraft:cave_entrance", first_octave: -7, amps: &[0.40000000000000002, 0.5, 1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness_modulator", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness_modulator", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness_modulator", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness_modulator", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness_modulator", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness_modulator", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness_modulator", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness_modulator", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness", first_octave: -5, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness", first_octave: -5, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness", first_octave: -5, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness", first_octave: -5, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness", first_octave: -5, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness", first_octave: -5, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness", first_octave: -5, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_roughness", first_octave: -5, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_rarity", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_rarity", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_rarity", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_rarity", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_rarity", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_rarity", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_rarity", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_rarity", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_1", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_1", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_1", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_1", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_1", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_1", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_1", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_1", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_2", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_2", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_2", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_2", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_2", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_2", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_2", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_2", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_3d_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:cave_layer", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:cave_layer", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:cave_layer", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:cave_layer", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:cave_layer", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:cave_layer", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:cave_layer", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:cave_layer", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:cave_cheese", first_octave: -8, amps: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0] },
    NoiseInit { key: "minecraft:cave_cheese", first_octave: -8, amps: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0] },
    NoiseInit { key: "minecraft:cave_cheese", first_octave: -8, amps: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0] },
    NoiseInit { key: "minecraft:cave_cheese", first_octave: -8, amps: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0] },
    NoiseInit { key: "minecraft:cave_cheese", first_octave: -8, amps: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0] },
    NoiseInit { key: "minecraft:cave_cheese", first_octave: -8, amps: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0] },
    NoiseInit { key: "minecraft:cave_cheese", first_octave: -8, amps: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0] },
    NoiseInit { key: "minecraft:cave_cheese", first_octave: -8, amps: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_modulator", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_modulator", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_modulator", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_modulator", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_modulator", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_modulator", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_modulator", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_modulator", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_thickness", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_thickness", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_thickness", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_thickness", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_thickness", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_thickness", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_thickness", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_thickness", first_octave: -11, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_elevation", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_elevation", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_elevation", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_elevation", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_elevation", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_elevation", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_elevation", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:spaghetti_2d_elevation", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar", first_octave: -7, amps: &[1.0, 1.0] },
    NoiseInit { key: "minecraft:pillar", first_octave: -7, amps: &[1.0, 1.0] },
    NoiseInit { key: "minecraft:pillar", first_octave: -7, amps: &[1.0, 1.0] },
    NoiseInit { key: "minecraft:pillar", first_octave: -7, amps: &[1.0, 1.0] },
    NoiseInit { key: "minecraft:pillar", first_octave: -7, amps: &[1.0, 1.0] },
    NoiseInit { key: "minecraft:pillar", first_octave: -7, amps: &[1.0, 1.0] },
    NoiseInit { key: "minecraft:pillar", first_octave: -7, amps: &[1.0, 1.0] },
    NoiseInit { key: "minecraft:pillar", first_octave: -7, amps: &[1.0, 1.0] },
    NoiseInit { key: "minecraft:pillar_rareness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_rareness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_rareness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_rareness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_rareness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_rareness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_rareness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_rareness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:pillar_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_thickness", first_octave: -8, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_a", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_a", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_a", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_a", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_a", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_a", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_a", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_a", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_b", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_b", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_b", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_b", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_b", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_b", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_b", first_octave: -7, amps: &[1.0] },
    NoiseInit { key: "minecraft:noodle_ridge_b", first_octave: -7, amps: &[1.0] },
];
pub static OLD_INIT: [OldInit; N_OLDS] = [
    OldInit { xz_scale: 0.25, y_scale: 0.125, xz_factor: 80.0, y_factor: 160.0, smear: 8.0 },
    OldInit { xz_scale: 0.25, y_scale: 0.125, xz_factor: 80.0, y_factor: 160.0, smear: 8.0 },
    OldInit { xz_scale: 0.25, y_scale: 0.125, xz_factor: 80.0, y_factor: 160.0, smear: 8.0 },
    OldInit { xz_scale: 0.25, y_scale: 0.125, xz_factor: 80.0, y_factor: 160.0, smear: 8.0 },
    OldInit { xz_scale: 0.25, y_scale: 0.125, xz_factor: 80.0, y_factor: 160.0, smear: 8.0 },
    OldInit { xz_scale: 0.25, y_scale: 0.125, xz_factor: 80.0, y_factor: 160.0, smear: 8.0 },
    OldInit { xz_scale: 0.25, y_scale: 0.125, xz_factor: 80.0, y_factor: 160.0, smear: 8.0 },
    OldInit { xz_scale: 0.25, y_scale: 0.125, xz_factor: 80.0, y_factor: 160.0, smear: 8.0 },
];

pub const DF_TYPE: [i32; 163] = [0, 0, 18, 0, 0, 18, 0, 0, 18, 0, 0, 0, 7, 6, 7, 0, 4, 6, 7, 6, 21, 6, 4, 6, 7, 6, 21, 2, 13, 7, 6, 0, 0, 4, 6, 7, 6, 21, 7, 14, 7, 3, 6, 0, 0, 2, 6, 18, 6, 0, 2, 7, 6, 0, 2, 10, 6, 7, 2, 22, 22, 9, 0, 0, 2, 7, 6, 6, 16, 6, 8, 7, 8, 2, 11, 7, 0, 2, 6, 16, 0, 0, 7, 6, 16, 6, 6, 8, 2, 22, 0, 0, 0, 2, 7, 6, 7, 6, 0, 2, 7, 6, 18, 6, 10, 6, 12, 9, 16, 6, 8, 0, 2, 7, 2, 7, 6, 6, 0, 2, 7, 6, 12, 7, 0, 17, 9, 17, 6, 7, 6, 6, 7, 6, 20, 5, 7, 15, 1, 2, 17, 5, 0, 0, 0, 2, 7, 6, 17, 5, 2, 17, 5, 10, 2, 17, 5, 10, 9, 7, 6, 17, 8];
pub const DF_A1: [i32; 163] = [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, 10, 9, -1, 28, 15, 17, 14, 19, 8, 36, 9, 10, 9, 25, 3, 27, 26, 21, -1, -1, 55, 32, 10, 31, 36, 30, 38, 7, 4, 40, -1, -1, 5, 44, -1, 46, -1, 6, 49, 49, -1, 7, 54, 53, 52, 8, 58, 58, 59, -1, -1, 11, 63, 62, 61, 67, 57, 48, 43, 42, 12, 73, 7, -1, 13, 76, 78, -1, -1, 81, 80, 83, 79, 75, 86, 14, 88, -1, -1, -1, 16, 92, 91, 90, 89, -1, 17, 98, 9, -1, 101, 103, 104, 105, 97, 107, 108, 87, -1, 18, 111, 19, 11, 11, 113, -1, 20, 118, 118, 121, 117, -1, 123, 110, 42, 6, 5, 4, 3, 2, 1, 133, 0, 0, 136, -1, 21, 138, 1, -1, -1, -1, 22, 144, 143, 138, 2, 23, 138, 3, 152, 24, 138, 4, 156, 153, 80, 149, 141, 137];
pub const DF_A2: [i32; 163] = [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10, 12, 13, -1, 1, 16, 10, 18, -1, 20, 1, 22, 23, 24, -1, -1, -1, 28, 29, -1, -1, 1, 33, 34, 35, -1, 37, -1, 39, -1, 41, -1, -1, -1, 45, -1, 47, -1, -1, 50, 51, -1, -1, -1, 55, 56, -1, 9, 10, 60, -1, -1, -1, 64, 65, 66, -1, 68, 69, 70, 71, -1, -1, 74, -1, -1, 77, -1, -1, -1, 42, 82, -1, 84, 85, 70, -1, 15, -1, -1, -1, -1, 93, 94, 95, 96, -1, -1, 99, 100, -1, 102, -1, 95, -1, 106, -1, 57, 109, -1, -1, 112, -1, 114, 115, 116, -1, -1, 119, 120, -1, 122, -1, 124, 125, 72, 127, 128, 129, 130, 131, 132, -1, -1, 135, -1, -1, -1, 139, -1, -1, -1, -1, -1, 145, 146, 147, -1, -1, 150, -1, -1, -1, 154, -1, -1, 157, 158, 159, 142, 161];
pub const DF_A3: [i32; 163] = [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 123, -1, 126, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, -1, -1, -1, -1, -1, -1, -1, 9, -1, -1, 9, -1, -1, -1, 9, -1, -1, -1, -1, -1, 160, -1];
pub const DF_F0: [f32; 163] = [0.64, 0.1171875, -64.0, -0.1171875, -0.078125, 240.0, 0.078125, 4.0, -64.0, 0.0, 1.0, -1.0, 0.0, 0.0, 0.0, -0.503750026, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 10.0, -10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 0.37, 0.0, 0.0, -10.0, 0.0, -0.05, 0.0, 0.0, 0.0, -0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.0765, -0.0115, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.27, 0.0, 0.0, -1.0, 1.5, -0.64, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.083, -0.95, -0.35, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0, 0.0, 0.0, 0.0, -64.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.55, 0.0, 0.0, 0.0, 0.0, 0.0, -1000000.0, -1000000.0, 0.0, -1000000.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -60.0, 0.0, 64.0, -0.075, -0.025, 0.0, 0.0, 0.0, -60.0, 0.0, 0.0, -60.0, 0.0, 0.0, 0.0, -60.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1000000.0, 0.0];
pub const DF_F1: [f32; 163] = [0.0, 0.0, -40.0, 0.0, 0.0, 256.0, 0.0, 0.0, 320.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 30.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 320.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.03, 0.0, 1.5625, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 321.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 321.0, 0.0, 0.0, 321.0, 0.0, 0.0, 0.0, 321.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
pub const DF_F2: [f32; 163] = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
pub const DF_F3: [f32; 163] = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -40.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
pub const INTERP_ROOTS: [i32; 5] = [134, 140, 148, 151, 155];
pub const CLOSURE_OFF: [i32; 5] = [0, 134, 155, 175, 192];
pub const CLOSURE_LEN: [i32; 5] = [134, 21, 20, 17, 18];
pub const CLOSURE_VAL_SLOTS: [i32; 5] = [18, 7, 6, 6, 6];
pub const CLOSURE_ROOT_POS: [i32; 5] = [133, 20, 19, 16, 17];
pub const CLOSURE_TYPE: [i32; 210] = [0, 18, 0, 0, 18, 0, 0, 18, 0, 0, 0, 7, 6, 7, 0, 4, 6, 7, 6, 21, 6, 4, 6, 7, 6, 21, 2, 13, 7, 6, 0, 0, 4, 6, 7, 6, 21, 7, 14, 7, 3, 6, 0, 0, 2, 6, 18, 6, 0, 2, 7, 6, 0, 2, 10, 6, 7, 2, 22, 22, 9, 0, 0, 2, 7, 6, 6, 16, 6, 8, 7, 8, 2, 11, 7, 0, 2, 6, 16, 0, 0, 7, 6, 16, 6, 6, 8, 2, 22, 0, 0, 0, 2, 7, 6, 7, 6, 0, 2, 7, 6, 18, 6, 10, 6, 12, 9, 16, 6, 8, 0, 2, 7, 2, 7, 6, 6, 0, 2, 7, 6, 12, 7, 0, 17, 9, 17, 6, 7, 6, 6, 7, 6, 20, 0, 0, 18, 0, 0, 0, 7, 6, 7, 0, 4, 6, 7, 6, 21, 6, 2, 13, 1, 2, 17, 0, 0, 0, 0, 4, 0, 0, 4, 6, 7, 6, 2, 10, 1, 0, 0, 2, 7, 6, 17, 0, 0, 0, 0, 4, 6, 0, 0, 4, 6, 7, 6, 2, 10, 1, 2, 17, 0, 0, 0, 0, 4, 6, 7, 0, 0, 4, 6, 7, 6, 2, 10, 1, 2, 17];
pub const CLOSURE_A1: [i32; 210] = [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10, 9, 8, -1, 28, 14, 16, 13, 18, 7, 36, 8, 9, 8, 24, 3, 26, 25, 20, -1, -1, 55, 31, 9, 30, 35, 29, 37, 6, 4, 39, -1, -1, 5, 43, -1, 45, -1, 6, 48, 48, -1, 7, 53, 52, 51, 8, 57, 57, 58, -1, -1, 11, 62, 61, 60, 66, 56, 47, 42, 41, 12, 72, 6, -1, 13, 75, 77, -1, -1, 80, 79, 82, 78, 74, 85, 14, 87, -1, -1, -1, 16, 91, 90, 89, 88, -1, 17, 97, 8, -1, 100, 102, 103, 104, 96, 106, 107, 86, -1, 18, 110, 19, 10, 10, 112, -1, 20, 117, 117, 120, 116, -1, 122, 109, 41, 5, 4, 3, 2, 1, 0, 132, -1, -1, -1, -1, -1, -1, 5, 4, 3, -1, 28, 9, 11, 8, 13, 2, 3, 16, -1, 21, 18, -1, -1, -1, -1, 36, -1, -1, 55, 6, 3, 5, 7, 11, -1, -1, -1, 22, 15, 14, 13, -1, -1, -1, -1, 36, 2, -1, -1, 55, 7, 3, 6, 7, 12, -1, 23, 14, -1, -1, -1, -1, 36, 2, 3, -1, -1, 55, 8, 3, 7, 7, 13, -1, 24, 15];
pub const CLOSURE_A2: [i32; 210] = [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 9, 11, 12, -1, 1, 15, 9, 17, -1, 19, 1, 21, 22, 23, -1, -1, -1, 27, 28, -1, -1, 1, 32, 33, 34, -1, 36, -1, 38, -1, 40, -1, -1, -1, 44, -1, 46, -1, -1, 49, 50, -1, -1, -1, 54, 55, -1, 9, 10, 59, -1, -1, -1, 63, 64, 65, -1, 67, 68, 69, 70, -1, -1, 73, -1, -1, 76, -1, -1, -1, 41, 81, -1, 83, 84, 69, -1, 15, -1, -1, -1, -1, 92, 93, 94, 95, -1, -1, 98, 99, -1, 101, -1, 94, -1, 105, -1, 56, 108, -1, -1, 111, -1, 113, 114, 115, -1, -1, 118, 119, -1, 121, -1, 123, 124, 71, 126, 127, 128, 129, 130, 131, -1, -1, -1, -1, -1, -1, -1, 4, 6, 7, -1, 1, 10, 4, 12, -1, 14, -1, -1, -1, -1, 19, -1, -1, -1, -1, 1, -1, -1, 1, 7, 8, 9, -1, -1, -1, -1, -1, -1, 16, 17, 18, -1, -1, -1, -1, 1, 4, -1, -1, 1, 8, 9, 10, -1, -1, -1, -1, 15, -1, -1, -1, -1, 1, 4, 5, -1, -1, 1, 9, 10, 11, -1, -1, -1, -1, 16];
pub const CLOSURE_A3: [i32; 210] = [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 122, -1, 125, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2];
pub const CLOSURE_F0: [f32; 210] = [0.1171875, -64.0, -0.1171875, -0.078125, 240.0, 0.078125, 4.0, -64.0, 0.0, 1.0, -1.0, 0.0, 0.0, 0.0, -0.503750026, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 10.0, -10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 0.37, 0.0, 0.0, -10.0, 0.0, -0.05, 0.0, 0.0, 0.0, -0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.0765, -0.0115, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.27, 0.0, 0.0, -1.0, 1.5, -0.64, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.083, -0.95, -0.35, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0, 0.0, 0.0, 0.0, -64.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.55, 0.0, 0.0, 0.0, 0.0, 0.0, -1000000.0, -1000000.0, 0.0, -1000000.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1171875, -0.1171875, -64.0, 0.0, 1.0, -1.0, 0.0, 0.0, 0.0, -0.503750026, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -60.0, 0.1171875, 4.0, 0.0, 1.0, 0.0, 10.0, -10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.075, -0.025, 0.0, 0.0, 0.0, -60.0, 0.1171875, 4.0, 0.0, 1.0, 0.0, 0.0, 10.0, -10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -60.0, 0.1171875, 4.0, 0.0, 1.0, 0.0, 0.0, 0.0, 10.0, -10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -60.0];
pub const CLOSURE_F1: [f32; 210] = [0.0, -40.0, 0.0, 0.0, 256.0, 0.0, 0.0, 320.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 30.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 320.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.03, 0.0, 1.5625, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 320.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 321.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 321.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 321.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 321.0];
pub const CLOSURE_F2: [f32; 210] = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
pub const CLOSURE_F3: [f32; 210] = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -40.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
pub const CLOSURE_SLOT: [i32; 210] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 11, 12, 13, 14, 12, 13, 11, 12, 7, 11, 7, 11, 7, 11, 13, 11, 7, 11, 12, 13, 14, 12, 9, 11, 9, 7, 9, 7, 11, 7, 9, 12, 13, 9, 12, 9, 13, 14, 13, 9, 14, 15, 14, 9, 13, 14, 15, 13, 14, 15, 16, 17, 15, 14, 13, 14, 13, 12, 7, 12, 14, 12, 6, 14, 15, 6, 14, 15, 16, 15, 14, 15, 6, 12, 6, 13, 6, 14, 15, 16, 17, 15, 14, 6, 13, 14, 16, 13, 8, 14, 8, 13, 8, 13, 6, 8, 6, 8, 9, 12, 8, 9, 8, 9, 8, 10, 12, 10, 8, 10, 8, 9, 8, 6, 7, 5, 4, 3, 2, 1, 0, 0, 0, 0, 1, 2, 3, 4, 5, 4, 1, 5, 6, 1, 2, 1, 2, 0, 1, 0, 1, 2, 0, 0, 0, 1, 2, 2, 3, 4, 5, 3, 1, 1, 2, 1, 2, 3, 4, 5, 3, 2, 0, 0, 0, 1, 2, 3, 2, 3, 4, 5, 3, 1, 1, 2, 1, 2, 3, 0, 0, 0, 1, 2, 3, 2, 2, 3, 4, 5, 3, 1, 1, 2, 1, 2, 3];
pub const TOP_TYPE: [i32; 21] = [0, 0, 18, 0, 0, 0, 5, 7, 15, 5, 0, 5, 5, 10, 5, 10, 9, 7, 6, 17, 8];
pub const TOP_A1: [i32; 21] = [-1, -1, -1, -1, -1, -1, 0, 0, 7, 1, -1, 2, 3, 12, 4, 14, 13, 5, 11, 9, 8];
pub const TOP_A2: [i32; 21] = [-1, -1, -1, -1, -1, -1, -1, 6, -1, -1, -1, -1, -1, -1, -1, -1, 15, 16, 17, 10, 19];
pub const TOP_A3: [i32; 21] = [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 18, -1];
pub const TOP_F0: [f32; 21] = [0.64, 0.1171875, -64.0, -0.1171875, -0.078125, 1.5, 0.0, 0.0, 0.0, 0.0, 64.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1000000.0, 0.0];
pub const TOP_F1: [f32; 21] = [0.0, 0.0, -40.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
pub const TOP_F2: [f32; 21] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
pub const TOP_F3: [f32; 21] = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
pub const TOP_SLOT: [i32; 21] = [0, 1, 1, 1, 1, 1, 2, 3, 0, 2, 3, 4, 5, 6, 5, 7, 5, 6, 1, 4, 1];
pub const SPLINE_NODE_PACK: [i32; 280] = [2, 2, 17, 17, 0, 2, 2, 19, 19, 2, 2, 6, 21, 21, 4, 2, 5, 27, 27, 10, 2, 5, 32, 32, 15, 2, 5, 37, 37, 20, 2, 5, 42, 42, 25, 1, 7, 10, 10, 30, 2, 5, 54, 54, 37, 2, 5, 59, 59, 42, 2, 5, 64, 64, 47, 2, 5, 69, 69, 52, 1, 7, 47, 47, 57, 2, 3, 85, 85, 64, 2, 3, 88, 88, 67, 2, 3, 91, 91, 70, 2, 5, 94, 94, 73, 2, 5, 99, 99, 78, 2, 3, 104, 104, 83, 1, 11, 74, 74, 86, 2, 3, 118, 118, 97, 2, 3, 121, 121, 100, 2, 5, 124, 124, 103, 2, 5, 129, 129, 108, 2, 5, 134, 134, 113, 2, 3, 139, 139, 118, 2, 5, 142, 142, 121, 1, 11, 107, 107, 126, 0, 10, 0, 0, 137, 3, 2, 157, 157, 147, 2, 3, 154, 154, 149, 3, 2, 162, 162, 152, 2, 3, 159, 159, 154, 1, 4, 150, 150, 157, 2, 3, 168, 168, 161, 1, 4, 164, 164, 164, 0, 3, 147, 147, 168, 3, 2, 186, 186, 171, 3, 2, 188, 188, 173, 3, 2, 190, 190, 175, 3, 2, 194, 194, 177, 2, 2, 192, 192, 179, 1, 10, 176, 176, 181, 3, 2, 206, 206, 191, 3, 2, 210, 210, 193, 2, 2, 208, 208, 195, 1, 10, 196, 196, 197, 3, 2, 222, 222, 207, 3, 2, 226, 226, 209, 2, 2, 224, 224, 211, 1, 10, 212, 212, 213, 3, 2, 239, 239, 223, 2, 2, 241, 241, 225, 2, 2, 243, 243, 227, 1, 11, 228, 228, 229, 0, 5, 171, 171, 240];
pub const SPLINE_LOCS: [f32; 245] = [-1.1, -1.02, -0.51, -0.44, -0.18, -0.16, -0.15, -0.1, 0.25, 1.0, -0.85, -0.7, -0.4, -0.35, -0.1, 0.2, 0.7, -1.0, 1.0, -1.0, 1.0, -1.0, -0.75, -0.65, 0.5954547, 0.6054547, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -0.85, -0.7, -0.4, -0.35, -0.1, 0.2, 0.7, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -0.85, -0.7, -0.4, -0.35, -0.1, 0.2, 0.4, 0.45, 0.55, 0.58, 0.7, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, -0.85, -0.7, -0.4, -0.35, -0.1, 0.2, 0.4, 0.45, 0.55, 0.58, 0.7, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, 0.4, 1.0, -1.0, -0.4, 0.0, -1.0, -0.4, 0.0, 0.4, 1.0, -0.11, 0.03, 0.65, -1.0, -0.78, -0.5775, -0.375, 0.19999999, 0.44999996, 1.0, -0.01, 0.01, 0.19999999, 0.44999996, 1.0, -0.01, 0.01, -1.0, -0.78, -0.5775, -0.375, 0.19999999, 0.44999996, 1.0, -0.19, -0.15, -0.1, 0.03, 0.06, -0.6, -0.5, -0.35, -0.25, -0.1, 0.03, 0.35, 0.45, 0.55, 0.62, -0.2, 0.2, -0.05, 0.05, -0.05, 0.05, -0.9, -0.69, 0.0, 0.1, -0.6, -0.5, -0.35, -0.25, -0.1, 0.03, 0.35, 0.45, 0.55, 0.62, -0.2, 0.2, -0.9, -0.69, 0.0, 0.1, -0.6, -0.5, -0.35, -0.25, -0.1, 0.03, 0.35, 0.45, 0.55, 0.62, -0.2, 0.2, -0.9, -0.69, 0.0, 0.1, -0.6, -0.5, -0.35, -0.25, -0.1, 0.03, 0.05, 0.4, 0.45, 0.55, 0.58, -0.2, 0.2, 0.45, 0.7, -0.7, -0.15];
pub const SPLINE_DERS: [f32; 245] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.38940096, 0.38940096, 0.37788022, 0.37788022, 0.0, 0.0, 0.0, 0.0, 0.2534563, 0.2534563, 0.5, 0.0, 0.0, 0.0, 0.007000001, 0.5, 0.0, 0.0, 0.1, 0.007000001, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.06, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.007000001, 0.5, 0.01, 0.01, 0.094000004, 0.007000001, 0.5, 0.0, 0.0, 0.04, 0.049, 0.0, 0.0, 0.0, 0.12, 0.049, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5138249, 0.5138249, 0.0, 0.43317974, 0.43317974, 0.0, 0.3917051, 0.3917051, 0.5, 0.0, 0.0, 0.0, 0.049000014, 0.5, 0.07, 0.07, 0.658, 0.049000014, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5760369, 0.5760369, 0.0, 0.4608295, 0.4608295, 0.5, 0.0, 0.0, 0.0, 0.070000015, 0.5, 0.099999994, 0.099999994, 0.94, 0.070000015, 0.5, 0.0, 0.0, 0.04, 0.049, 0.0, 0.0, 0.0, 0.015, 0.0, 0.0, 0.04, 0.049, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
pub const SPLINE_VAL_F: [f32; 245] = [-0.08880186, 0.69000006, -0.115760356, 0.6400001, -0.2222, -0.2222, 0.0, 2.9802322e-08, 2.9802322e-08, 0.100000024, -0.3, 0.05, 0.05, 0.05, 0.060000002, -0.15, 0.0, 0.0, 0.05, 0.060000002, -0.15, 0.0, 0.0, 0.0, 0.0, -0.02, -0.03, -0.03, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.25, 0.05, 0.05, 0.05, 0.060000002, -0.1, 0.001, 0.003, 0.05, 0.060000002, -0.1, 0.01, 0.01, 0.03, 0.1, -0.02, -0.03, -0.03, 0.03, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.20235021, 0.7161751, 1.23, 0.2, 0.44682026, 0.88, 0.2, 0.30829495, 0.70000005, -0.25, 0.35, 0.35, 0.35, 0.42000002, -0.1, 0.0069999998, 0.021, 0.35, 0.42000002, -0.1, 0.0, 0.17, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.34792626, 0.9239631, 1.5, 0.2, 0.5391705, 1.0, -0.2, 0.5, 0.5, 0.5, 0.6, -0.05, 0.01, 0.03, 0.5, 0.6, -0.05, 0.01, 0.01, 0.03, 0.1, -0.05, 0.0, 0.17, -0.02, 0.01, 0.01, 0.03, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.044, -0.2222, -0.2222, -0.12, -0.12, 0.0, 0.0, 0.0, 0.0, 0.0, 0.63, 0.3, 0.0, 0.0, 0.0, 0.315, 0.15, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 6.3, 6.25, 6.3, 2.67, 2.67, 6.3, 6.25, 0.625, 6.25, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 6.25, 0.0, 0.0, 6.25, 6.3, 5.47, 5.47, 0.625, 5.47, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.47, 0.0, 0.0, 5.47, 6.3, 5.08, 5.08, 0.625, 5.08, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.08, 0.0, 0.0, 5.08, 6.3, 4.69, 0.0, 1.56, 0.0, 1.37, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.69, 3.95, 0.0, 0.0, 0.0, 0.0];
pub const SPLINE_VAL_KIND: [i32; 245] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1];
pub const SPLINE_VAL_NODE: [i32; 245] = [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 8, 9, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10, -1, 13, 14, 15, 16, 17, 10, 10, 18, 18, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 24, -1, -1, -1, -1, -1, -1, 20, 21, 21, 22, 23, 24, 24, 25, 25, 24, 26, -1, -1, -1, -1, -1, 7, 7, 12, 19, 27, -1, -1, -1, -1, 29, -1, -1, -1, -1, 31, 30, 32, 32, -1, -1, 29, 29, 34, 30, 30, -1, -1, 33, 35, -1, -1, -1, -1, -1, -1, -1, -1, -1, 40, 37, 38, 37, 37, 39, 37, -1, 41, 41, -1, -1, -1, -1, -1, -1, 44, 43, 38, 43, 43, 39, 43, -1, 45, 45, -1, -1, -1, -1, -1, -1, 48, 47, 38, 47, 47, 39, 47, -1, 49, 49, -1, -1, -1, 51, -1, 51, -1, 51, 38, 51, 51, 39, 51, 52, 52, 53, 53, -1, -1, 42, 46, 50, 54];
pub const NOISE_SLOT_BASE: [i32; 25] = [0, 8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128, 136, 144, 152, 160, 168, 176, 184, 192];
pub const NOISE_SLOT_STRIDE: [i32; 25] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
pub const COORD_SLOT_TABLE: [i32; 4] = [0, 1, 2, 2];
pub const NORMAL_PACK: [i32; 600] = [9, 0, 0, 9, 18, 108, 9, 36, 216, 9, 54, 324, 9, 72, 432, 9, 90, 540, 9, 108, 648, 9, 126, 756, 5, 144, 864, 5, 154, 924, 5, 164, 984, 5, 174, 1044, 5, 184, 1104, 5, 194, 1164, 5, 204, 1224, 5, 214, 1284, 6, 224, 1344, 6, 236, 1416, 6, 248, 1488, 6, 260, 1560, 6, 272, 1632, 6, 284, 1704, 6, 296, 1776, 6, 308, 1848, 16, 320, 1920, 16, 352, 2112, 16, 384, 2304, 16, 416, 2496, 16, 448, 2688, 16, 480, 2880, 16, 512, 3072, 16, 544, 3264, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 896, 5696, 3, 902, 5732, 3, 908, 5768, 3, 914, 5804, 3, 920, 5840, 3, 926, 5876, 3, 932, 5912, 3, 938, 5948, 1, 944, 5984, 1, 946, 5996, 1, 948, 6008, 1, 950, 6020, 1, 952, 6032, 1, 954, 6044, 1, 956, 6056, 1, 958, 6068, 1, 960, 6080, 1, 962, 6092, 1, 964, 6104, 1, 966, 6116, 1, 968, 6128, 1, 970, 6140, 1, 972, 6152, 1, 974, 6164, 1, 976, 6176, 1, 978, 6188, 1, 980, 6200, 1, 982, 6212, 1, 984, 6224, 1, 986, 6236, 1, 988, 6248, 1, 990, 6260, 1, 992, 6272, 1, 994, 6284, 1, 996, 6296, 1, 998, 6308, 1, 1000, 6320, 1, 1002, 6332, 1, 1004, 6344, 1, 1006, 6356, 1, 1008, 6368, 1, 1010, 6380, 1, 1012, 6392, 1, 1014, 6404, 1, 1016, 6416, 1, 1018, 6428, 1, 1020, 6440, 1, 1022, 6452, 1, 1024, 6464, 1, 1026, 6476, 1, 1028, 6488, 1, 1030, 6500, 1, 1032, 6512, 1, 1034, 6524, 1, 1036, 6536, 1, 1038, 6548, 1, 1040, 6560, 1, 1042, 6572, 1, 1044, 6584, 1, 1046, 6596, 1, 1048, 6608, 1, 1050, 6620, 1, 1052, 6632, 1, 1054, 6644, 9, 1056, 6656, 9, 1074, 6764, 9, 1092, 6872, 9, 1110, 6980, 9, 1128, 7088, 9, 1146, 7196, 9, 1164, 7304, 9, 1182, 7412, 1, 1200, 7520, 1, 1202, 7532, 1, 1204, 7544, 1, 1206, 7556, 1, 1208, 7568, 1, 1210, 7580, 1, 1212, 7592, 1, 1214, 7604, 1, 1216, 7616, 1, 1218, 7628, 1, 1220, 7640, 1, 1222, 7652, 1, 1224, 7664, 1, 1226, 7676, 1, 1228, 7688, 1, 1230, 7700, 1, 1232, 7712, 1, 1234, 7724, 1, 1236, 7736, 1, 1238, 7748, 1, 1240, 7760, 1, 1242, 7772, 1, 1244, 7784, 1, 1246, 7796, 1, 1248, 7808, 1, 1250, 7820, 1, 1252, 7832, 1, 1254, 7844, 1, 1256, 7856, 1, 1258, 7868, 1, 1260, 7880, 1, 1262, 7892, 2, 1264, 7904, 2, 1268, 7928, 2, 1272, 7952, 2, 1276, 7976, 2, 1280, 8000, 2, 1284, 8024, 2, 1288, 8048, 2, 1292, 8072, 1, 1296, 8096, 1, 1298, 8108, 1, 1300, 8120, 1, 1302, 8132, 1, 1304, 8144, 1, 1306, 8156, 1, 1308, 8168, 1, 1310, 8180, 1, 1312, 8192, 1, 1314, 8204, 1, 1316, 8216, 1, 1318, 8228, 1, 1320, 8240, 1, 1322, 8252, 1, 1324, 8264, 1, 1326, 8276, 1, 1328, 8288, 1, 1330, 8300, 1, 1332, 8312, 1, 1334, 8324, 1, 1336, 8336, 1, 1338, 8348, 1, 1340, 8360, 1, 1342, 8372, 1, 1344, 8384, 1, 1346, 8396, 1, 1348, 8408, 1, 1350, 8420, 1, 1352, 8432, 1, 1354, 8444, 1, 1356, 8456, 1, 1358, 8468, 1, 1360, 8480, 1, 1362, 8492, 1, 1364, 8504, 1, 1366, 8516, 1, 1368, 8528, 1, 1370, 8540, 1, 1372, 8552, 1, 1374, 8564, 1, 1376, 8576, 1, 1378, 8588, 1, 1380, 8600, 1, 1382, 8612, 1, 1384, 8624, 1, 1386, 8636, 1, 1388, 8648, 1, 1390, 8660];
pub const NORMAL_PACK_F: [f32; 400] = [0.500978474, 1.5, 0.500978474, 1.5, 0.500978474, 1.5, 0.500978474, 1.5, 0.500978474, 1.5, 0.500978474, 1.5, 0.500978474, 1.5, 0.500978474, 1.5, 0.516129032, 1.38888889, 0.516129032, 1.38888889, 0.516129032, 1.38888889, 0.516129032, 1.38888889, 0.516129032, 1.38888889, 0.516129032, 1.38888889, 0.516129032, 1.38888889, 0.516129032, 1.38888889, 0.507936508, 1.25, 0.507936508, 1.25, 0.507936508, 1.25, 0.507936508, 1.25, 0.507936508, 1.25, 0.507936508, 1.25, 0.507936508, 1.25, 0.507936508, 1.25, 0.50000763, 1.56862745, 0.50000763, 1.56862745, 0.50000763, 1.56862745, 0.50000763, 1.56862745, 0.50000763, 1.56862745, 0.50000763, 1.56862745, 0.50000763, 1.56862745, 0.50000763, 1.56862745, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.571428571, 1.25, 0.571428571, 1.25, 0.571428571, 1.25, 0.571428571, 1.25, 0.571428571, 1.25, 0.571428571, 1.25, 0.571428571, 1.25, 0.571428571, 1.25, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 0.500978474, 1.48148148, 0.500978474, 1.48148148, 0.500978474, 1.48148148, 0.500978474, 1.48148148, 0.500978474, 1.48148148, 0.500978474, 1.48148148, 0.500978474, 1.48148148, 0.500978474, 1.48148148, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 0.666666667, 1.11111111, 0.666666667, 1.11111111, 0.666666667, 1.11111111, 0.666666667, 1.11111111, 0.666666667, 1.11111111, 0.666666667, 1.11111111, 0.666666667, 1.11111111, 0.666666667, 1.11111111, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333, 1.0, 0.833333333];
pub const NORMAL_AMPS: [f32; 536] = [1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.4, 0.5, 1.0, 0.4, 0.5, 1.0, 0.4, 0.5, 1.0, 0.4, 0.5, 1.0, 0.4, 0.5, 1.0, 0.4, 0.5, 1.0, 0.4, 0.5, 1.0, 0.4, 0.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0, 0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0, 0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0, 0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0, 0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0, 0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0, 0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0, 0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
pub const NORMAL_AMP_OFF: [i32; 200] = [0, 9, 18, 27, 36, 45, 54, 63, 72, 77, 82, 87, 92, 97, 102, 107, 112, 118, 124, 130, 136, 142, 148, 154, 160, 176, 192, 208, 224, 240, 256, 272, 288, 288, 288, 288, 288, 288, 288, 288, 288, 291, 294, 297, 300, 303, 306, 309, 312, 313, 314, 315, 316, 317, 318, 319, 320, 321, 322, 323, 324, 325, 326, 327, 328, 329, 330, 331, 332, 333, 334, 335, 336, 337, 338, 339, 340, 341, 342, 343, 344, 345, 346, 347, 348, 349, 350, 351, 352, 353, 354, 355, 356, 357, 358, 359, 360, 361, 362, 363, 364, 365, 366, 367, 368, 377, 386, 395, 404, 413, 422, 431, 440, 441, 442, 443, 444, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 455, 456, 457, 458, 459, 460, 461, 462, 463, 464, 465, 466, 467, 468, 469, 470, 471, 472, 474, 476, 478, 480, 482, 484, 486, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 500, 501, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511, 512, 513, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529, 530, 531, 532, 533, 534, 535];
pub const OLD_PACK: [i32; 400] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 576, 3456, 616, 3736, 656, 4016, 696, 4296, 736, 4576, 776, 4856, 816, 5136, 856, 5416, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub fn spline_coord_fold(coord_type: usize, v: f32) -> f32 {
    match coord_type {
        0 => (v),
        1 => (v),
        2 => (-3.0 * (-0.3333333333333333 + f32::abs((-0.6666666666666666 + f32::abs((v)))))),
        3 => (v),
        _ => v,
    }
}

