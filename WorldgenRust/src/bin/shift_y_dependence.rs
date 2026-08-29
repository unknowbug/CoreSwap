// shift_y_dependence.rs — 测 ShiftDF 的 y 依赖（能否 Cache2D 缓存）。
// 遍历 ch#0 树找 ShiftDF，测同 xz 不同 y 的值差异。若差异小 → y 独立可缓存。
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, NoisePos, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;

fn find_shiftdfs<'a>(df: &'a DensityFunction, out: &mut Vec<&'a DensityFunction>) {
    if let DensityFunction::ShiftDF{..} = df { out.push(df); }
    match df {
        DensityFunction::LinearOp{input,..}|DensityFunction::UnaryOp{input,..}|DensityFunction::Clamp{input,..}
        |DensityFunction::BlendDensity{input}|DensityFunction::Wrapping{input}=> find_shiftdfs(input, out),
        DensityFunction::BinaryOp{a,b,..}=>{ find_shiftdfs(a,out); find_shiftdfs(b,out); }
        DensityFunction::Spline(s)=>{ for f in &s.loc_fns { find_shiftdfs(f,out); } }
        DensityFunction::Cache2D(c)=>{ find_shiftdfs(&c.arg,out); } DensityFunction::FlatCache(f)=>{ find_shiftdfs(&f.arg,out); }
        DensityFunction::ShiftedNoise{shift_x,shift_y,shift_z,..}=>{ find_shiftdfs(shift_x,out); find_shiftdfs(shift_y,out); find_shiftdfs(shift_z,out); }
        DensityFunction::RangeChoice{input,in_range,out_of_range,..}=>{ find_shiftdfs(input,out); find_shiftdfs(in_range,out); find_shiftdfs(out_of_range,out); }
        DensityFunction::WeirdScaled{input,..}=>{ find_shiftdfs(input,out); }
        DensityFunction::Lazy{..}=>{ /* 跳过 Lazy 内部（占位/循环引用，ShiftDF 不在其直接 subtree） */ }
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
    let mut sh = Vec::new();
    find_shiftdfs(&channels[0], &mut sh);
    println!("ch#0 ShiftDF 节点数: {}", sh.len());
    // 取前 5 个 ShiftDF，测 y 依赖（同 xz，y=0..320 step 8）
    for (idx, sdf) in sh.iter().take(5).enumerate() {
        let (x, z) = (-288i32*16+4, -256i32*16+4);
        let mut vals = Vec::new();
        for y in (0..320).step_by(8) { vals.push(sdf.sample(&NoisePos{x, y, z})); }
        let maxd = (0..vals.len()-1).map(|i| (vals[i+1]-vals[i]).abs()).fold(0.0f64, f64::max);
        let total = vals.iter().map(|v| v.abs()).sum::<f64>();
        println!("  ShiftDF#{} y=0..320 max_delta={:.4} total={:.4} y_range_impact={:.2}%", idx, maxd, total, maxd/(total+1e-9)*100.0);
    }
}
