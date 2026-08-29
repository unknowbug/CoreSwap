// ch0_tree_analysis.rs — 分析 ch#0 (BlendDensity terrain) 树节点构成，评估 noise 采样是否大头。
use std::collections::HashMap;
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, NoisePos, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;

fn count(df: &DensityFunction, counters: &mut HashMap<&'static str, usize>) {
    let k = match df {
        DensityFunction::Constant{..}=>"Constant", DensityFunction::Noise{..}=>"Noise",
        DensityFunction::LinearOp{..}=>"LinearOp", DensityFunction::BinaryOp{..}=>"BinaryOp",
        DensityFunction::UnaryOp{..}=>"UnaryOp", DensityFunction::Clamp{..}=>"Clamp",
        DensityFunction::Spline(_)=>"Spline", DensityFunction::Interpolated(_)=>"Interpolated",
        DensityFunction::Cache2D(_)=>"Cache2D", DensityFunction::FlatCache(_)=>"FlatCache",
        DensityFunction::ShiftDF{..}=>"ShiftDF", DensityFunction::ShiftedNoise{..}=>"ShiftedNoise",
        DensityFunction::RangeChoice{..}=>"RangeChoice", DensityFunction::YClampedGradient{..}=>"YClampedGradient",
        DensityFunction::WeirdScaled{..}=>"WeirdScaled", DensityFunction::BlendAlpha=>"BlendAlpha",
        DensityFunction::BlendOffset=>"BlendOffset", DensityFunction::BlendDensity{..}=>"BlendDensity",
        DensityFunction::Wrapping{..}=>"Wrapping", DensityFunction::InterpolatedNoise(_)=>"InterpolatedNoise",
        DensityFunction::Lazy{..}=>"Lazy", DensityFunction::ReadChannel{..}=>"ReadChannel",
    };
    *counters.entry(k).or_insert(0) += 1;
    match df {
        DensityFunction::LinearOp{input,..}|DensityFunction::UnaryOp{input,..}|DensityFunction::Clamp{input,..}
        |DensityFunction::BlendDensity{input}|DensityFunction::Wrapping{input}=> count(input, counters),
        DensityFunction::BinaryOp{a,b,..}=>{ count(a,counters); count(b,counters); }
        DensityFunction::Spline(s)=>{ for f in &s.loc_fns { count(f,counters); } }
        DensityFunction::Cache2D(c)=>{ count(&c.arg,counters); } DensityFunction::FlatCache(f)=>{ count(&f.arg,counters); }
        DensityFunction::ShiftedNoise{shift_x,shift_y,shift_z,..}=>{ count(shift_x,counters); count(shift_y,counters); count(shift_z,counters); }
        DensityFunction::RangeChoice{input,in_range,out_of_range,..}=>{ count(input,counters); count(in_range,counters); count(out_of_range,counters); }
        DensityFunction::WeirdScaled{input,..}=>{ count(input,counters); }
        DensityFunction::Lazy{target}=>{ let t=target.lock().unwrap(); if let Some(x)=t.as_ref(){ count(x,counters);} }
        _=>{}
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
    let mut counters = HashMap::new();
    count(ch0, &mut counters);
    let mut items: Vec<_> = counters.iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    let total: usize = counters.values().sum();
    println!("ch#0 terrain tree node composition (total {}):", total);
    for (k, c) in &items { println!("  {:<20} {}", k, c); }
    // noise 相关估算（Noise + ShiftedNoise + InterpolatedNoise + WeirdScaled）
    let g = |k: &str| counters.get(k).copied().unwrap_or(0);
    let noise_related = g("Noise") + g("ShiftedNoise") + g("InterpolatedNoise") + g("WeirdScaled");
    println!("noise 相关节点: {}  ({:.0}%)", noise_related, noise_related as f64/total as f64*100.0);
    println!("(BlendDensity={} BlendAlpha={} BlendOffset={})",
        g("BlendDensity"), g("BlendAlpha"), g("BlendOffset"));
}
