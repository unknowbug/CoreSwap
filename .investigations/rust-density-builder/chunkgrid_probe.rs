// chunkgrid_probe.rs — Rust 整块密度网格填充：对 chunk(45,-26) 全部 16x16 列 × 10 代表 y 采样 finalDensity，对比当前 C++ 网格参照。
// 参照：cpp_grid45.txt（rust_ref_check 输出 `GRID x z y val`）
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::NoisePos;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn main() {
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    let settings_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json";
    let noise_params_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json";
    let ref_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\cpp\\build-msvc\\bin\\cpp_grid45.txt";

    let mut db = DensityBuilder::new(8576294172403134396, -64, 384);
    db.load_noise_params_file(noise_params_path).expect("load noise_params.json");
    db.set_external_loader(Box::new(move |_full: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {} -> {}", p.display(), e))
    }));

    let settings = parse(&fs::read_to_string(settings_path).unwrap()).unwrap();
    let fd = settings.get("noise_router").and_then(|r| r.get("final_density")).expect("final_density");
    let tree = db.build_node(fd).expect("build final_density");

    // 读参照 `GRID x z y val`
    let mut refs: std::collections::HashMap<(i32, i32, i32), f64> = std::collections::HashMap::new();
    for line in fs::read_to_string(ref_path).unwrap().lines() {
        let mut it = line.split_whitespace();
        if it.next().unwrap_or("") == "GRID" {
            if let (Some(x), Some(z), Some(y), Some(v)) = (it.next(), it.next(), it.next(), it.next()) {
                if let (Ok(x), Ok(z), Ok(y), Ok(v)) = (x.parse::<i32>(), z.parse::<i32>(), y.parse::<i32>(), v.parse::<f64>()) {
                    refs.insert((x, z, y), v);
                }
            }
        }
    }

    let ys = [-64, -32, 0, 32, 63, 96, 128, 200, 256, 319];
    let (cx, cz) = (45, -26);
    let mut matched = 0usize; let mut total = 0usize; let mut max_diff = 0.0f64; let mut worst = (0i32, 0i32, 0i32, 0.0f64, 0.0f64);
    for bx in 0..16 { for bz in 0..16 {
        let (x, z) = (cx*16 + bx, cz*16 + bz);
        for &y in &ys {
            let got = tree.sample(&NoisePos { x, y, z });
            if let Some(&rv) = refs.get(&(x, z, y)) {
                total += 1;
                let d = (got - rv).abs();
                if d < 1e-6 { matched += 1; }
                if d > max_diff { max_diff = d; worst = (x, y, z, rv, got); }
            }
        }
    }}
    println!("chunk(45,-26) full grid fill: {} points", total);
    println!("matched(<1e-6)={}/{}  maxDiff={:.3e}  @(x={},y={},z={}) ref={} got={}", matched, total, max_diff, worst.0, worst.1, worst.2, worst.3, worst.4);
}
