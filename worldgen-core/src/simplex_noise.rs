// simplex_noise.rs — SimplexNoiseSampler + EndIslands 密度函数（end 维度接管，260906-04）
// Java 参照（yarn 1.20.1，mc_src_extract/）：
//   - net/minecraft/util/math/noise/SimplexNoiseSampler.java（全文，D1 逐行移植）
//   - net/minecraft/world/gen/densityfunction/DensityFunctionTypes.java L626-682（EndIslands）
//   - net/minecraft/util/math/random/Random.java L101-105（skip = count × nextInt()）
// 种子链（02 篇 §2.4）：EndIslands 用 worldSeed 直传 CheckedRandom，无 randomDeriver.split，
//   构造前 skip(17292)（DensityFunctionTypes.java L631-635；NoiseConfig.java L105）。

use crate::chunkrandom::CheckedRandom;

// ---------- SimplexNoiseSampler（SimplexNoiseSampler.java 全文移植） ----------

const GRADIENTS: [[i32; 3]; 16] = [
    [1, 1, 0], [-1, 1, 0], [1, -1, 0], [-1, -1, 0],
    [1, 0, 1], [-1, 0, 1], [1, 0, -1], [-1, 0, -1],
    [0, 1, 1], [0, -1, 1], [0, 1, -1], [0, -1, -1],
    [1, 1, 0], [0, -1, 1], [-1, 1, 0], [0, -1, -1],
];

const SQRT_3: f64 = 1.7320508075688772;
const SKEW_FACTOR_2D: f64 = 0.5 * (SQRT_3 - 1.0);
const UNSKEW_FACTOR_2D: f64 = (3.0 - SQRT_3) / 6.0;

pub struct SimplexNoiseSampler {
    permutation: [i32; 256],
    pub origin_x: f64,
    pub origin_y: f64,
    pub origin_z: f64,
}

// MathHelper.floor(double) = (int)Math.floor
#[inline]
fn mh_floor(v: f64) -> i32 { v.floor() as i32 }

impl SimplexNoiseSampler {
    // 构造消费 Random 序列（L33-49）：originX/Y/Z = nextDouble()*256（各 1 次），
    // 然后 Fisher-Yates：j = nextInt(256 - ix)，swap permutation[ix] <-> permutation[j+ix]。
    // Java 入参是 Random 接口；end 链路固定 CheckedRandom，先做具体类型（将来需要再泛化）。
    pub fn new(random: &mut CheckedRandom) -> Self {
        let origin_x = random.next_double() * 256.0;
        let origin_y = random.next_double() * 256.0;
        let origin_z = random.next_double() * 256.0;
        let mut permutation = [0i32; 256];
        let mut i = 0usize;
        while i < 256 {
            permutation[i] = i as i32;
            i += 1;
        }
        for ix in 0..256usize {
            let j = random.next_int(256 - ix as i32) as usize;
            let k = permutation[ix];
            permutation[ix] = permutation[j + ix];
            permutation[j + ix] = k;
        }
        SimplexNoiseSampler { permutation, origin_x, origin_y, origin_z }
    }

    // map（L51-53）：permutation[input & 0xFF]
    #[inline]
    fn map(&self, input: i32) -> i32 { self.permutation[(input & 0xFF) as usize] }

    // 对拍只读取置换表项（索引已由调用方保证 < 256）
    pub fn perm_at(&self, i: usize) -> i32 { self.permutation[i] }

    // grad（L59-70）
    #[inline]
    fn grad(&self, hash: i32, x: f64, y: f64, z: f64, distance: f64) -> f64 {
        let g = GRADIENTS[hash as usize];
        let mut d = distance - x * x - y * y - z * z;
        if d < 0.0 { 0.0 } else {
            d *= d;
            d * d * (g[0] as f64 * x + g[1] as f64 * y + g[2] as f64 * z)
        }
    }

    // 2D sample（L72-104）——EndIslands 只用此入口
    pub fn sample(&self, x: f64, y: f64) -> f64 {
        let d = (x + y) * SKEW_FACTOR_2D;
        let i = mh_floor(x + d);
        let j = mh_floor(y + d);
        let e = (i + j) as f64 * UNSKEW_FACTOR_2D;
        let f = i as f64 - e;
        let g = j as f64 - e;
        let h = x - f;
        let k = y - g;
        let (l, m): (i32, i32) = if h > k { (1, 0) } else { (0, 1) };
        let n = h - l as f64 + UNSKEW_FACTOR_2D;
        let o = k - m as f64 + UNSKEW_FACTOR_2D;
        let p = h - 1.0 + 2.0 * UNSKEW_FACTOR_2D;
        let q = k - 1.0 + 2.0 * UNSKEW_FACTOR_2D;
        let r = i & 0xFF;
        let s = j & 0xFF;
        let t = self.map(r + self.map(s)) % 12;
        let u = self.map(r + l + self.map(s + m)) % 12;
        let v = self.map(r + 1 + self.map(s + 1)) % 12;
        let w = self.grad(t, h, k, 0.0, 0.5);
        let z = self.grad(u, n, o, 0.0, 0.5);
        let aa = self.grad(v, p, q, 0.0, 0.5);
        70.0 * (w + z + aa)
    }

    // 3D sample（L106-193）——当前 end 链路不用，完整移植备用（防半成品误导后续复用）
    pub fn sample_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let e = (x + y + z) * 0.333_333_333_333_333_3;
        let i = mh_floor(x + e);
        let j = mh_floor(y + e);
        let k = mh_floor(z + e);
        let g = (i + j + k) as f64 * 0.166_666_666_666_666_66;
        let h = i as f64 - g;
        let l = j as f64 - g;
        let m = k as f64 - g;
        let n = x - h;
        let o = y - l;
        let p = z - m;
        let (q, r, s, t, u, v): (i32, i32, i32, i32, i32, i32);
        if n >= o {
            if o >= p { (q, r, s, t, u, v) = (1, 0, 0, 1, 1, 0); }
            else if n >= p { (q, r, s, t, u, v) = (1, 0, 0, 1, 0, 1); }
            else { (q, r, s, t, u, v) = (0, 0, 1, 1, 0, 1); }
        } else if o < p { (q, r, s, t, u, v) = (0, 0, 1, 0, 1, 1); }
        else if n < p { (q, r, s, t, u, v) = (0, 1, 0, 0, 1, 1); }
        else { (q, r, s, t, u, v) = (0, 1, 0, 1, 1, 0); }
        let w = n - q as f64 + 0.166_666_666_666_666_66;
        let aa = o - r as f64 + 0.166_666_666_666_666_66;
        let ab = p - s as f64 + 0.166_666_666_666_666_66;
        let ac = n - t as f64 + 0.333_333_333_333_333_3;
        let ad = o - u as f64 + 0.333_333_333_333_333_3;
        let ae = p - v as f64 + 0.333_333_333_333_333_3;
        let af = n - 1.0 + 0.5;
        let ag = o - 1.0 + 0.5;
        let ah = p - 1.0 + 0.5;
        let ai = i & 0xFF;
        let aj = j & 0xFF;
        let ak = k & 0xFF;
        let al = self.map(ai + self.map(aj + self.map(ak))) % 12;
        let am = self.map(ai + q + self.map(aj + r + self.map(ak + s))) % 12;
        let an = self.map(ai + t + self.map(aj + u + self.map(ak + v))) % 12;
        let ao = self.map(ai + 1 + self.map(aj + 1 + self.map(ak + 1))) % 12;
        let ap = self.grad(al, n, o, p, 0.6);
        let aq = self.grad(am, w, aa, ab, 0.6);
        let ar = self.grad(an, ac, ad, ae, 0.6);
        let as_ = self.grad(ao, af, ag, ah, 0.6);
        32.0 * (ap + aq + ar + as_)
    }
}

// ---------- EndIslands（DensityFunctionTypes.java L626-682 逐行移植） ----------

pub struct EndIslandsNoise {
    sampler: SimplexNoiseSampler,
}

// L628 field_37677 = -0.9F。Java 比较 `sampler.sample(o,p) < -0.9F` 中 sample 返回 double、
// -0.9F 提升为 double（= -0.89999997615814208984375，非精确 -0.9）——必须用 f32→f64 提升值。
const ISLAND_THRESHOLD: f64 = (-0.9f32) as f64;

// MathHelper.sqrt(float f) = (float)Math.sqrt((double)f)——先 f64 sqrt 再截 f32（非 f32 sqrt）
#[inline]
fn mh_sqrt_f32(v: f32) -> f32 { (v as f64).sqrt() as f32 }

// MathHelper.clamp(float, float, float)
#[inline]
fn clamp_f32(v: f32, mn: f32, mx: f32) -> f32 { v.max(mn).min(mx) }

impl EndIslandsNoise {
    // L631-635：CheckedRandom(worldSeed) + skip(17292) + SimplexNoiseSampler
    pub fn new(world_seed: i64) -> Self {
        let mut random = CheckedRandom::new(world_seed);
        random.skip(17292);
        EndIslandsNoise { sampler: SimplexNoiseSampler::new(&mut random) }
    }

    // L637-661 私有静态 sample：float 域计算（f32 语义逐行对齐）
    fn sample_impl(&self, x: i32, z: i32) -> f64 {
        let i = x / 2; // Java int 除法截断向零（Rust 同语义）
        let j = z / 2;
        let k = x % 2;
        let l = z % 2;
        let mut f: f32 = 100.0 - mh_sqrt_f32(((x as i64 * x as i64 + z as i64 * z as i64) as i32) as f32) * 8.0;
        // 注：x*x + z*z Java 是 int 运算（可能溢出回绕）；i64 中转再截 int 复刻回绕语义。
        // 实际调用域 x,z = 块坐标/8，|x|,|z| ≤ 2^28/8，x*x+z*z < 2^31 无溢出，两式等价。
        f = clamp_f32(f, -100.0, 80.0);
        for m in -12i64..=12 {
            for n in -12i64..=12 {
                let o = i as i64 + m;
                let p = j as i64 + n;
                if o * o + p * p > 4096 && self.sampler.sample(o as f64, p as f64) < ISLAND_THRESHOLD {
                    // L650：float 域 (|o|*3439 + |p|*147) % 13 + 9（Java float %）
                    let g = ((o.wrapping_abs() as f32) * 3439.0f32 + (p.wrapping_abs() as f32) * 147.0f32) % 13.0f32 + 9.0f32;
                    let h = k as f32 - (m * 2) as f32;
                    let q = l as f32 - (n * 2) as f32;
                    let mut r: f32 = 100.0 - mh_sqrt_f32(h * h + q * q) * g;
                    r = clamp_f32(r, -100.0, 80.0);
                    f = f.max(r);
                }
            }
        }
        f as f64
    }

    // L663-666：pos.blockX()/8, pos.blockZ()/8（int 截断除法），输出 (f - 8.0)/128.0
    pub fn sample(&self, block_x: i32, block_z: i32) -> f64 {
        (self.sample_impl(block_x / 8, block_z / 8) as f64 - 8.0) / 128.0
    }

    pub const MIN_VALUE: f64 = -0.84375;
    pub const MAX_VALUE: f64 = 0.5625;
}
