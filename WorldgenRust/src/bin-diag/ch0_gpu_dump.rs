// ch0_gpu_dump.rs — 260903-06 P-A：GPU ch0 @ 指定 (wx,wz) 列角点 dump（vs C++ CPU dfDump oracle 对比）。
// 角点 y = -64+iy*8（cell min-corner），slices 布局 i=(iy*5+iz)*5+ix（ix 最低位）。
// 用法：WG_GPU_CHANNELS=1 + PATH 含 build-msvc/bin；仓库根运行。
// 参数（编译期固定，与 ch0-cpp-dump-*-260903-06.txt 三列一致）：
//   (4,16) chunk(0,0) | (3208,3208) chunk(200,200) | (-36,-76) chunk(-3,-5)
use WorldgenRust::terrain::{ChunkDensitySampler, DensitySource};
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let seed: i64 = 8576294172403134396;
    let wg_dir = "versions/1.20.1/data/worldgen";
    let min_y = -64; let nh = 384;
    println!("=== ch0_gpu_dump seed={} ===", seed);
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "overworld.json", "biome_params.json", 384) {
        Some(h) => h,
        None => { eprintln!("[FAIL] handle create"); std::process::exit(1); }
    };
    let gc = match h.gpu_channels_density() {
        Some(g) => g,
        None => { eprintln!("[FAIL] WG_GPU_CHANNELS not set or engine create failed"); std::process::exit(1); }
    };
    let cols: Vec<(i32, i32)> = vec![(4, 16), (3208, 3208), (-36, -76)];
    for (wx, wz) in cols {
        let cx = wx >> 4; let cz = wz >> 4;
        let ix = ((wx & 15) / 4) as usize; let iz = ((wz & 15) / 4) as usize;
        println!("--- column wx={} wz={} chunk({},{}), local corner ix={} iz={} ---", wx, wz, cx, cz, ix, iz);
        let gpu = match gc.sample_chunk(cx, cz, min_y, nh) {
            Some(cd) => cd.slices().to_vec(),
            None => { eprintln!("[FAIL] sample_chunk None"); std::process::exit(1); }
        };
        let nch = 5usize;
        let gx = 5usize; let gz = 5usize;
        let gy = (nh / 8 + 1) as usize; // 49
        for iy in 0..gy {
            let i = (iy * gz + iz) * gx + ix;
            let y = min_y + iy as i32 * 8;
            println!("{} {:.17}", y, gpu[i * nch]); // ch0 = interp_order[0]
        }
    }
    println!("=== done ===");
}
