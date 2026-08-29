// xoroshiro.rs — XoroshiroRandom + Xoroshiro128PlusPlus + Splitter（从 random.h / xoroshiro.h 移植）
// create_xoroshiro_seed_str 的 md5 暂为 stub（后续从 md5.h 移植完整 md5）。

fn rotl(x: u64, c: u32) -> u64 { (x << c) | (x >> (64 - c)) }

pub fn mix_stafford13(seed: u64) -> u64 {
    let mut s = seed;
    s = (s ^ (s >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    s = (s ^ (s >> 27)).wrapping_mul(0x94D049BB133111EB);
    s ^ (s >> 31)
}

#[derive(Clone, Copy)]
pub struct XoroshiroSeed { pub seed_lo: u64, pub seed_hi: u64 }
impl XoroshiroSeed {
    pub fn split(&self, lo: u64, hi: u64) -> XoroshiroSeed {
        XoroshiroSeed { seed_lo: self.seed_lo ^ lo, seed_hi: self.seed_hi ^ hi }
    }
    pub fn mix(&self) -> XoroshiroSeed {
        XoroshiroSeed { seed_lo: mix_stafford13(self.seed_lo), seed_hi: mix_stafford13(self.seed_hi) }
    }
}

fn create_unmixed_xoroshiro_seed(seed: u64) -> XoroshiroSeed {
    let lo = seed ^ 0x6A09E667F3BCC909;
    let hi = lo.wrapping_add(0x9E3779B97F4A7C15);
    XoroshiroSeed { seed_lo: lo, seed_hi: hi }
}
pub fn create_xoroshiro_seed(seed: u64) -> XoroshiroSeed {
    create_unmixed_xoroshiro_seed(seed).mix()
}
// md5(str) -> lo/hi（Java md5 + Longs.fromBytes big-endian）
pub fn create_xoroshiro_seed_str(seed: &str) -> XoroshiroSeed {
    let (lo, hi) = crate::md5::md5_lo_hi(seed);
    XoroshiroSeed { seed_lo: lo, seed_hi: hi }
}

#[derive(Clone)]
pub struct Xoroshiro128PlusPlus { pub seed_lo: u64, pub seed_hi: u64 }
impl Xoroshiro128PlusPlus {
    pub fn new(lo: u64, hi: u64) -> Self {
        let (mut l, mut h) = (lo, hi);
        if (l | h) == 0 { l = 0x9E3779B97F4A7C15; h = 0x6A09E667F3BCC909; }
        Xoroshiro128PlusPlus { seed_lo: l, seed_hi: h }
    }
    pub fn next(&mut self) -> u64 {
        let lo = self.seed_lo;
        let hi = self.seed_hi;
        let n = rotl(lo.wrapping_add(hi), 17).wrapping_add(lo);
        let hi2 = hi ^ lo;
        self.seed_lo = rotl(lo, 49) ^ hi2 ^ (hi2 << 21);
        self.seed_hi = rotl(hi2, 28);
        n
    }
}

// MathHelper.hashCode(x,y,z)——1.20.1 long 版（xoroshiro.h L13-23）
pub fn hash_xyz(x: i32, y: i32, z: i32) -> i64 {
    let xi = ((x as u32).wrapping_mul(3129871u32)) as i32;
    let l = (xi as i64) ^ ((z as i64) * 116129781i64) ^ (y as i64);
    let mut u = l as u64;
    u = u.wrapping_mul(u).wrapping_mul(42317861).wrapping_add(u.wrapping_mul(11));
    (u as i64) >> 16
}

#[derive(Clone)]
pub struct XoroshiroRandom { impl_pp: Xoroshiro128PlusPlus }
impl XoroshiroRandom {
    pub fn new(seed: u64) -> Self {
        let s = create_xoroshiro_seed(seed);
        XoroshiroRandom { impl_pp: Xoroshiro128PlusPlus::new(s.seed_lo, s.seed_hi) }
    }
    pub fn new2(lo: u64, hi: u64) -> Self { XoroshiroRandom { impl_pp: Xoroshiro128PlusPlus::new(lo, hi) } }
    pub fn new_seed(s: &XoroshiroSeed) -> Self { XoroshiroRandom { impl_pp: Xoroshiro128PlusPlus::new(s.seed_lo, s.seed_hi) } }
    pub fn next(&mut self) -> u64 { self.impl_pp.next() }
    pub fn next_int(&mut self) -> i32 { self.impl_pp.next() as i32 }
    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        let mut l = (self.next_int() as u32) as u64;
        let mut m = l.wrapping_mul(bound as u64);
        let mut n = m & 0xFFFFFFFF;
        if n < bound as u64 {
            let rem = ((!(bound as u32)).wrapping_add(1u32)) % (bound as u32);
            while n < rem as u64 {
                l = (self.next_int() as u32) as u64;
                m = l.wrapping_mul(bound as u64);
                n = m & 0xFFFFFFFF;
            }
        }
        (m >> 32) as i32
    }
    // C++ nextDouble：(double)((uint64)(impl.next()>>11) * 1.110223E-16F)——uint64 转 float（精度损失）+ float 乘法。
    // Rust 若用 f64 常量会差（更精确≠对齐）；复刻 C++/Java 的 float 数学（Java nextDouble = next(53)*1.110223E-16F）
    pub fn next_double(&mut self) -> f64 {
        (((self.impl_pp.next() >> 11) as f32) * 1.110223E-16f32) as f64
    }
    pub fn next_float(&mut self) -> f32 { ((self.impl_pp.next() >> 40) as f32) * 5.9604645E-8 }
    pub fn skip(&mut self, count: i64) { for _ in 0..count { self.impl_pp.next(); } }
    pub fn split(&mut self) -> XoroshiroRandom {
        let a = self.impl_pp.next();
        let b = self.impl_pp.next();
        XoroshiroRandom::new2(a, b)
    }
    pub fn next_splitter(&mut self) -> XoroshiroSplitter {
        let a = self.impl_pp.next();
        let b = self.impl_pp.next();
        XoroshiroSplitter { seed_lo: a, seed_hi: b }
    }
}

#[derive(Clone)]
pub struct XoroshiroSplitter { seed_lo: u64, seed_hi: u64 }
impl XoroshiroSplitter {
    pub fn split_xyz(&self, x: i32, y: i32, z: i32) -> XoroshiroRandom {
        let l = hash_xyz(x, y, z);
        let m = (l as u64) ^ self.seed_lo;
        XoroshiroRandom::new2(m, self.seed_hi)
    }
    pub fn split_str(&self, seed: &str) -> XoroshiroRandom {
        let s = create_xoroshiro_seed_str(seed);
        let sp = s.split(self.seed_lo, self.seed_hi);
        XoroshiroRandom::new_seed(&sp)
    }
}
