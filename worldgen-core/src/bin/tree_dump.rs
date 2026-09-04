// tree_dump.rs — 打印 temperature/vegetation density 树结构（确认是否含 Interpolated 3D）。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::DensityFunction;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn describe(df: &DensityFunction, depth: usize) {
    let pad = "  ".repeat(depth);
    match df {
        DensityFunction::Interpolated(_) => println!("{}Interpolated (3D grid)", pad),
        DensityFunction::Cache2D(_) => println!("{}Cache2D", pad),
        DensityFunction::FlatCache(_) => println!("{}FlatCache", pad),
        DensityFunction::Noise { .. } => println!("{}Noise", pad),
        DensityFunction::Spline(_) => println!("{}Spline", pad),
        DensityFunction::ShiftedNoise { shift_x, shift_y, shift_z, .. } => {
            println!("{}ShiftedNoise", pad);
            describe(shift_x, depth+1); describe(shift_y, depth+1); describe(shift_z, depth+1);
        }
        DensityFunction::BinaryOp { a, b, .. } => { println!("{}BinaryOp", pad); describe(a, depth+1); describe(b, depth+1); }
        DensityFunction::UnaryOp { input, .. } => { println!("{}UnaryOp", pad); describe(input, depth+1); }
        DensityFunction::LinearOp { input, .. } => { println!("{}LinearOp", pad); describe(input, depth+1); }
        DensityFunction::Clamp { input, .. } => { println!("{}Clamp", pad); describe(input, depth+1); }
        DensityFunction::RangeChoice { input, in_range, out_of_range, .. } => {
            println!("{}RangeChoice", pad); describe(input, depth+1); describe(in_range, depth+1); describe(out_of_range, depth+1);
        }
        DensityFunction::WeirdScaled { input, .. } => { println!("{}WeirdScaled", pad); describe(input, depth+1); }
        DensityFunction::BlendDensity { input } => { println!("{}BlendDensity", pad); describe(input, depth+1); }
        DensityFunction::Wrapping { input } => { println!("{}Wrapping", pad); describe(input, depth+1); }
        DensityFunction::InterpolatedNoise(_) => println!("{}InterpolatedNoise", pad),
        DensityFunction::Lazy { target } => {
            println!("{}Lazy", pad);
            let t = target.lock().unwrap();
            if let Some(t) = t.as_ref() { describe(t, depth+1); }
        }
        DensityFunction::ShiftDF { .. } => println!("{}ShiftDF", pad),
        DensityFunction::YClampedGradient { .. } => println!("{}YClampedGradient", pad),
        DensityFunction::BlendAlpha => println!("{}BlendAlpha", pad),
        DensityFunction::BlendOffset => println!("{}BlendOffset", pad),
        DensityFunction::Constant { .. } => println!("{}Constant", pad),
        _ => println!("{}other", pad),
    }
}

fn main() {
    let seed: i64 = -2032795982907864146;
    let mut db = DensityBuilder::new(seed as u64, -64, 384i32);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}", p.display()))
    }));
    let settings = parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    for name in ["temperature", "vegetation", "continents"] {
        let df = db.build_node(router.get(name).unwrap()).unwrap();
        println!("=== {} tree ===", name);
        describe(&df, 0);
    }
}
