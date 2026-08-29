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

    // === ChunkRandom(Xoroshiro base) setPopulationSeed/setDecoratorSeed ===
    // C++ 参照（chunkrandom_test.cpp）：worldSeed, blockX=720*16, blockZ=-432*16
    let mut crx = ChunkRandom::xoroshiro();
    let pop = crx.set_population_seed(world_seed, 720 * 16, -432 * 16);
    check!("populationSeed", pop, -3665859634238804548);
    check!("afterPop.nextLong#1", crx.next_long(), -7508349385403582096);
    check!("afterPop.nextLong#2", crx.next_long(), -5481884486643468655);
    check!("afterPop.nextInt(256)", crx.next_int_bound(256) as i64, 7);
    let nfd = crx.next_float();
    if f2u(nfd) == f2u(0.49389488) { println!("[OK] afterPop.nextFloat bits = 0x{:08X}", f2u(nfd)); }
    else { println!("[FAIL] afterPop.nextFloat bits = 0x{:08X} (want 0x{:08X})", f2u(nfd), f2u(0.49389488)); failures += 1; }
    // setDecoratorSeed(step, index)
    let mut deco = ChunkRandom::xoroshiro();
    deco.set_population_seed(world_seed, 720 * 16, -432 * 16);
    deco.set_decorator_seed(pop, 0, 0);
    check!("deco(0,0).nextLong", deco.next_long(), -7508349385403582096);
    let mut deco1 = ChunkRandom::xoroshiro();
    deco1.set_population_seed(world_seed, 720 * 16, -432 * 16);
    deco1.set_decorator_seed(pop, 1, 0);
    check!("deco(1,0).nextLong", deco1.next_long(), -3766154493263439697);
    let mut deco2 = ChunkRandom::xoroshiro();
    deco2.set_population_seed(world_seed, 720 * 16, -432 * 16);
    deco2.set_decorator_seed(pop, 2, 0);
    check!("deco(2,0).nextLong", deco2.next_long(), 1940938260340561462);

    println!("=== {} (failures={}) ===", if failures == 0 { "ALL PASS" } else { "FAILED" }, failures);
}
