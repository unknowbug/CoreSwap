// chunkfill_probe.rs — Rust 块级 y-column 填充引擎验证：sampling finalDensity 整列，对比已知 seed 的 C++ 密度参照。
// 参照：cpp_density_8576_45_-26_b8_8.txt（C++ finalDensity，chunk(45,-26) row(8,8)，x=728 z=-408，y=319..-64 step1）
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::NoisePos;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn main() {
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    let settings_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json";
    let noise_params_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json";
    let ref_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\cpp\\build-msvc\\bin\\cpp_col728.txt";

    let mut db = DensityBuilder::new(8576294172403134396, -64, 384);
    db.load_noise_params_file(noise_params_path).expect("load noise_params.json");
    db.set_external_loader(Box::new(move |_full: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {} -> {}", p.display(), e))
    }));

    let settings = parse(&fs::read_to_string(settings_path).unwrap()).unwrap();
    let fd = settings.get("noise_router").and_then(|r| r.get("final_density")).expect("final_density");
    let tree = db.build_node(fd).expect("build final_density");

    // 参照列：(x,z) = chunk(45,-26) block(8,8) => (728, -408)；读当前 C++ 参照(COL y val)
    let mut refs: std::collections::HashMap<i32, f64> = std::collections::HashMap::new();
    for line in fs::read_to_string(ref_path).unwrap().lines() {
        let mut it = line.split_whitespace();
        if (it.next().unwrap_or("") == "COL") {
            if let (Some(y), Some(v)) = (it.next(), it.next()) {
                if let (Ok(y), Ok(v)) = (y.parse::<i32>(), v.parse::<f64>()) {
                    refs.insert(y, v);
                }
            }
        }
    }

    let (x, z) = (45 * 16 + 8, -26 * 16 + 8);
    let mut matched = 0usize; let mut total = 0usize; let mut max_diff = 0.0f64; let mut worst = (0, 0.0, 0.0);
    for (y, rv) in &refs {
        let got = tree.sample(&NoisePos { x, y: *y, z });
        total += 1;
        let d = (got - rv).abs();
        if d < 1e-6 { matched += 1; }
        if d > max_diff { max_diff = d; worst = (*y, *rv, got); }
    }
    println!("x={} z={} column fill: {} points", x, z, total);
    println!("matched(<1e-6)={}/{}  maxDiff={:.3e}  @y={} ref={} got={}", matched, total, max_diff, worst.0, worst.1, worst.2);
}
