// olb_hitrate_probe.rs — B-1 前置计数探针（260908-02）：old_blended 采样重复率（记忆化命中率上界）
// 动机：phase0-architecture 路线 B-1（old_blended 结果记忆化 LRU）的收益上界取决于坐标重复度——命中率未知，judge J1 要求探针钉死（含连片预生成爬升带）。
// 方法：WG_OLB_STATS 插桩（density.rs InterpolatedNoiseData::sample 入口，key=(实例指针,x,y,z)），
//       冷态 4 chunk + 顺序连片 512 chunk 爬升带，检查点累积 dump（total/unique/dup/capped）+ est L2 stats。
// 口径：单线程串行（#28）；est L2 状态随 env（注意 ⚠️ l2 默认值疑似开，见 #53 补充案例——本探针把 l2 当
//       「现状生产」口径测量，另跑 l2=off 臂可分离，暂不做）；dup 含跨阶段/跨 chunk 全部重复。
use std::time::Instant;
use WorldgenRust::density::olb_watch;
use WorldgenRust::worldgen_handle::WorldgenHandle;

const SEED: i64 = 8576294172403134396;
const WG_DIR: &str = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
const ORIGIN: (i32, i32) = (200, 200);

fn dump(h: &WorldgenHandle, tag: &str) {
    let (total, unique, dup, capped) = WorldgenRust::density::olb_stats();
    let l2 = h.est_l2_stats();
    let hit_rate = if total > 0 { dup as f64 / total as f64 * 100.0 } else { 0.0 };
    println!("[OLB:{}] total={} unique={} dup={} dup_ratio={:.2}% capped={} l2hits={} l2miss={}",
        tag, total, unique, dup, hit_rate, capped, l2[0], l2[1]);
}

fn main() {
    println!("=== olb_hitrate_probe (260908-02) seed={} ===", SEED);
    let h = WorldgenHandle::create(SEED, WG_DIR).expect("create handle");
    // 预热区外（watch 关）
    for i in 0..8 { let _ = h.fill_chunk_blocks(400 + (i % 4), 400 + (i / 4)); }

    // Phase 冷态：4 个新 chunk（200,200 起），watch 开
    WorldgenRust::density::olb_reset();
    olb_watch(true);
    let t = Instant::now();
    for cz in 0..2 { for cx in 0..2 { let _ = h.fill_chunk_blocks(ORIGIN.0 + cx, ORIGIN.1 + cz); } }
    println!("[cold] 4 chunks wall={:.1}ms", t.elapsed().as_secs_f64() * 1e3);
    dump(&h, "cold-4");

    // Phase 爬升带：从 (200,200) 顺序连片 32×16=512 chunk（行序，模拟 Chunky 式预生成）
    WorldgenRust::density::olb_reset();
    let t = Instant::now();
    let mut done = 0usize;
    for cz in 0..32usize {
        for cx in 0..16usize {
            let _ = h.fill_chunk_blocks(ORIGIN.0 + cx as i32, ORIGIN.1 + cz as i32);
            done += 1;
            if done % 128 == 0 {
                print!("[ramp:{}ch wall={:.0}ms] ", done, t.elapsed().as_secs_f64() * 1e3);
                dump(&h, &format!("ramp-{}", done));
            }
        }
    }
    println!("[ramp] 512 chunks total wall={:.0}ms", t.elapsed().as_secs_f64() * 1e3);
    olb_watch(false);
    println!("=== done ===");
}
