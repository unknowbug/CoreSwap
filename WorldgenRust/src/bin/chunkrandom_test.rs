// chunkrandom_test.rs — 验证 Rust CheckedRandom/ChunkRandom 对齐 C++ chunkrandom_test.cpp
// C++ 参照（chunkrandom_test.cpp，worldSeed=8576294172403134396）：
//   cr.next(32)=-1045225129, cr.next(32)=1084206043, cr.nextLong=-3933948616470016951
//   carver(-18,-15) nextFloat=0.5614767, nextInt(16)=12, nextInt(16)=11, nextInt(16)=2
use WorldgenRust::chunkrandom::{CheckedRandom, ChunkRandom};

fn f2u(f: f32) -> u32 { f.to_bits() }

fn main() {
    let world_seed: i64 = 8576294172403134396;
    let mut failures = 0;

    // CheckedRandom(worldSeed) 序列
    let mut cr = CheckedRandom::new(world_seed);
    macro_rules! check {
        ($name:expr, $got:expr, $want:expr) => {
            let g = $got as i64; let w = $want as i64;
            if g == w { println!("[OK] {} = {}", $name, g); }
            else { println!("[FAIL] {} = {} (want {})", $name, g, w); failures += 1; }
        };
    }
    check!("cr.next(32)#1", cr.next(32), -1045225129);
    check!("cr.next(32)#2", cr.next(32), 1084206043);
    check!("cr.nextLong#1", cr.next_long(), -3933948616470016951);
    check!("cr.nextLong#2", cr.next_long(), -518819946905544879);
    check!("cr.nextInt(10)#1", cr.next_int(10), 4);
    check!("cr.nextInt(10)#2", cr.next_int(10), 6);

    // ChunkRandom(CheckedRandom base) setCarverSeed(worldSeed, -18, -15)
    let mut crc = ChunkRandom::checked();
    crc.set_carver_seed(world_seed, -18, -15);
    let nf = crc.next_float();
    if f2u(nf) == f2u(0.5614767) { println!("[OK] carver nextFloat bits = 0x{:08X}", f2u(nf)); }
    else { println!("[FAIL] carver nextFloat bits = 0x{:08X} (want 0x{:08X})", f2u(nf), f2u(0.5614767)); failures += 1; }
    check!("carver nextInt(16)#1", crc.next_int_bound(16), 12);
    check!("carver nextInt(16)#2", crc.next_int_bound(16), 11);
    check!("carver nextInt(16)#3", crc.next_int_bound(16), 2);

    // 重新 setCarverSeed 验证确定性
    let mut crc2 = ChunkRandom::checked();
    crc2.set_carver_seed(world_seed, -18, -15);
    let nf2 = crc2.next_float();
    if f2u(nf2) == f2u(0.5614767) { println!("[OK] carver2 nextFloat bits = 0x{:08X}", f2u(nf2)); }
    else { println!("[FAIL] carver2 nextFloat bits = 0x{:08X} (want 0x{:08X})", f2u(nf2), f2u(0.5614767)); failures += 1; }
    check!("carver2 nextLong", crc2.next_long(), -3711936206981428316);

    println!("=== {} (failures={}) ===", if failures == 0 { "ALL PASS" } else { "FAILED" }, failures);
}
