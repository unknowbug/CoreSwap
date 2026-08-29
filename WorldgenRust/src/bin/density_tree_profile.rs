// density_tree_profile.rs — 分析 finalDensity 树结构（各节点类型总数），诊断是否指数级膨胀
use WorldgenRust::density::DensityFunction;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use std::collections::HashMap;
use std::sync::Arc;

fn count(node: &DensityFunction, counters: &mut HashMap<String, usize>) {
    let key = match node {
        DensityFunction::Constant { .. } => "Constant",
        DensityFunction::Noise { .. } => "Noise",
        DensityFunction::LinearOp { .. } => "LinearOp",
        DensityFunction::BinaryOp { .. } => "BinaryOp",
        DensityFunction::UnaryOp { .. } => "UnaryOp",
        DensityFunction::Clamp { .. } => "Clamp",
        DensityFunction::Spline(_) => "Spline",
        DensityFunction::Interpolated(_) => "Interpolated",
        DensityFunction::Cache2D(_) => "Cache2D",
        DensityFunction::FlatCache(_) => "FlatCache",
        DensityFunction::ShiftDF { .. } => "ShiftDF",
        DensityFunction::ShiftedNoise { .. } => "ShiftedNoise",
        DensityFunction::RangeChoice { .. } => "RangeChoice",
        DensityFunction::YClampedGradient { .. } => "YClampedGradient",
        DensityFunction::WeirdScaled { .. } => "WeirdScaled",
        DensityFunction::BlendAlpha => "BlendAlpha",
        DensityFunction::BlendOffset => "BlendOffset",
        DensityFunction::BlendDensity { .. } => "BlendDensity",
        DensityFunction::Wrapping { .. } => "Wrapping",
        DensityFunction::InterpolatedNoise(_) => "InterpolatedNoise",
        DensityFunction::Lazy { .. } => "Lazy",
        DensityFunction::ReadChannel { .. } => "ReadChannel",
    };
    *counters.entry(key.to_string()).or_insert(0) += 1;
    match node {
        DensityFunction::LinearOp { input, .. } => count(input, counters),
        DensityFunction::BinaryOp { a, b, .. } => { count(a, counters); count(b, counters); }
        DensityFunction::UnaryOp { input, .. } => count(input, counters),
        DensityFunction::Clamp { input, .. } => count(input, counters),
        DensityFunction::Spline(s) => { for lf in &s.loc_fns { count(lf, counters); } }
        DensityFunction::Interpolated(d) => count(&d.arg, counters),
        DensityFunction::Cache2D(d) => count(&d.arg, counters),
        DensityFunction::FlatCache(d) => count(&d.arg, counters),
        DensityFunction::ShiftedNoise { shift_x, shift_y, shift_z, .. } => { count(shift_x, counters); count(shift_y, counters); count(shift_z, counters); }
        DensityFunction::RangeChoice { input, in_range, out_of_range, .. } => { count(input, counters); count(in_range, counters); count(out_of_range, counters); }
        DensityFunction::WeirdScaled { input, .. } => count(input, counters),
        DensityFunction::BlendDensity { input } => count(input, counters),
        DensityFunction::Wrapping { input } => count(input, counters),
        _ => {}
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
    // 支持测任意 router DF（默认 final_density，可用 env DENSITY_TREE_KEY 指定）
    let key = std::env::var("DENSITY_TREE_KEY").unwrap_or_else(|_| "final_density".to_string());
    let tree: Arc<DensityFunction> = Arc::new(db.build_node(router.get(key.as_str()).unwrap()).ok().unwrap());
    println!("=== {} tree node counts ===", key);
    let mut counters = HashMap::new();
    count(&tree, &mut counters);
    let mut items: Vec<_> = counters.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    let total: usize = items.iter().map(|(_, c)| c).sum();
    println!("TOTAL density function nodes: {}", total);
    for (k, c) in &items { println!("  {:<22} {}", k, c); }
}
