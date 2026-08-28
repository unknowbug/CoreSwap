// WorldgenRust — CoreSwap worldgen Rust 重写（Phase 1 骨架）
// 目标：把 C++ worldgen（density/spline/terrain，逆向 Java）移植为 Rust（enum 数据驱动 + 软流 MLP）。
pub mod noise;             // DoublePerlinNoiseSampler / Octave / Perlin + perm
pub mod density;           // DF 树（InterpolatedDF/SplineDF/FlatCacheDF/Cache2DDF/NoiseDF/...）
pub mod density_builder;   // worldgen JSON -> DF 树
pub mod aquifer;           // 块级含水层（AquiferSampler.Impl 移植）——密度<0 区的 lava/水/空洞
pub mod surface;           // SurfaceBuilder/MaterialRules 深带规则（bedrock/deepslate/tuff）——v1 深带替换
pub mod surface_rules;     // vanilla surface rules 完整移植（条件/规则/规则树/buildSurface 引擎）
pub mod ore_vein;          // OreVeinSampler 矿脉（铜/铁，含 tuff/deepslate_*_ore/raw_*）
pub mod biome;             // 宏观 biome 分类（MultiNoise 近似：biome_params 盒包含）
pub mod blocks;             // 方块 ID 注册表 + 区块方块存储（BlockColumn）
pub mod beardifier;         // StructureWeightSampler（Beardifier）结构密度修正（对齐 C++ beardifier.h）
pub mod spline;            // SplineDF（data-driven 表 + 采样，可软流）
pub mod terrain;           // finalDensity 构建 + fill 逻辑
pub mod api;               // wg_create/fill/sample 等价（C ABI 导出）
pub mod json;              // worldgen JSON 解析
pub mod verif;             // 验证：ref 数据加载 + 逐位对比
pub mod md5;               // MD5（create_xoroshiro_seed_str 的 string 种子）
pub mod xoroshiro;         // XoroshiroRandom + Xoroshiro128PlusPlus + Splitter（随机源）
pub mod chunkrandom;       // ChunkRandom + CheckedRandom（CARVER 种子派生）
pub mod carver;            // CARVERS 阶段（洞穴雕刻）CaveCarver/RavineCarver
pub mod worldgen_handle;   // 生产句柄（C ABI 的 Rust 侧实现）

pub fn version() -> &'static str { "0.1.0" }
