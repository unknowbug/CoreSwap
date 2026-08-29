// macrolize_probe.rs — 验证 multi-channel 竖切（macrolize_channels）：
// ① channel 收集数量与构成 ② 树中 Interpolated 是否全部替换为 ReadChannel ③ combine 结构 sanity。
use WorldgenRust::density::{DensityFunction, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;
use std::sync::Arc;

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
    println!("macrolize: channels={} combine_root={}", channels.len(), node_name(&combine));
    for (i, ch) in channels.iter().enumerate() {
        println!("  ch#{}: {} nodes={}", i, node_name(ch), count_nodes(ch));
    }
    // 验证 combine 树里没有残留 Interpolated（应全部替换为 ReadChannel）
    let n_interp_left = count_interp(&combine);
    let n_readch = count_readch(&combine);
    println!("combine 树残留 Interpolated: {}; ReadChannel: {} (期望 0 / >=1)", n_interp_left, n_readch);

    // 简单数值 sanity：ReadChannel 用固定 channel 值，combine 应产出确定值
    let pos = WorldgenRust::density::NoisePos { x: -288*16+4, y: 4, z: -256*16+4 };
    let interp: Vec<f64> = (0..channels.len()).map(|i| i as f64 * 0.01).collect();
    let v = combine.sample_combine(&pos, &interp);
    println!("combine 固定 channel 采样: {}", v);
}

fn node_name(df: &DensityFunction) -> &'static str {
    match df {
        DensityFunction::Constant {..}=>"Constant", DensityFunction::Noise{..}=>"Noise",
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
    }
}
fn count_nodes(df: &DensityFunction) -> usize {
    let mut n = 1;
    match df {
        DensityFunction::LinearOp{input,..}|DensityFunction::UnaryOp{input,..}|DensityFunction::Clamp{input,..}
        |DensityFunction::BlendDensity{input}|DensityFunction::Wrapping{input}=> n += count_nodes(input),
        DensityFunction::BinaryOp{a,b,..}=>{ n+=count_nodes(a); n+=count_nodes(b); }
        DensityFunction::Spline(s)=>{ for f in &s.loc_fns { n += count_nodes(f); } }
        DensityFunction::Cache2D(c)=>{ n += count_nodes(&c.arg); }
        DensityFunction::FlatCache(f)=>{ n += count_nodes(&f.arg); }
        DensityFunction::ShiftedNoise{shift_x,shift_y,shift_z,..}=>{ n+=count_nodes(shift_x); n+=count_nodes(shift_y); n+=count_nodes(shift_z); }
        DensityFunction::RangeChoice{input,in_range,out_of_range,..}=>{ n+=count_nodes(input); n+=count_nodes(in_range); n+=count_nodes(out_of_range); }
        DensityFunction::WeirdScaled{input,..}=>{ n+=count_nodes(input); }
        DensityFunction::Lazy{target}=>{ let t=target.lock().unwrap(); if let Some(x)=t.as_ref(){ n+=count_nodes(x);} }
        _=>{}
    }
    n
}
fn count_interp(df: &DensityFunction) -> usize {
    let mut c = if matches!(df, DensityFunction::Interpolated(_)) {1} else {0};
    match df {
        DensityFunction::LinearOp{input,..}|DensityFunction::UnaryOp{input,..}|DensityFunction::Clamp{input,..}
        |DensityFunction::BlendDensity{input}|DensityFunction::Wrapping{input}=> c += count_interp(input),
        DensityFunction::BinaryOp{a,b,..}=>{ c+=count_interp(a); c+=count_interp(b); }
        DensityFunction::Spline(s)=>{ for f in &s.loc_fns { c += count_interp(f); } }
        DensityFunction::Cache2D(cd)=>{ c += count_interp(&cd.arg); }
        DensityFunction::FlatCache(fd)=>{ c += count_interp(&fd.arg); }
        DensityFunction::ShiftedNoise{shift_x,shift_y,shift_z,..}=>{ c+=count_interp(shift_x); c+=count_interp(shift_y); c+=count_interp(shift_z); }
        DensityFunction::RangeChoice{input,in_range,out_of_range,..}=>{ c+=count_interp(input); c+=count_interp(in_range); c+=count_interp(out_of_range); }
        DensityFunction::WeirdScaled{input,..}=>{ c+=count_interp(input); }
        DensityFunction::Lazy{target}=>{ let t=target.lock().unwrap(); if let Some(x)=t.as_ref(){ c+=count_interp(x);} }
        _=>{}
    }
    c
}
fn count_readch(df: &DensityFunction) -> usize {
    let mut c = if matches!(df, DensityFunction::ReadChannel{..}) {1} else {0};
    match df {
        DensityFunction::LinearOp{input,..}|DensityFunction::UnaryOp{input,..}|DensityFunction::Clamp{input,..}
        |DensityFunction::BlendDensity{input}|DensityFunction::Wrapping{input}=> c += count_readch(input),
        DensityFunction::BinaryOp{a,b,..}=>{ c+=count_readch(a); c+=count_readch(b); }
        DensityFunction::Spline(s)=>{ for f in &s.loc_fns { c += count_readch(f); } }
        DensityFunction::Cache2D(cd)=>{ c += count_readch(&cd.arg); }
        DensityFunction::FlatCache(fd)=>{ c += count_readch(&fd.arg); }
        DensityFunction::ShiftedNoise{shift_x,shift_y,shift_z,..}=>{ c+=count_readch(shift_x); c+=count_readch(shift_y); c+=count_readch(shift_z); }
        DensityFunction::RangeChoice{input,in_range,out_of_range,..}=>{ c+=count_readch(input); c+=count_readch(in_range); c+=count_readch(out_of_range); }
        DensityFunction::WeirdScaled{input,..}=>{ c+=count_readch(input); }
        DensityFunction::Lazy{target}=>{ let t=target.lock().unwrap(); if let Some(x)=t.as_ref(){ c+=count_readch(x);} }
        _=>{}
    }
    c
}
