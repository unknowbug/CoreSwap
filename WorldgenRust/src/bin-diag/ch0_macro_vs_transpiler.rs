// ch0_macro_vs_transpiler.rs — X2 ch0 判别（260903-05）：生产默认 macro_sampler vs transpiler 的 ch0 列对拍。
// 判读：macro ch0 带噪声扰动 + transpiler ch0 纯线性 → transpiler 特有缺陷（fallback 修复目标）；
//       两者皆纯线性 → Rust 全局 ch0 缺陷（影响面升级）。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = 8576294172403134396;
    let wg_dir = "versions/1.20.1/data/worldgen";
    let h = WorldgenHandle::create_for_dim(seed, wg_dir, "overworld.json", "biome_params.json", 384).expect("handle");
    let td = h.transpiler_density().expect("WG_TRANSPILER not set");
    let macro_s = h.macro_sampler();
    // 260903-06 判别：清缓存后重跑（若 transpiler 列转正确 → handle 进程内 C2D_CACHE 污染）
    let clear = std::env::var("CH0_CLEAR").is_ok();
    let st = {
        if clear { WorldgenRust::density::transpiler_cache_clear_all(); }
        td.build_slices_for(0, 0)
    };
    let sm = macro_s.build_slices_for(0, 0);
    let gx = 5usize; let gz = 5usize;
    let (ix, iz) = (1usize, 4usize);
    println!("=== ch0 column x=4 z=16: transpiler vs macro ===");
    let mut prev_t = f64::NAN; let mut prev_m = f64::NAN;
    for iy in 0..49usize {
        let vt = st[((iy * gz + iz) * gx + ix) * 5];
        let vm = sm[((iy * gz + iz) * gx + ix) * 5];
        let y = -64 + iy as i32 * 8;
        println!("y={:>4}: transpiler={:>10.6}{} macro={:>10.6}{} diff={:.6}",
                 y, vt, if vt == prev_t { "*" } else { " " }, vm, if vm == prev_m { "*" } else { " " }, (vt - vm).abs());
        prev_t = vt; prev_m = vm;
    }
}
