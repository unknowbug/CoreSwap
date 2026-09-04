// overworld_probe.rs — 用真实 overworld JSON 验证 buildNode 端到端（external_loader 读盘 + resolve_ref 递归 + 采样）
// 目的：catch buildNode 对各真实密度函数文件（含 spline/old_blended_noise/shifted_noise/registry 引用）的运行时 bug。
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::NoisePos;
use std::fs;
use std::path::PathBuf;

fn main() {
    let base = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    let mut db = DensityBuilder::new(8576294172403134396, -64, 384);
    db.set_external_loader(Box::new(move |_full: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", base, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {} -> {}", p.display(), e))
    }));

    let names = ["base_3d_noise", "continents", "erosion", "ridges", "ridges_folded",
                 "factor", "offset", "jaggedness", "depth", "sloped_cheese",
                 "caves/entrances", "caves/noodle", "caves/pillars",
                 "caves/spaghetti_2d", "caves/spaghetti_2d_thickness_modulator",
                 "caves/spaghetti_roughness_function"];
    let pts = [(0, 0, 0), (4, 64, 4), (8, 128, 8), (40, 192, 40), (100, -64, -40),
               (-64, 64, -64), (128, 288, 128), (200, 0, 200), (16, -112, 16), (72, 320, 72)];
    for name in names {
        let key = format!("minecraft:overworld/{}", name);
        let df = db.resolve_ref(&key);
        let mut line = format!("{}:", name);
        for (x, y, z) in pts {
            let v = df.sample(&NoisePos { x, y, z });
            line.push_str(&format!(" ({},{},{})={:.8}", x, y, z, v));
        }
        line.push_str(&format!("  min={:.4} max={:.4}", df.min_value(), df.max_value()));
        println!("{}", line);
    }
    // 顶层 sloped_cheese 采样 min/max（== finalDensity 主体，供后续对拍）
    let df = db.resolve_ref("minecraft:overworld/sloped_cheese");
    println!("sloped_cheese min={:.6} max={:.6}", df.min_value(), df.max_value());
}
