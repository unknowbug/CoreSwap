// legacy_random.rs — Java LegacyRandomSource（yarn）= MC `Random`（Mojang 名，CheckedRandom 为其并发检查孪生）移植。
// 源码依据：mc_src_extract/net/minecraft/util/math/random/{CheckedRandom,BaseRandom,RandomSplitter}.java（2026-08-30）。
// 背景（M6）：nether.json "legacy_random_source": true → 噪声种子派生 + surface 概率 roll 全走 LCG，
// Rust 原全链 Xoroshiro → 下界大面积错位（C++ surface.h 同病，未修）。
//
// LCG（CheckedRandom L13-15/L34-51）：seed = (seed * 25214903917 + 11) & (2^48-1)；
// setSeed = (seed ^ 25214903917) & (2^48-1)；next(bits) = seed >> (48-bits)。
// Splitter（CheckedRandom.Splitter L58-83 = LegacyPositionalRandomFactory）：
//   split(x,y,z) = new Legacy(hashCode(x,y,z) ^ factory_seed)
//   split(String) = new Legacy(name.hashCode() ^ factory_seed)
// BaseRandom 默认语义：nextLong = (next32<<32)+next32；nextFloat = next(24)*5.9604645E-8F；
//   nextDouble = (next26<<27 + next27) * 1.110223E-16F（float 乘法再升 double——精度对齐）。

/// Java String.hashCode（s[0]*31^(n-1) + ...，i32 环绕）
pub fn java_string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(c as i32);
    }
    h
}

#[derive(Clone)]
pub struct LegacyRandom { seed: u64 }

impl LegacyRandom {
    pub fn new(seed: i64) -> Self {
        let mut r = LegacyRandom { seed: 0 };
        r.set_seed(seed);
        r
    }
    pub fn set_seed(&mut self, seed: i64) {
        self.seed = ((seed as u64) ^ 25214903917) & 281474976710655;
    }
    pub fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(25214903917).wrapping_add(11) & 281474976710655;
        (self.seed >> (48 - bits)) as i32
    }
    pub fn next_int(&mut self) -> i32 { self.next(32) }
    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        // BaseRandom.nextInt(bound)：幂次 = (bound * next(31)) >> 31；非幂次 rejection
        if bound <= 0 { panic!("bound must be positive"); }
        if bound & (bound - 1) == 0 {
            return (((bound as i64) * (self.next(31) as i64)) >> 31) as i32;
        }
        let mut i; let mut j;
        loop {
            i = self.next(31);
            j = i % bound;
            if i.wrapping_sub(j).wrapping_add(bound - 1) >= 0 { break; }
        }
        j
    }
    pub fn next_long(&mut self) -> i64 {
        let i = self.next(32);
        let j = self.next(32);
        ((i as i64) << 32).wrapping_add(j as i64)
    }
    pub fn next_boolean(&mut self) -> bool { self.next(1) != 0 }
    pub fn next_float(&mut self) -> f32 { self.next(24) as f32 * 5.9604645E-8f32 }
    pub fn next_double(&mut self) -> f64 {
        let i = self.next(26);
        let j = self.next(27);
        let l = ((i as i64) << 27) + j as i64;
        // Java: l * 1.110223E-16F（float 乘法，结果转 double——精度对齐 XoroshiroRandom::next_double 同款）
        (l as f32 * 1.110223E-16f32) as f64
    }
    pub fn split(&mut self) -> LegacyRandom { LegacyRandom::new_seed(self.next_long()) }
    pub fn next_splitter(&mut self) -> LegacySplitter { LegacySplitter { seed: self.next_long() } }
    pub fn new_seed(seed: i64) -> Self { LegacyRandom { seed: 0 }.init(seed) }
    fn init(mut self, seed: i64) -> Self { self.set_seed(seed); self }
}

/// LegacyPositionalRandomFactory（= CheckedRandom.Splitter）
#[derive(Clone)]
pub struct LegacySplitter { pub seed: i64 }
impl LegacySplitter {
    pub fn split_xyz(&self, x: i32, y: i32, z: i32) -> LegacyRandom {
        // MathHelper.hashCode(x,y,z)（long 版）——与 xoroshiro.rs hash_xyz 同源
        let l = crate::xoroshiro::hash_xyz(x, y, z);
        LegacyRandom::new_seed(l ^ self.seed)
    }
    pub fn split_str(&self, seed: &str) -> LegacyRandom {
        LegacyRandom::new_seed((java_string_hash(seed) as i64) ^ self.seed)
    }
    pub fn next_splitter(&self) -> LegacySplitter { self.clone() }
}

// ===== 统一随机源枚举（overworld=Xoroshiro / legacy 维度=Legacy）=====
// 让 DensityBuilder / NoiseSet / surface 规则对上游透明：构造期分流，运行期同接口转发。
#[derive(Clone)]
pub enum RsRandom {
    Xoro(crate::xoroshiro::XoroshiroRandom),
    Legacy(LegacyRandom),
}
impl RsRandom {
    pub fn next(&mut self) -> u64 {
        match self { RsRandom::Xoro(r) => r.next(), RsRandom::Legacy(r) => r.next_long() as u64 }
    }
    pub fn next_int(&mut self) -> i32 {
        match self { RsRandom::Xoro(r) => r.next_int(), RsRandom::Legacy(r) => r.next_int() }
    }
    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        match self { RsRandom::Xoro(r) => r.next_int_bound(bound), RsRandom::Legacy(r) => r.next_int_bound(bound) }
    }
    pub fn next_double(&mut self) -> f64 {
        match self { RsRandom::Xoro(r) => r.next_double(), RsRandom::Legacy(r) => r.next_double() }
    }
    pub fn next_float(&mut self) -> f32 {
        match self { RsRandom::Xoro(r) => r.next_float(), RsRandom::Legacy(r) => r.next_float() }
    }
    pub fn skip(&mut self, count: i64) {
        match self { RsRandom::Xoro(r) => r.skip(count), RsRandom::Legacy(r) => { for _ in 0..count { r.next_int(); } } }
    }
    pub fn split(&mut self) -> RsRandom {
        match self { RsRandom::Xoro(r) => RsRandom::Xoro(r.split()), RsRandom::Legacy(r) => RsRandom::Legacy(r.split()) }
    }
    pub fn next_splitter(&mut self) -> RsSplitter {
        match self { RsRandom::Xoro(r) => RsSplitter::Xoro(r.next_splitter()), RsRandom::Legacy(r) => RsSplitter::Legacy(r.next_splitter()) }
    }
}

#[derive(Clone)]
pub enum RsSplitter {
    Xoro(crate::xoroshiro::XoroshiroSplitter),
    Legacy(LegacySplitter),
}
impl RsSplitter {
    pub fn split_xyz(&self, x: i32, y: i32, z: i32) -> RsRandom {
        match self { RsSplitter::Xoro(s) => RsRandom::Xoro(s.split_xyz(x, y, z)), RsSplitter::Legacy(s) => RsRandom::Legacy(s.split_xyz(x, y, z)) }
    }
    pub fn split_str(&self, seed: &str) -> RsRandom {
        match self { RsSplitter::Xoro(s) => RsRandom::Xoro(s.split_str(seed)), RsSplitter::Legacy(s) => RsRandom::Legacy(s.split_str(seed)) }
    }
}
