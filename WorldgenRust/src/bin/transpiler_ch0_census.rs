// transpiler_ch0_census.rs — 判决性普查：channels[0]（macrolize 后）是否残留 Interpolated marker。
// 依据（transpiler_ch0_decompose）：runtime 精确点内部值偏离自身角值双线性 0.069、transpiler 自洽；
// diff 随 y 从 ~52 起线性增长。若 runtime channel 内残留 Interpolated marker（macrolize 漏转嵌套），
// 则 runtime 内部点=格点插值、corner=精确值，与全部证据吻合。
// 输出：channels[0] 全树节点类型统计 + Interpolated/其他缓存节点的出现路径。
use std::collections::BTreeMap;
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;

fn walk(df: &DensityFunction, path: &str, counts: &mut BTreeMap<String, u64>, hits: &mut Vec<(String, String)>) {
    let ty = node_type(df);
    *counts.entry(ty.clone()).or_insert(0) += 1;
    if ty == "Interpolated" && hits.iter().filter(|(t, _)| t == "Interpolated").count() < 5 {
        hits.push(("Interpolated".into(), path.to_string()));
    }
    let child_path = |seg: &str| format!("{}.{}", path, seg);
    match df {
        DensityFunction::Interpolated(id) => {
            walk(&id.arg, &child_path("arg"), counts, hits);
        }
        DensityFunction::Cache2D(c) => walk(&c.arg, &child_path("cache2d"), counts, hits),
        DensityFunction::FlatCache(f) => walk(&f.arg, &child_path("flatcache"), counts, hits),
        DensityFunction::LinearOp { input, .. } => walk(input, &child_path("linear"), counts, hits),
        DensityFunction::BinaryOp { a, b, .. } => {
            walk(a, &child_path("a"), counts, hits);
            walk(b, &child_path("b"), counts, hits);
        }
        DensityFunction::UnaryOp { input, .. } => walk(input, &child_path("unary"), counts, hits),
        DensityFunction::Clamp { input, .. } => walk(input, &child_path("clamp"), counts, hits),
        DensityFunction::BlendDensity { input } => walk(input, &child_path("blenddensity"), counts, hits),
        DensityFunction::Wrapping { input } => walk(input, &child_path("wrapping"), counts, hits),
        DensityFunction::RangeChoice { input, in_range, out_of_range, .. } => {
            walk(input, &child_path("rc.input"), counts, hits);
            walk(in_range, &child_path("rc.in"), counts, hits);
            walk(out_of_range, &child_path("rc.out"), counts, hits);
        }
        DensityFunction::WeirdScaled { input, .. } => walk(input, &child_path("weird"), counts, hits),
        DensityFunction::ShiftedNoise { shift_x, shift_y, shift_z, .. } => {
            walk(shift_x, &child_path("shx"), counts, hits);
            walk(shift_y, &child_path("shifty"), counts, hits);
            walk(shift_z, &child_path("shiftz"), counts, hits);
        }
        DensityFunction::Spline(s) => {
            for (i, f) in s.loc_fns.iter().enumerate() {
                walk(f, &child_path(&format!("spline.loc{}", i)), counts, hits);
            }
        }
        DensityFunction::Lazy { target } => {
            if let Ok(t) = target.lock() {
                if let Some(inner) = t.as_ref() {
                    walk(inner, &child_path("lazy"), counts, hits);
                }
            }
        }
        _ => {}
    }
}

fn node_type(df: &DensityFunction) -> String {
    match df {
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
    }
    .to_string()
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

    let (channels, combine) = macrolize_channels(&tree);
    println!("channels = {}", channels.len());
    for (i, ch) in channels.iter().enumerate() {
        let mut counts = BTreeMap::new();
        let mut hits: Vec<(String, String)> = Vec::new();
        walk(ch, &format!("channels[{}]", i), &mut counts, &mut hits);
        let interp = counts.get("Interpolated").copied().unwrap_or(0);
        println!("channels[{}]: Interpolated残留={} (全: {})", i, interp,
            counts.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(" "));
        for (t, p) in &hits {
            if t == "Interpolated" { println!("   └ Interpolated at {}", p); }
        }
    }
    // combine 树也查（population 路径）
    let mut counts = BTreeMap::new();
    let mut hits: Vec<(String, String)> = Vec::new();
    walk(&combine, "combine", &mut counts, &mut hits);
    println!("combine: {}", counts.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(" "));
    for (t, p) in &hits {
        if t == "Interpolated" { println!("   └ Interpolated at {}", p); }
    }
}