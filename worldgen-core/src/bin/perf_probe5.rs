// perf_probe5.rs — S2 前置量化：测 ch#9 巨型内层 arg 的单点采样成本 vs sloped_cheese（已知 0.9μs）。
// 若 ch#9 arg >> 0.9μs，说明其深递归/大表达式有直排（S2）空间；若接近，则直排收益有限。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

// 收集第一个 Interpolated 的 arg（ch#9 巨型内层）
fn first_interp_arg(df: &DensityFunction) -> Option<&Arc<DensityFunction>> {
    match df {
        DensityFunction::Interpolated(id) => Some(&id.arg),
        DensityFunction::BinaryOp { a, b, .. } => first_interp_arg(a).or_else(|| first_interp_arg(b)),
        DensityFunction::UnaryOp { input, .. } => first_interp_arg(input),
        DensityFunction::Clamp { input, .. } => first_interp_arg(input),
        DensityFunction::LinearOp { input, .. } => first_interp_arg(input),
        DensityFunction::Spline(s) => s.loc_fns.iter().find_map(|f| first_interp_arg(f.as_ref())),
        DensityFunction::Cache2D(c) => first_interp_arg(&c.arg),
        DensityFunction::FlatCache(f) => first_interp_arg(&f.arg),
        DensityFunction::ShiftedNoise { shift_x, shift_y, shift_z, .. } => first_interp_arg(shift_x).or_else(|| first_interp_arg(shift_y)).or_else(|| first_interp_arg(shift_z)),
        DensityFunction::RangeChoice { input, in_range, out_of_range, .. } => first_interp_arg(input).or_else(|| first_interp_arg(in_range)).or_else(|| first_interp_arg(out_of_range)),
        DensityFunction::BlendDensity { input } => first_interp_arg(input),
        DensityFunction::Wrapping { input } => first_interp_arg(input),
        _ => None,
    }
}

fn bench_varied(df: &DensityFunction, iters: usize) -> f64 {
    let (cx, cz) = (45, -26);
    let mut idx = 0usize;
    for _ in 0..10 { let _ = df.sample(&NoisePos { x: cx*16+(idx%16) as i32, y: -64+((idx/16)%96) as i32*4, z: cz*16+((idx/(16*96))%16) as i32 }); idx += 1; }
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for _ in 0..iters {
        let x = cx*16+(idx%16) as i32; let y = -64+((idx/16)%96) as i32*4; let z = cz*16+((idx/(16*96))%16) as i32;
        acc += df.sample(&NoisePos { x, y, z }); idx += 1;
    }
    let _ = acc;
    t0.elapsed().as_secs_f64() * 1e6 / iters as f64
}

fn main() {
    let mut db = DensityBuilder::new(8576294172403134396, -64, 384);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}", p.display()))
    }));
    let settings = parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let fd = settings.get("noise_router").and_then(|r| r.get("final_density")).unwrap();
    let tree = db.build_node(fd).unwrap();

    // ch#9 巨型内层
    let ch9 = first_interp_arg(&tree).expect("no interp arg");
    let us_ch9 = bench_varied(ch9, 3000);
    // 对比 sloped_cheese（已知 ~0.9μs）
    let sc = db.resolve_ref("minecraft:overworld/sloped_cheese");
    let us_sc = bench_varied(&sc, 3000);
    // 全 finalDensity
    let us_fd = bench_varied(&tree, 3000);
    println!("ch#9 arg (huge inner)     : {:.1} us/pt", us_ch9);
    println!("sloped_cheese             : {:.1} us/pt", us_sc);
    println!("finalDensity (outer tree) : {:.1} us/pt", us_fd);
    println!("=> ch#9 arg vs sloped_cheese ratio: {:.1}x", us_ch9 / us_sc);
}
