// transpiler_prod_ch0.rs — dump 运行时 channels[0]（BlendDensity terrain）结构，对比 transpiler out[0]。
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;

fn dump_limited(df: &DensityFunction, depth: usize, max_depth: usize) {
    let ind = "  ".repeat(depth);
    if depth > max_depth { println!("{}...", ind); return; }
    match df {
        DensityFunction::ReadChannel { ch, mn, mx } => println!("{}ReadChannel[{}] (mn={:.4} mx={:.4})", ind, ch, mn, mx),
        DensityFunction::Constant { value } => println!("{}Constant({})", ind, value),
        DensityFunction::BinaryOp { op, a, b, mn, mx } => {
            println!("{}BinaryOp op={} (mn={:.4} mx={:.4})", ind, match op {
                WorldgenRust::density::BinOp::Add => "Add", WorldgenRust::density::BinOp::Mul => "Mul",
                WorldgenRust::density::BinOp::Min => "MIN", WorldgenRust::density::BinOp::Max => "Max",
            }, mn, mx);
            dump_limited(a, depth+1, max_depth); dump_limited(b, depth+1, max_depth);
        }
        DensityFunction::LinearOp { op, input, c, mn, mx } => {
            println!("{}LinearOp c={} (mn={:.4} mx={:.4})", ind, c, mn, mx);
            dump_limited(input, depth+1, max_depth);
        }
        DensityFunction::UnaryOp { op, input, mn, mx } => {
            println!("{}UnaryOp (mn={:.4} mx={:.4})", ind, mn, mx);
            dump_limited(input, depth+1, max_depth);
        }
        DensityFunction::Clamp { input, mn, mx, .. } => {
            println!("{}Clamp (mn={:.4} mx={:.4})", ind, mn, mx);
            dump_limited(input, depth+1, max_depth);
        }
        DensityFunction::RangeChoice { input, min_inclusive, max_exclusive, in_range, out_of_range } => {
            println!("{}RangeChoice([{}, {}))", ind, min_inclusive, max_exclusive);
            dump_limited(input, depth+1, max_depth); dump_limited(in_range, depth+1, max_depth); dump_limited(out_of_range, depth+1, max_depth);
        }
        DensityFunction::Noise { noise, xz_scale, y_scale, .. } => println!("{}Noise(xz={}, y={})", ind, xz_scale, y_scale),
        DensityFunction::Interpolated(_) => println!("{}Interpolated", ind),
        DensityFunction::BlendDensity { input } => {
            println!("{}BlendDensity", ind);
            dump_limited(input, depth+1, max_depth);
        }
        DensityFunction::InterpolatedNoise(_) => println!("{}InterpolatedNoise", ind),
        DensityFunction::Spline(_) => println!("{}Spline", ind),
        DensityFunction::Cache2D(_) => println!("{}Cache2D", ind),
        DensityFunction::FlatCache(_) => println!("{}FlatCache", ind),
        DensityFunction::ShiftDF { .. } => println!("{}ShiftDF", ind),
        DensityFunction::ShiftedNoise { .. } => println!("{}ShiftedNoise", ind),
        DensityFunction::YClampedGradient { .. } => println!("{}YClampedGradient", ind),
        DensityFunction::Lazy { .. } => println!("{}Lazy", ind),
        DensityFunction::Wrapping { .. } => println!("{}Wrapping", ind),
        DensityFunction::BlendAlpha => println!("{}BlendAlpha", ind),
        DensityFunction::BlendOffset => println!("{}BlendOffset", ind),
        DensityFunction::WeirdScaled { .. } => println!("{}WeirdScaled", ind),
        _ => println!("{}other", ind),
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
    let tree = Arc::new(db.build_node(router.get("final_density").unwrap()).ok().unwrap());

    let (channels, _combine) = macrolize_channels(&tree);
    println!("channels 数: {}", channels.len());
    for ch in 0..channels.len() {
        println!("=== channels[{}] 结构 (depth 限 10) ===", ch);
        dump_limited(&channels[ch], 0, 10);
    }
}
