// channel_probe.rs — S0 通道分析：遍历 finalDensity 树，收集所有 Interpolated 标记（通道）及其内层构成。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::DensityFunction;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

// 递归遍历树，遇到 Interpolated 收集 (id, 内层用途摘要)
fn walk(df: &DensityFunction, out: &mut Vec<(u32, String)>) {
    match df {
        DensityFunction::Interpolated(id) => {
            let mut inner = String::new();
            summarize(&id.arg, 0, &mut inner);
            out.push((id.id, format!("arg=[{}]", inner)));
        }
        DensityFunction::BinaryOp { a, b, .. } => { walk(a, out); walk(b, out); }
        DensityFunction::UnaryOp { input, .. } => walk(input, out),
        DensityFunction::Clamp { input, .. } => walk(input, out),
        DensityFunction::LinearOp { input, .. } => walk(input, out),
        DensityFunction::Spline(s) => { for f in &s.loc_fns { walk(f, out); } }
        DensityFunction::Cache2D(c) => walk(&c.arg, out),
        DensityFunction::FlatCache(f) => walk(&f.arg, out),
        DensityFunction::ShiftedNoise { shift_x, shift_y, shift_z, .. } => { walk(shift_x, out); walk(shift_y, out); walk(shift_z, out); }
        DensityFunction::RangeChoice { input, in_range, out_of_range, .. } => { walk(input, out); walk(in_range, out); walk(out_of_range, out); }
        DensityFunction::BlendDensity { input } => walk(input, out),
        DensityFunction::Wrapping { input } => walk(input, out),
        DensityFunction::Constant { .. } | DensityFunction::Noise { .. } | DensityFunction::ShiftDF { .. }
        | DensityFunction::YClampedGradient { .. } | DensityFunction::WeirdScaled { .. }
        | DensityFunction::BlendAlpha | DensityFunction::BlendOffset | DensityFunction::InterpolatedNoise(_)
        | DensityFunction::Lazy { .. } => {}
    }
}

// 统计内层构成（spline/interp/noise/... 计数），带深度上限防爆炸。
fn summarize(df: &DensityFunction, depth: usize, out: &mut String) {
    if depth > 14 { out.push_str("..."); return; }
    match df {
        DensityFunction::Spline(s) => { out.push_str(&format!("spline[{}] ", s.nodes.len())); for f in &s.loc_fns { summarize(f, depth+1, out); } }
        DensityFunction::Interpolated(id) => out.push_str(&format!("interp#{} ", id.id)),
        DensityFunction::InterpolatedNoise(_) => out.push_str("blended "),
        DensityFunction::Noise { .. } => out.push_str("noise "),
        DensityFunction::BinaryOp { a, b, .. } => { out.push_str("bin("); summarize(a, depth+1, out); summarize(b, depth+1, out); out.push_str(") "); }
        DensityFunction::UnaryOp { input, .. } => { out.push_str("un("); summarize(input, depth+1, out); out.push_str(") "); }
        DensityFunction::Clamp { input, .. } => { out.push_str("clamp("); summarize(input, depth+1, out); out.push_str(") "); }
        DensityFunction::RangeChoice { input, in_range, out_of_range, .. } => { out.push_str("rc("); summarize(input, depth+1, out); summarize(in_range, depth+1, out); summarize(out_of_range, depth+1, out); out.push_str(") "); }
        DensityFunction::Cache2D(c) => { out.push_str("c2d("); summarize(&c.arg, depth+1, out); out.push_str(") "); }
        DensityFunction::FlatCache(f) => { out.push_str("flat("); summarize(&f.arg, depth+1, out); out.push_str(") "); }
        DensityFunction::Wrapping { input } => { out.push_str("wrap("); summarize(input, depth+1, out); out.push_str(") "); }
        DensityFunction::Constant { .. } => out.push_str("const "),
        DensityFunction::YClampedGradient { .. } => out.push_str("ycg "),
        DensityFunction::ShiftDF { .. } => out.push_str("shift "),
        DensityFunction::BlendDensity { input } => { out.push_str("blendd("); summarize(input, depth+1, out); out.push_str(") "); }
        DensityFunction::WeirdScaled { input, .. } => { out.push_str("weird("); summarize(input, depth+1, out); out.push_str(") "); }
        DensityFunction::ShiftedNoise { shift_x, shift_y, shift_z, .. } => { out.push_str("shnoise("); summarize(shift_x, depth+1, out); summarize(shift_y, depth+1, out); summarize(shift_z, depth+1, out); out.push_str(") "); }
        DensityFunction::LinearOp { input, .. } => { out.push_str("lin("); summarize(input, depth+1, out); out.push_str(") "); }
        DensityFunction::BlendAlpha => out.push_str("blendA "),
        DensityFunction::BlendOffset => out.push_str("blendO "),
        DensityFunction::Lazy { .. } => out.push_str("lazy "),
    }
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

    let mut chans = Vec::new();
    walk(&tree, &mut chans);
    println!("finalDensity Interpolated markers (channels): {}", chans.len());
    for (id, desc) in &chans { println!("  ch#{} {}", id, desc); }
    println!("(SteelMC overworld expects ~8 channels: 1 terrain + 4 noodle + 3 vein)");
}
