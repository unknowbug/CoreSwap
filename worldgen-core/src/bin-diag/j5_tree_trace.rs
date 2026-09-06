// j5_tree_trace.rs — 260906-01 J5 单棵 mega RNG 流 trace（bin-diag 诊断，不参与默认构建）
// 用法：j5_tree_trace.exe <seed> <cx> <cz>  → WG_TREEDIAG=1 时 stderr 输出 [MJT*]/[TH]/[TREESET] 打点
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: i64 = if args.len() > 1 { args[1].parse().expect("seed i64") } else { 8576294172403134396 };
    let cx: i32 = if args.len() > 2 { args[2].parse().expect("cx") } else { 29 };
    let cz: i32 = if args.len() > 3 { args[3].parse().expect("cz") } else { -16 };
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    eprintln!("[j5] seed={} chunk=({}, {})", seed, cx, cz);
    let h = WorldgenHandle::create(seed, wg_dir).expect("create handle");
    eprintln!("[j5] handle ready min_y={} height={}", h.min_y, h.height);
    // 邻域预生成（feature 放置依赖邻 chunk 状态，5×5 覆盖装饰半径）
    for dx in -2..=2 {
        for dz in -2..=2 {
            let blocks = h.fill_chunk_blocks(cx + dx, cz + dz);
            if dx == 0 && dz == 0 {
                let nonair = blocks.iter().filter(|b| **b != 0).count();
                eprintln!("[j5] target chunk generated, nonair={}", nonair);
            }
        }
    }
    eprintln!("[j5] done");
}
