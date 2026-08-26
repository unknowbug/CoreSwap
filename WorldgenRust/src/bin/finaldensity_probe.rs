// finaldensity_probe.rs — 纯 Rust：读 overworld.json noise_router.final_density → buildNode 整棵最终密度树 → 采样
// 验证 buildNode 对完整 worldgen finalDensity 链（big min/squeeze/interpolated/blend_density/range_choice + caves refs）的贯通。
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::NoisePos;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn main() {
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    let settings_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json";
    let noise_params_path = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json";

    let mut db = DensityBuilder::new(8576294172403134396, -64, 384);
    // 用权威 noise_params.json 覆盖硬编码表（judge P2-e：对齐基准切到文件）
    db.load_noise_params_file(noise_params_path).expect("load noise_params.json");
    db.set_external_loader(Box::new(move |_full: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {} -> {}", p.display(), e))
    }));

    // 解析 overworld.json → noise_router.final_density
    let settings = parse(&fs::read_to_string(settings_path).unwrap()).unwrap();
    let router = settings.get("noise_router").expect("noise_router");
    let fd = router.get("final_density").expect("final_density");
    let tree = db.build_node(fd).expect("build final_density");

    // 采样一组覆盖不同 y/xz 的点 + min/max
    let pts = [(0, 0, 0), (8, 64, 8), (100, -64, -40), (4, 120, 4), (-64, 320, -64),
               (200, 40, 200), (16, -112, 16), (72, 240, 72), (-200, 96, 96), (0, 200, -16)];
    let mut line = String::from("final_density:");
    for (x, y, z) in pts {
        let v = tree.sample(&NoisePos { x, y, z });
        line.push_str(&format!(" ({},{},{})={:.8}", x, y, z, v));
    }
    line.push_str(&format!("  min={:.8} max={:.8}", tree.min_value(), tree.max_value()));
    println!("{}", line);
}
