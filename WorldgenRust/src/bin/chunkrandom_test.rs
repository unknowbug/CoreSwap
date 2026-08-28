// chunkrandom_test.rs — 验证 Rust CheckedRandom/ChunkRandom 对齐 C++ chunkrandom_probe_run1.txt
// C++ 参照（chunkrandom_probe_run1.txt）：
//   cr.next(32)=-1045225129, cr.next(32)=1084206043, cr.nextLong=-3933948616470016951
//   carver(-18,-15) nextFloat=0.5614767, nextInt(16)=12, nextInt(16)=11, nextInt(16)=2
use WorldgenRust::chunkrandom::{CheckedRandom, ChunkRandom};

fn main() {
    // CheckedRandom(0) 序列
    let mut cr = CheckedRandom::new(0);
    println!("cr.next(32)={}", cr.next(32));
    println!("cr.next(32)={}", cr.next(32));
    println!("cr.nextLong={}", cr.next_long());
    println!("cr.nextLong={}", cr.next_long());
    println!("cr.nextInt(10)={}", cr.next_int(10));
    println!("cr.nextInt(10)={}", cr.next_int(10));

    // ChunkRandom(CheckedRandom base) setCarverSeed(-18,-15)
    // 需要知道 worldSeed。C++ probe 用 seed=-8248318472910187742? 检查 chunkrandom_probe 的 seed。
    // 这里用 seed=0 验证 setCarverSeed 序列（与 C++ 同 seed 对比）
    let mut cr2 = ChunkRandom::checked();
    cr2.set_carver_seed(0, -18, -15);
    println!("carver(0,-18,-15) nextFloat={:.7}", cr2.next_float());
    println!("carver(0,-18,-15) nextInt(16)={}", cr2.next_int_bound(16));
    println!("carver(0,-18,-15) nextInt(16)={}", cr2.next_int_bound(16));
    println!("carver(0,-18,-15) nextInt(16)={}", cr2.next_int_bound(16));
}
