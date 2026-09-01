// gapdecomp.rs — 正确种子(-2032)：逐块 Rust vs vanilla 分类，按 (vanilla_type, rust_type) 桶 + surface Y 带分布，
// 精确定位 surface 层缺什么（deepslate/tuff/草方块/gravel/bedrock）。Rust 块管线 = d>0→STONE(1), d<=0→aquifer(AIR/WATER/LAVA)。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::aquifer::Aquifer;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

fn be16(b: &[u8], i: &mut usize) -> u16 { let v = u16::from_be_bytes(b[*i..*i+2].try_into().unwrap()); *i += 2; v }
fn be32(b: &[u8], i: &mut usize) -> i32 { let v = i32::from_be_bytes(b[*i..*i+4].try_into().unwrap()); *i += 4; v }
fn be64(b: &[u8], i: &mut usize) -> i64 { let v = i64::from_be_bytes(b[*i..*i+8].try_into().unwrap()); *i += 8; v }

const AIR: i32 = 0; const STONE: i32 = 1;
fn rust_type_name(id: i32) -> &'static str {
    match id { 0 => "air", 1 => "stone", 32 => "water", 33 => "lava", _ => "?" }
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
    let mut b = |k: &str| -> Arc<DensityFunction> { Arc::new(db.build_node(router.get(k).unwrap()).unwrap()) };
    let barrier = b("barrier"); let flooded = b("fluid_level_floodedness"); let spread = b("fluid_level_spread");
    let lava = b("lava"); let erosion = b("erosion"); let depth = b("depth"); let init = b("initial_density_without_jaggedness");
    let splitter = db.random_deriver().split_str("minecraft:aquifer").next_splitter();

    // vanilla block id -> name (加载 blocks.json)
    let bj = parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\java\\src\\main\\resources\\worldgen-data\\blocks.json").unwrap()).unwrap();
    let mut id2name: HashMap<i32,String> = HashMap::new();
    if let Some(o)=bj.as_object(){ for (k,v) in o { if let Some(n)=v.as_f64(){ id2name.insert(n as i32, k.clone()); } } }
    let nameof = |id:i32| -> String { id2name.get(&id).map(|s| s.replace("minecraft:","")).unwrap_or_else(|| format!("id{}",id)) };

    let bd = fs::read("E:\\PYTHON\\MC\\data\\vanilla_-2032795982907864146_4_0_0.blocks").unwrap();
    let mut i = 0usize;
    let _magic=be32(&bd,&mut i); let _seed=be64(&bd,&mut i); let size=be32(&bd,&mut i);
    let _ox=be32(&bd,&mut i); let _oz=be32(&bd,&mut i); let miny=be32(&bd,&mut i); let height=be32(&bd,&mut i);
    let bpc=(16*16*height) as usize;
    // 桶统计：vanilla类别 -> (rust类别 -> count)，按 surface-y 带分三档（<0 deep, 0-62 subsurface, >=63 surface/air + 湖面）
    let mut buckets: HashMap<String, HashMap<String, (u64,u64)>> = HashMap::new(); // vtype -> rtype -> (band<0, band>=63)? 用两键
    // 简化：vtype -> (negY count, posY count) per rusttype
    let mut stats: HashMap<String, HashMap<String,(u64,u64)>> = HashMap::new();
    let mut rust_surface_mismatch=0u64; let mut deepslate_mismatch=0u64;
    for _c in 0..(size*size){
        let cx=be32(&bd,&mut i); let cz=be32(&bd,&mut i);
        let mut vanilla=vec![0i32; bpc];
        for k in 0..bpc { vanilla[k]=be16(&bd,&mut i) as i32; }
        // biome 段跳过（16x16 UTF）
        { let mut j=i; for _z in 0..16 { for _x in 0..16 { let ln=be16(&bd,&mut j) as usize; j+=ln; } } i=j; }
        let mut aq=Aquifer::new(barrier.clone(),flooded.clone(),spread.clone(),lava.clone(),erosion.clone(),depth.clone(),init.clone(),splitter.clone(),cx*16,cz*16,miny,height);
        for ly in 0..height {
            let yy=miny+ly;
            for lz in 0..16 { for lx in 0..16 {
                let x=cx*16+lx; let z=cz*16+lz; let y=yy;
                let d=tree.sample(&NoisePos{x,y,z});
                let rid = if d>0.0 {STONE} else { let blk=aq.apply(x,y,z,d); match blk {1=>32,2=>33,_=>0} };
                let vid=vanilla[(lx+lz*16+ly*256) as usize];
                if rid==vid {continue;}
                let vn=nameof(vid); let rn=rust_type_name(rid);
                let band = if yy<0 {0u64} else {1u64}; // 0=deep,1=surf/air
                let e=stats.entry(vn.clone()).or_default().entry(rn.to_string()).or_insert((0,0));
                if band==0 {e.0+=1;} else {e.1+=1;}
                if vn=="deepslate" || vn=="tuff" { deepslate_mismatch+=1; }
                if yy>=55 && yy<=130 { rust_surface_mismatch+=1; }
            }}
        }
    }
    println!("=== mismatch by (vanilla_type -> rust_type): (deepY<0, surfY>=0) ===");
    let mut vtypes: Vec<(String, u64,u64)> = Vec::new();
    for (vn, rtmap) in &stats {
        let mut t1=0u64; let mut t2=0u64;
        let mut detail=String::new();
        for (rn,c) in rtmap { t1+=c.0; t2+=c.1; detail.push_str(&format!(" {}={}/{}", rn, c.0, c.1)); }
        vtypes.push((vn.clone(), t1, t2));
        if t1+t2>0 { println!("{:<16}:{}", vn, detail); }
    }
    vtypes.sort_by(|a,b| (b.1+b.2).cmp(&(a.1+a.2)));
    println!("--- top mismatched vanilla types (total) ---");
    for (vn,t1,t2) in &vtypes { println!("  {:<24} total={} (deep={} surf={})", vn, t1+t2, t1, t2); }
    println!("deepslate/tuff mismatch total = {}", deepslate_mismatch);
    println!("surface-band(y55-130) mismatch total = {}", rust_surface_mismatch);
}
