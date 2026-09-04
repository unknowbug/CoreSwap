// fillmap.rs — 用 terrain::fill_chunk 生成 spawn 附近宏观地形图（表面高度 + biome + 水/岩）。
// 验证：端到端 fill 管线（density+aquifer+biome）能产出可辨识的地形（山/湖/平原）+ biome。
// 宏观验收：山高、湖低、biome 对——不追 block id。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::aquifer::Aquifer;
use WorldgenRust::terrain::{fill_chunk, VanillaDensity, VanillaAquifer};
use WorldgenRust::biome::BiomeClassifier;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

struct MacroBiome { bc: BiomeClassifier, tempf: Arc<DensityFunction>, humf: Arc<DensityFunction>, contf: Arc<DensityFunction>, erof: Arc<DensityFunction>, depthf: Arc<DensityFunction>, weirdf: Arc<DensityFunction> }
impl WorldgenRust::terrain::BiomeSource for MacroBiome {
    fn biome(&self, pos: &NoisePos) -> String {
        self.bc.biome_of(&self.tempf, &self.humf, &self.contf, &self.erof, &self.depthf, &self.weirdf, pos)
    }
}

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
    let barrier: Arc<DensityFunction> = Arc::new(db.build_node(router.get("barrier").unwrap()).unwrap());
    let flooded: Arc<DensityFunction> = Arc::new(db.build_node(router.get("fluid_level_floodedness").unwrap()).unwrap());
    let spread: Arc<DensityFunction> = Arc::new(db.build_node(router.get("fluid_level_spread").unwrap()).unwrap());
    let lava: Arc<DensityFunction> = Arc::new(db.build_node(router.get("lava").unwrap()).unwrap());
    let erosion: Arc<DensityFunction> = Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap());
    let depth: Arc<DensityFunction> = Arc::new(db.build_node(router.get("depth").unwrap()).unwrap());
    let init: Arc<DensityFunction> = Arc::new(db.build_node(router.get("initial_density_without_jaggedness").unwrap()).unwrap());
    let splitter = db.random_deriver().split_str("minecraft:aquifer").next_splitter();

    let dense = VanillaDensity { df: &tree };
    let bc = BiomeClassifier::load("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\biome_params.json");
    let biomesrc = MacroBiome {
        bc,
        tempf: Arc::new(db.build_node(router.get("temperature").unwrap()).unwrap()),
        humf: Arc::new(db.build_node(router.get("vegetation").unwrap()).unwrap()),
        contf: Arc::new(db.build_node(router.get("continents").unwrap()).unwrap()),
        erof: Arc::new(db.build_node(router.get("erosion").unwrap()).unwrap()),
        depthf: Arc::new(db.build_node(router.get("depth").unwrap()).unwrap()),
        weirdf: Arc::new(db.build_node(router.get("ridges").unwrap()).unwrap()),
    };
    // 生成 spawn (-96,-48) 附近 4x4 chunk 宏观地形图
    // 每 chunk 用 surface_height 抽 4 个角点 → ASCII
    let mut rows: Vec<String> = Vec::new();
    for cz in (-6..=4).rev() {
        let mut row = String::new();
        for cx in -7..=3 {
            let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, -64, 384);
            let mut vaaq = VanillaAquifer::new(aq);
            let cd = fill_chunk(&dense, &mut vaaq, &biomesrc, cx, cz, -64, 384, None, 384);
            // 宏观地形：每 chunk 取表面高度平均（湖面=water 会低）
            let mut hs=0i64; let mut cnt=0i64; let mut water=0i64;
            for lz in (0..16).step_by(4) { for lx in (0..16).step_by(4) {
                let y=cd.surface_height[(lz*16+lx) as usize];
                if y!=i32::MIN { hs+=y as i64; cnt+=1; if y<63 { water+=1; } }
            }}
            let ch = if cnt==0 { '.' }
                else { let avg=hs/cnt; if water as i64 > (cnt/2) { '~' } // 多水=湖
                    else if avg>105 { '^' } else if avg>85 { 'h' } else if avg>60 { 'p' } else { 'l' } };
            row.push(ch);
        }
        rows.push(row);
    }
    println!("seed={} spawn区(-96,-48) 宏观地形: legend ^山 h丘 p平原 l低地 ~湖 .无solid", seed);
    println!("   x: -7..3");
    for r in &rows { println!("   {}", r); }
    // biome map (per chunk 多数 biome)
    println!("--- 宏观 biome map (per chunk majority) ---");
    let mut bros: Vec<String> = Vec::new();
    for cz in (-6..=4).rev() {
        let mut row = String::new();
        for cx in -7..=3 {
            let mut aq = Aquifer::new(barrier.clone(), flooded.clone(), spread.clone(), lava.clone(), erosion.clone(), depth.clone(), init.clone(), splitter.clone(), cx*16, cz*16, -64, 384);
            let mut vaaq = VanillaAquifer::new(aq);
            let cd = fill_chunk(&dense, &mut vaaq, &biomesrc, cx, cz, -64, 384, None, 384);
            // 取该 chunk biome 标签（多数列）
            let mut map: std::collections::HashMap<String,u32> = std::collections::HashMap::new();
            for i in 0..256 { *map.entry(cd.biome[i].clone()).or_insert(0) += 1; }
            if let Some((b,_)) = map.iter().max_by_key(|(_,c)| **c) {
                row.push(match b.as_str() {
                    "minecraft:cherry_grove" => 'C', "minecraft:plains" => 'p', "minecraft:sunflower_plains" => 'S',
                    "minecraft:forest" => 'f', "minecraft:birch_forest" => 'b', "minecraft:meadow" => 'm',
                    "minecraft:grove" | "minecraft:snowy_slopes" => 'h', "minecraft:frozen_ocean" | "minecraft:cold_ocean" | "minecraft:ocean" | "minecraft:deep_ocean" => '~',
                    _ => '?',
                });
            } else { row.push('.'); }
        }
        bros.push(row);
    }
    for r in &bros { println!("   {}", r); }
    // 调试: 已知 cherry 点 (64,-176) 的 6 params + biome_of（在列表面 top 采样，非固定 y）
    for &(dx,dz) in &[(64,-176),(96,-144),(0,0),(200,60)] {
        // 找地表 top
        let mut top=-64i32;
        for y in (40..260).rev(){ if tree.sample(&NoisePos{x:dx,y,z:dz})>0.0 { top=y; break; } }
        let bp = NoisePos{x:(dx>>2)<<2, y:(top>>2)<<2, z:(dz>>2)<<2};
        let t=biomesrc.tempf.sample(&bp); let h=biomesrc.humf.sample(&bp); let c=biomesrc.contf.sample(&bp);
        let e=biomesrc.erof.sample(&bp); let d=biomesrc.depthf.sample(&bp); let w=biomesrc.weirdf.sample(&bp);
        let b = biomesrc.bc.biome_of(&biomesrc.tempf, &biomesrc.humf, &biomesrc.contf, &biomesrc.erof, &biomesrc.depthf, &biomesrc.weirdf, &bp);
        println!("({},{}) atFlooredTop(y={}) temp={:.3} hum={:.3} cont={:.3} ero={:.3} dep={:.3} weird={:.3} -> {}", dx,dz,bp.y,t,h,c,e,d,w,b);
    }
}

