// aquifer_probe.rs — 验证 Aquifer 移植：finalDensity(cherry seed) + Aquifer 生成 chunk(0,0) 块分类（石头/水/空气）。
// 对比旧 sea-level 规则（density<=0 && y<63 = 水）——Aquifer 应正确把地下负密度分为洞穴空气 vs 含水层水/溪湖。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::aquifer::{Aquifer, WATER, AIR, LAVA};
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn main() {
    let seed: i64 = -2032795982907864146;
    let mut db = DensityBuilder::new(seed as u64, -64, 384);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}", p.display()))
    }));
    let settings = parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let fd = router.get("final_density").unwrap();
    let tree: Arc<DensityFunction> = Arc::new(db.build_node(fd).unwrap());
    // router 分量（Aquifer 需要）
    let mut b = |k: &str| -> Arc<DensityFunction> { Arc::new(db.build_node(router.get(k).unwrap()).unwrap()) };
    let barrier = b("barrier"); let flooded = b("fluid_level_floodedness"); let spread = b("fluid_level_spread");
    let lava = b("lava"); let erosion = b("erosion"); let depth = b("depth"); let init = b("initial_density_without_jaggedness");
    let splitter = db.random_deriver().split_str("minecraft:aquifer").next_splitter();

    let chunk = (0, 0);
    let mut aq = Aquifer::new(barrier, flooded, spread, lava, erosion, depth, init, splitter, chunk.0*16, chunk.1*16, -64, 384);
    // 三层的 16x16 块分类切片
    for &ly in &[-40i32, 0, 40] {
        println!("--- y={} (#=stone ~=water .=air) ---", ly);
        for bz in 0..16 { let mut row = String::new();
            for bx in 0..16 {
                let x = chunk.0*16+bx; let z = chunk.1*16+bz;
                let d = tree.sample(&NoisePos{x, y: ly, z});
                let blk = aq.apply(x, ly, z, d);
                let c = if d > 0.0 { '#' } else { match blk { WATER => '~', LAVA => '!', _ => '.' } };
                row.push(c);
            }
            println!("  {}", row);
        }
    }
    // 统计
    let (mut st, mut wa, mut ai) = (0u64, 0u64, 0u64);
    for bz in 0..16 { for bx in 0..16 { let x=chunk.0*16+bx; let z=chunk.1*16+bz;
        for y in (-64..320).step_by(4) { let d=tree.sample(&NoisePos{x,y,z});
            if d>0.0 { st+=1; } else { let blk=aq.apply(x,y,z,d); if blk==WATER {wa+=1;} else if blk==AIR {ai+=1;} else {} }
        } } }
    println!("chunk(0,0) stone={} water={} air={} (y step4)", st, wa, ai);
}
