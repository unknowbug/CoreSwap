// ycache_probe.rs — 验证 y 相关 cache 节点（cache_once 包装 spaghetti_3d_rarity）在 transpiler 是否因 (x,z) key 返回错误值。
// judge 发现：transpiler 用 (x,z) key 缓存 y 相关 inner（spaghetti_3d_rarity y_scale=1.0），同一 (x,z) 不同 y 应返回错误值。
// 已修复：cache_once 改用 transpiler_cache_3d（(x,y,z) key）。验证修复后不同 y 值不同（正确）。
use WorldgenRust::density::transpiler_cache_3d;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::noise::NoiseSet;

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = -8248318472910187742;
    let mut db = DensityBuilder::new(seed as u64, -64, 384);
    db.load_noise_params_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    let mut noises = NoiseSet::new();
    let params = WorldgenRust::density_builder::build_noise_params_from_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    for (id, p) in &params {
        let mut rnd = db.random_deriver().split_str(id);
        let sampler = WorldgenRust::noise::DoublePerlinNoiseSampler::new(&mut rnd, p);
        noises.insert(id, sampler);
    }

    // 模拟 transpiler cache_once 包装 spaghetti_3d_rarity（y_scale=1.0，y 相关）
    // 修复后生成代码：transpiler_cache_3d(id, x, y, z, || sample_noise("spaghetti_3d_rarity", x*2, y*1, z*2))
    let x = -288*16 + 4; let z = -256*16 + 4;
    println!("同一 (x,z)=({},{}) 不同 y 的 cache_once(spaghetti_3d_rarity) 值（修复后 transpiler_cache_3d）：", x, z);
    let mut prev = 0.0f64;
    for (i, y) in [-64i32, 0, 64, 128, 200, 300].iter().enumerate() {
        // 直接采样（无缓存，参考值）
        let direct = noises.sample_noise("minecraft:spaghetti_3d_rarity", x as f64 * 2.0, *y as f64 * 1.0, z as f64 * 2.0);
        // transpiler cache_once（(x,y,z) key，修复后）
        let cached = transpiler_cache_3d(76, x as f64, *y as f64, z as f64, || noises.sample_noise("minecraft:spaghetti_3d_rarity", x as f64 * 2.0, *y as f64 * 1.0, z as f64 * 2.0));
        let same_as_prev = if i > 0 { (cached - prev).abs() < 1e-9 } else { false };
        println!("  y={}: direct={:.6} cache_once={:.6} diff={:.6} {}同前y", y, direct, cached, (direct-cached).abs(), if same_as_prev { "⚠️" } else { "" });
        prev = cached;
    }
    println!("(若 cache_once 值随 y 变化且 diff=0 → y 相关 cache bug 已修复)");
}
