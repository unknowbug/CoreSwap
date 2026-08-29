// tree_vs_noise_breakdown.rs — 分离 ch#0 采样成本：树遍历(match/递归) vs noise 采样。
// 方法：把 ch#0 树里所有 noise 节点替换为 Constant(0)，测采样成本。若成本大降 → noise 是大头；若几乎不变 → 树遍历是大头。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density::{DensityFunction, NoisePos, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;

// 递归替换 noise 节点为 Constant(0)
fn strip_noise(df: &DensityFunction) -> DensityFunction {
    match df {
        DensityFunction::Noise{..} | DensityFunction::ShiftDF{..} | DensityFunction::InterpolatedNoise(_)
        | DensityFunction::WeirdScaled{..} => DensityFunction::Constant { value: 0.0 },
        DensityFunction::ShiftedNoise{..} => DensityFunction::Constant { value: 0.0 },
        DensityFunction::Constant{value} => DensityFunction::Constant{value:*value},
        DensityFunction::LinearOp{op,input,c,mn,mx} => DensityFunction::LinearOp{op:*op,input:Box::new(strip_noise(input)),c:*c,mn:*mn,mx:*mx},
        DensityFunction::BinaryOp{op,a,b,mn,mx} => DensityFunction::BinaryOp{op:*op,a:Box::new(strip_noise(a)),b:Box::new(strip_noise(b)),mn:*mn,mx:*mx},
        DensityFunction::UnaryOp{op,input,mn,mx} => DensityFunction::UnaryOp{op:*op,input:Box::new(strip_noise(input)),mn:*mn,mx:*mx},
        DensityFunction::Clamp{input,mn,mx} => DensityFunction::Clamp{input:Box::new(strip_noise(input)),mn:*mn,mx:*mx},
        DensityFunction::Spline(s) => { let mut nd=s.clone(); nd.loc_fns = s.loc_fns.iter().map(|f| Arc::new(strip_noise(f))).collect(); DensityFunction::Spline(nd) }
        DensityFunction::Cache2D(c) => DensityFunction::Cache2D(WorldgenRust::density::Cache2DData::new(Arc::new(strip_noise(&c.arg)))),
        DensityFunction::FlatCache(f) => DensityFunction::FlatCache(WorldgenRust::density::FlatCacheData::new(Arc::new(strip_noise(&f.arg)))),
        DensityFunction::RangeChoice{input,min_inclusive,max_exclusive,in_range,out_of_range} => DensityFunction::RangeChoice{input:Box::new(strip_noise(input)),min_inclusive:*min_inclusive,max_exclusive:*max_exclusive,in_range:Box::new(strip_noise(in_range)),out_of_range:Box::new(strip_noise(out_of_range))},
        DensityFunction::YClampedGradient{from_y,to_y,from_value,to_value} => DensityFunction::YClampedGradient{from_y:*from_y,to_y:*to_y,from_value:*from_value,to_value:*to_value},
        DensityFunction::BlendAlpha => DensityFunction::BlendAlpha,
        DensityFunction::BlendOffset => DensityFunction::BlendOffset,
        DensityFunction::BlendDensity{input} => DensityFunction::BlendDensity{input:Box::new(strip_noise(input))},
        DensityFunction::Wrapping{input} => DensityFunction::Wrapping{input:Box::new(strip_noise(input))},
        DensityFunction::Interpolated(id) => DensityFunction::Interpolated(id.clone()),
        DensityFunction::Lazy{target} => DensityFunction::Lazy{target:target.clone()},
        DensityFunction::ReadChannel{ch,mn,mx} => DensityFunction::ReadChannel{ch:*ch,mn:*mn,mx:*mx},
    }
}

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = -8248318472910187742;
    let mut db = DensityBuilder::new(seed as u64, -64, 384);
    db.load_noise_params_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    let df_dir = format!("{}/data/minecraft/worldgen/density_function/overworld", wg_dir);
    let df_dir2 = df_dir.clone();
    db.set_df_ns("overworld");
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        std::fs::read_to_string(&format!("{}/{}.json", df_dir2, name)).unwrap()
    }));
    let settings = parse(&std::fs::read_to_string(format!("{}/data/minecraft/worldgen/noise_settings/overworld.json", wg_dir)).unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let tree = db.build_node(router.get("final_density").unwrap()).ok().unwrap();
    let (channels, _) = macrolize_channels(&tree);
    let ch0 = &channels[0];
    let ch0_stripped = strip_noise(ch0);

    let cx = -288; let cz = -256;
    let cell_w = 4; let cell_h = 8;
    let gx = 16/4+1; let gy = 384/8+1; let gz = 16/4+1;
    // 预热
    for ch in [ch0, &ch0_stripped] { for y in [-64i32,0,100,200,300] { for z in [0i32,8,15] { for x in [0i32,3,15] { let _ = ch.sample(&NoisePos{x:cx*16+x,y,z:cz*16+z}); }}}}
    // 测完整 ch#0 corners
    let t0 = Instant::now();
    for _r in 0..10 { for ix in 0..gx { for iz in 0..gz { for iy in 0..gy {
        let px=cx*16+ix*cell_w; let py=-64+iy*cell_h; let pz=cz*16+iz*cell_w;
        let _ = ch0.sample(&NoisePos{x:px,y:py,z:pz});
    }}}}
    let dt_full = t0.elapsed().as_secs_f64()/10.0*1e3;
    // 测 stripped（noise=0）ch#0 corners
    let t1 = Instant::now();
    for _r in 0..10 { for ix in 0..gx { for iz in 0..gz { for iy in 0..gy {
        let px=cx*16+ix*cell_w; let py=-64+iy*cell_h; let pz=cz*16+iz*cell_w;
        let _ = ch0_stripped.sample(&NoisePos{x:px,y:py,z:pz});
    }}}}
    let dt_stripped = t1.elapsed().as_secs_f64()/10.0*1e3;
    println!("ch#0 完整 corners 采样: {:.2}ms/chunk", dt_full);
    println!("ch#0 去 noise(noise=0) corners: {:.2}ms/chunk", dt_stripped);
    println!("noise 采样贡献: {:.2}ms ({:.0}%); 树遍历贡献: {:.2}ms ({:.0}%)",
        dt_full-dt_stripped, (dt_full-dt_stripped)/dt_full*100.0, dt_stripped, dt_stripped/dt_full*100.0);
}
