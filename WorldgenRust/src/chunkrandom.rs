// chunkrandom.rs — ChunkRandom + CheckedRandom（MC 1.20.1 移植，CARVER 种子派生）
// 语义来源：C++ versions/1.20.1/cpp/worldgen/src/chunkrandom.h
//   - CheckedRandom.java（48 位 LCG，java.util.Random 算法）
//   - ChunkRandom.java（包装 baseRandom，next(bits) 按基类类型分发）
//   - BaseRandom.java（nextLong/nextInt(bound)/nextFloat/nextDouble 默认实现）
// 关键易错点（MC-239059）：BaseRandom.nextLong() = (long)next(32) << 32 + next(32)
//   —— i/j 都是 int 符号扩展后做有符号加法，j<0 时高 32 位被 0xFFFFFFFF 填充
//   （非无符号位拼接！）。setCarverSeed 的 nextLong 走 ChunkRandom.next(bits)，
//   CheckedRandom 基类下 = 每次消费 1 轮 LCG 的高 32 位（共 2 轮）。

use crate::xoroshiro::Xoroshiro128PlusPlus;

// CheckedRandom（48 位 LCG）——CARVERS 阶段 ChunkRandom 的基类
#[derive(Clone)]
pub struct CheckedRandom {
    seed: u64,
}

const MULTIPLIER: u64 = 25214903917;
const INCREMENT: u64 = 11;
const SEED_MASK: u64 = 281474976710655; // (1<<48)-1

impl CheckedRandom {
    pub fn new(seed: i64) -> Self {
        let mut r = CheckedRandom { seed: 0 };
        r.set_seed(seed);
        r
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = ((seed as u64) ^ MULTIPLIER) & SEED_MASK;
    }

    // Java next(int bits)：seed = seed*M + 11 & MASK；返回 (int)(seed >> 48-bits)
    pub fn next(&mut self, bits: i32) -> i32 {
        self.seed = (self.seed.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT)) & SEED_MASK;
        (self.seed >> (48 - bits)) as i32
    }

    // BaseRandom.nextLong()：(long)next(32) << 32 + next(32)（有符号拼接，MC-239059）
    pub fn next_long(&mut self) -> i64 {
        let i = self.next(32);
        let j = self.next(32);
        ((i as i64) << 32) + (j as i64)
    }

    // BaseRandom.nextInt(bound)（默认实现）：幂 2 用 next(31)，否则拒绝采样
    pub fn next_int(&mut self, bound: i32) -> i32 {
        debug_assert!(bound > 0, "Bound must be positive");
        if (bound & (bound - 1)) == 0 {
            return (((bound as i64) * (self.next(31) as i64)) >> 31) as i32;
        }
        let mut i: i32;
        let mut j: i32;
        loop {
            i = self.next(31);
            j = i % bound;
            // Java int 回绕（无符号模拟防 UB）：(int)((uint32)i - (uint32)j + (uint32)(bound-1)) < 0
            let check = (i as u32).wrapping_sub(j as u32).wrapping_add((bound - 1) as u32) as i32;
            if check >= 0 { break; }
        }
        j
    }

    pub fn next_float(&mut self) -> f32 {
        (self.next(24) as f32) * 5.9604645E-8
    }
}

// ChunkRandom：包装基类（CheckedRandom=LCG 或 Xoroshiro128PlusPlus）
// next(bits)：基类为 CheckedRandom → checkedRandom.next(bits)（LCG）
//             基类为 Xoroshiro → (int)(baseRandom.nextLong() >>> 64-bits)（高 bits 位）
// 其余 nextInt/nextLong/nextFloat/nextBoolean/nextDouble 走 BaseRandom 默认实现
#[derive(Clone)]
pub enum ChunkRandom {
    Checked(CheckedRandom),
    Xoroshiro(Xoroshiro128PlusPlus),
}

impl ChunkRandom {
    pub fn checked() -> Self { ChunkRandom::Checked(CheckedRandom::new(0)) }
    pub fn xoroshiro() -> Self { ChunkRandom::Xoroshiro(Xoroshiro128PlusPlus::new(0, 0)) }

    pub fn set_seed(&mut self, seed: i64) {
        match self {
            ChunkRandom::Checked(r) => r.set_seed(seed),
            ChunkRandom::Xoroshiro(r) => {
                let s = crate::xoroshiro::create_xoroshiro_seed(seed as u64);
                r.seed_lo = s.seed_lo;
                r.seed_hi = s.seed_hi;
            }
        }
    }

    // Java ChunkRandom.next(bits)
    pub fn next(&mut self, bits: i32) -> i32 {
        match self {
            ChunkRandom::Checked(r) => r.next(bits),
            // (int)(baseRandom.nextLong() >>> 64 - bits)——Xoroshiro nextLong = 完整 64 位，取高 bits 位
            ChunkRandom::Xoroshiro(r) => (r.next() >> (64 - bits)) as i32,
        }
    }

    // BaseRandom.nextLong()：(long)next(32) << 32 + next(32)
    pub fn next_long(&mut self) -> i64 {
        let i = self.next(32);
        let j = self.next(32);
        ((i as i64) << 32) + (j as i64) // 有符号拼接（j 符号扩展相加，MC-239059）
    }

    pub fn next_int(&mut self) -> i32 { self.next(32) }

    // BaseRandom.nextInt(bound)：幂 2 用 (int)((long)bound * next(31) >> 31)，否则拒绝采样
    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        debug_assert!(bound > 0, "Bound must be positive");
        if (bound & (bound - 1)) == 0 {
            return (((bound as i64) * (self.next(31) as i64)) >> 31) as i32;
        }
        let mut i: i32;
        let mut j: i32;
        loop {
            i = self.next(31);
            j = i % bound;
            let check = (i as u32).wrapping_sub(j as u32).wrapping_add((bound - 1) as u32) as i32;
            if check >= 0 { break; }
        }
        j
    }

    pub fn next_boolean(&mut self) -> bool { self.next(1) != 0 }

    // BaseRandom.nextFloat() = next(24) * 5.9604645E-8F（float 乘法）
    pub fn next_float(&mut self) -> f32 { (self.next(24) as f32) * 5.9604645E-8 }

    // BaseRandom.nextDouble() = ((long)next(26) << 27 + next(27)) * 1.110223E-16F
    // Java 语义：long * float 是 float 乘法（精度截断），结果提升回 double——用 float 模拟
    pub fn next_double(&mut self) -> f64 {
        let i = self.next(26);
        let j = self.next(27);
        let l = ((i as i64) << 27) + (j as i64);
        ((l as f32) * 1.110223E-16f32) as f64
    }

    // ---- 种子派生（ChunkRandom.java）----

    // setPopulationSeed(worldSeed, blockX, blockZ)：FEATURES 阶段
    //   setSeed(worldSeed); l=nextLong()|1; m=nextLong()|1; n=blockX*l + blockZ*m ^ worldSeed; setSeed(n)
    pub fn set_population_seed(&mut self, world_seed: i64, block_x: i32, block_z: i32) -> i64 {
        self.set_seed(world_seed);
        let l = self.next_long() | 1;
        let m = self.next_long() | 1;
        let n = ((block_x as i64) * l + (block_z as i64) * m) ^ world_seed;
        self.set_seed(n);
        n
    }

    // setDecoratorSeed(populationSeed, index, step)：l = populationSeed + index + 10000*step
    pub fn set_decorator_seed(&mut self, population_seed: i64, index: i32, step: i32) {
        let l = population_seed + (index as i64) + 10000i64 * (step as i64);
        self.set_seed(l);
    }

    // setCarverSeed(worldSeed, chunkX, chunkZ)：CARVERS 阶段
    //   setSeed(worldSeed); l=nextLong(); m=nextLong(); n=chunkX*l ^ chunkZ*m ^ worldSeed; setSeed(n)
    pub fn set_carver_seed(&mut self, world_seed: i64, chunk_x: i32, chunk_z: i32) {
        self.set_seed(world_seed);
        let l = self.next_long();
        let m = self.next_long();
        let n = ((chunk_x as i64) * l) ^ ((chunk_z as i64) * m) ^ world_seed;
        self.set_seed(n);
    }
}
