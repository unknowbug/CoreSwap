// rvv_surface.rs — Rust(finalDensity+Aquifer+DeepSurface) vs vanilla 块，验证深带替换收益。
// 每块：d>0 → STONE 再 apply_deep_rules（bedrock/deepslate）；d<=0 → aquifer(air/water/lava)。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::aquifer::Aquifer;
use WorldgenRust::ore_vein::OreVeinSampler;
use WorldgenRust::surface::apply_deep_rules;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn be16(b: &[u8], i: &mut usize) -> u16 { let v = u16::from_be_bytes(b[*i..*i+2].try_into().unwrap()); *i += 2; v }
fn be32(b: &[u8], i: &mut usize) -> i32 { let v = i32::from_be_bytes(b[*i..*i+4].try_into().unwrap()); *i += 4; v }
fn be64(b: &[u8], i: &mut usize) -> i64 { let v = i64::from_be_bytes(b[*i..*i+8].try_into().unwrap()); *i += 8; v }

const AIR: i32 = 0; const STONE: i32 = 1; const WATER: i32 = 32; const LAVA: i32 = 33;

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
    let tree: Arc<DensityFunction> = Arc::new(db.build_node(router.get("final_density").unwrap()).unwrap());
    let mut b = |k: &str| -> Arc<DensityFunction> { Arc::new(db.build_node(router.get(k).unwrap()).unwrap()) };
    let barrier = b("barrier"); let flooded = b("fluid_level_floodedness"); let spread = b("fluid_level_spread");
    let lava = b("lava"); let erosion = b("erosion"); let depth = b("depth"); let init = b("initial_density_without_jaggedness");
    let (vein_toggle, vein_ridged, vein_gap) = (b("vein_toggle"), b("vein_ridged"), b("vein_gap"));
    let splitter = db.random_deriver().split_str("minecraft:aquifer").next_splitter();
    let vein_splitter = db.random_deriver().split_str("minecraft:ore").next_splitter();
    let mut ore = OreVeinSampler::new(vein_toggle, vein_ridged, vein_gap, vein_splitter);

    let path = "E:\\PYTHON\\MC\\data\\vanilla_-2032795982907864146_4_0_0.blocks";
    let bd = fs::read(path).unwrap();
    let mut i = 0usize;
    let magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let size = be32(&bd, &mut i);
    let origin_x = be32(&bd, &mut i); let origin_z = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("magic=0x{:X} seed={} size={} origin=({},{}) minY={} height={}", magic, vseed, size, origin_x, origin_z, min_y, height);
    let bpc = 16*16*height as usize;
    let mut total = 0u64; let mut match_t = 0u64; let mut tnair = 0u64; let mut mnair = 0u64;
    for _c in 0..(size*size) {
        let cx = be32(&bd, &mut i); let cz = be32(&bd, &mut i);
        let mut vanilla = vec![0i32; bpc];
        for k in 0..bpc { vanilla[k] = be16(&bd, &mut i) as i32; }
        for _bi in 0..256 { let bl = be16(&bd, &mut i) as usize; if bl>0 { i += bl; } } // biome 段
        let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, min_y, height);
        for k in 0..bpc {
            let lx = (k % 16) as i32; let ly = (k / 256) as i32; let lz = ((k / 16) % 16) as i32;
            let x = cx*16 + lx; let y = min_y + ly; let z = cz*16 + lz;
            let d = tree.sample(&NoisePos{x, y, z});
            let mut got = if d > 0.0 { STONE } else { let blk = aq.apply(x, y, z, d); match blk { 1 => WATER, 2 => LAVA, _ => AIR } };
            if got == STONE { got = apply_deep_rules(got, true, y, min_y); } // solid stone → deep-band rule
            // 矿脉替换：deepslate/stone → tuff/ore（y 在 [-60,50]）
            let ov = ore.apply(x, y, z);
            if ov >= 0 { got = ov; }
            total += 1;
            if vanilla[k] != 0 { tnair += 1; }
            if got == vanilla[k] { match_t += 1; if vanilla[k] != 0 { mnair += 1; } }
        }
    }
    println!("Rust(+deepSurface) vs vanilla: match={}/{} ({:.2}%)  nonAir={}/{} ({:.2}%)", match_t, total, 100.0*match_t as f64/total as f64, mnair, tnair, if tnair>0 {100.0*mnair as f64/tnair as f64} else {0.0});
    println!("(baseline stone-only = 91.17% / 73.55%)");
}
