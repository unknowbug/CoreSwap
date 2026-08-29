// shift_y_dependence.rs — 测 ShiftDF 的 y 依赖（能否 Cache2D 缓存）。
// 遍历 ch#0 树找 ShiftDF，测同 xz 不同 y 的值差异。若差异小 → y 独立可缓存。
use std::sync::Arc;
use WorldgenRust::density::{DensityFunction, NoisePos, ShiftMode, macrolize_channels};
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
    println!("ch#0 ShiftDF 节点数: {} (mode: Shift/ShiftA/ShiftB 分布)", sh.len());
    // 统计 mode 分布
    let (mut n_shift, mut n_a, mut n_b) = (0, 0, 0);
    for s in &sh {
        if let DensityFunction::ShiftDF { mode, .. } = s {
            match mode { WorldgenRust::density::ShiftMode::Shift => n_shift+=1, WorldgenRust::density::ShiftMode::ShiftA=>n_a+=1, WorldgenRust::density::ShiftMode::ShiftB=>n_b+=1 }
        }
    }
    println!("  Shift={} ShiftA={} ShiftB={}", n_shift, n_a, n_b);
    // 测每个 ShiftDF 的 y 独立性（多列 + 含负Y）
    let mut y_dep_count = 0;
    for (idx, sdf) in sh.iter().enumerate() {
        let (x, z) = (-288i32*16+4, -256i32*16+4);
        // 同 xz，y 从 -64 到 320（含负Y），步 8
        let mut vals = Vec::new();
        for y in (-64..320).step_by(8) { vals.push(sdf.sample(&NoisePos{x, y, z})); }
        let mut maxd = 0.0f64;
        for i in 0..vals.len()-1 { let d = (vals[i+1]-vals[i]).abs(); if d > maxd { maxd = d; } }
        if maxd > 1e-9 { y_dep_count += 1; }
        // 换 3 个列再确认
        if maxd <= 1e-9 {
            for (dx, dz) in [(8i32,8),(12,4),(0,0)] {
                let x2 = x + dx; let z2 = z + dz;
                let mut vals2 = Vec::new();
                for y in (-64..320).step_by(8) { vals2.push(sdf.sample(&NoisePos{x:x2,y,z:z2})); }
                for i in 0..vals2.len()-1 { let dd = (vals2[i+1]-vals2[i]).abs(); if dd > maxd { maxd = dd; } }
            }
            if maxd > 1e-9 { y_dep_count += 1; }
        }
        if idx < 3 || maxd > 1e-9 {
            println!("  ShiftDF#{} mode={:?} y_dep(max_delta={:.5})", idx, sdf.mode_label(), maxd);
        }
    }
    println!("y 独立(全部列 y_delta<=1e-9): {} / {} 节点", sh.len() - y_dep_count, sh.len());
}

// helper：label mode
trait ModeLabel { fn mode_label(&self) -> String; }
impl ModeLabel for WorldgenRust::density::DensityFunction {
    fn mode_label(&self) -> String {
        if let DensityFunction::ShiftDF { mode, .. } = self {
            match mode { ShiftMode::Shift => "Shift".into(), ShiftMode::ShiftA => "ShiftA".into(), ShiftMode::ShiftB => "ShiftB".into() }
        } else { "?".to_string() }
    }
}
