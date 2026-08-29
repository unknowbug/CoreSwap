// noise.rs — PerlinNoiseSampler / OctavePerlinNoiseSampler / DoublePerlinNoiseSampler
// 从 C++ noise.h 移植（逆向 Java 1.20.1）。随机源 XoroshiroRandom 在 xoroshiro.rs。
// 依赖常量/辅助函数移自 C++ noise.h L10-26。

use crate::xoroshiro::XoroshiroRandom;

// ---- 辅助（对齐 C++ noise.h）----
pub fn floor_d(v: f64) -> i32 { let i = v as i32; if v < i as f64 { i - 1 } else { i } }
pub fn lerp(d: f64, s: f64, e: f64) -> f64 { s + d * (e - s) }
pub fn perlin_fade(v: f64) -> f64 { v * v * v * (v * (v * 6.0 - 15.0) + 10.0) }

// SimplexNoiseSampler.GRADIENTS（MC 1.20.1 16 个梯度）
const GRADIENTS: [[i32; 3]; 16] = [
    [1, 1, 0], [-1, 1, 0], [1, -1, 0], [-1, -1, 0],
    [1, 0, 1], [-1, 0, 1], [1, 0, -1], [-1, 0, -1],
    [0, 1, 1], [0, -1, 1], [0, 1, -1], [0, -1, -1],
    [1, 1, 0], [0, -1, 1], [-1, 1, 0], [0, -1, -1],
];
fn dot3(g: &[i32; 3], x: f64, y: f64, z: f64) -> f64 {
    g[0] as f64 * x + g[1] as f64 * y + g[2] as f64 * z
}

// ---- PerlinNoiseSampler（C++ noise.h L29-120）----
pub struct PerlinNoiseSampler {
    origin_x: f64, origin_y: f64, origin_z: f64,
    permutation: [u8; 256],
}
impl PerlinNoiseSampler {
    pub fn new(random: &mut XoroshiroRandom) -> Self {
        let origin_x = random.next_double() * 256.0;
        let origin_y = random.next_double() * 256.0;
        let origin_z = random.next_double() * 256.0;
        let mut permutation = [0u8; 256];
        for i in 0..256 { permutation[i] = i as u8; }
        for i in 0..256 {
            let j = random.next_int_bound(256 - i as i32) as usize;
            let b = permutation[i];
            permutation[i] = permutation[i + j];
            permutation[i + j] = b;
        }
        PerlinNoiseSampler { origin_x, origin_y, origin_z, permutation }
    }
    #[inline] pub fn map(&self, input: i32) -> u8 { self.permutation[(input & 0xFF) as usize] & 0xFF }

    // AVX 实验：sample_section 的 8 个 grad dot 用 __m256d 向量化（grad 查表标量，dot 算术 SIMD）。
    // 仅在 enable_avx 时用（bench 对比）；生产仍走标量 sample_section。
    #[cfg(target_arch = "x86_64")]
    pub fn sample_section_avx(&self, sx: i32, sy: i32, sz: i32, lx: f64, ly: f64, lz: f64, fade_y: f64) -> f64 {
        #[cfg(target_feature = "avx")]
        unsafe {
            use std::arch::x86_64::*;
            let i = self.map(sx) as i32;
            let j = self.map(sx + 1) as i32;
            let k = self.map(i + sy) as i32;
            let l = self.map(i + sy + 1) as i32;
            let m = self.map(j + sy) as i32;
            let n = self.map(j + sy + 1) as i32;
            // 8 个 grad hash
            let h0 = self.map(k + sz) as i32; let h1 = self.map(m + sz) as i32;
            let h2 = self.map(l + sz) as i32; let h3 = self.map(n + sz) as i32;
            let h4 = self.map(k + sz + 1) as i32; let h5 = self.map(m + sz + 1) as i32;
            let h6 = self.map(l + sz + 1) as i32; let h7 = self.map(n + sz + 1) as i32;
            let hashes = [h0, h1, h2, h3, h4, h5, h6, h7];
            // 8 个 grad 的 (gx, gy, gz)
            let mut gx = [0f64; 8]; let mut gy = [0f64; 8]; let mut gz = [0f64; 8];
            for a in 0..8 {
                let g = GRADIENTS[(hashes[a] & 15) as usize];
                gx[a] = g[0] as f64; gy[a] = g[1] as f64; gz[a] = g[2] as f64;
            }
            // 8 个 grad 的 dot(gx*sx + gy*sy + gz*sz) 用 __m256d 并行（两个 4-lane）
            // 注意各 grad 的系数 (lx/ly/lz) 按 grad 索引不同（0-3 用 (lx,ly,lz)，4-7 用 (lx-1,ly,lz-1) 等）
            let vx = _mm256_set_pd(lx, lx, lx, lx); // 反向（set 高位-低位）——用 set1
            let vx = _mm256_set1_pd(lx);
            let vy = _mm256_set1_pd(ly);
            let vz = _mm256_set1_pd(lz);
            let _ = vx;
            // 简化：先标量算 8 个 grad 值（查表已标量，dot 先不 SIMD 以便正确性验证）
            // 这里仅测「查表后 dot 的 SIMD 潜力」——先用标量 dot 跑通流程
            let g0 = dot3(&GRADIENTS[(h0 & 15) as usize], lx, ly, lz);
            let g1 = dot3(&GRADIENTS[(h1 & 15) as usize], lx - 1.0, ly, lz);
            let g2 = dot3(&GRADIENTS[(h2 & 15) as usize], lx, ly - 1.0, lz);
            let g3 = dot3(&GRADIENTS[(h3 & 15) as usize], lx - 1.0, ly - 1.0, lz);
            let g4 = dot3(&GRADIENTS[(h4 & 15) as usize], lx, ly, lz - 1.0);
            let g5 = dot3(&GRADIENTS[(h5 & 15) as usize], lx - 1.0, ly, lz - 1.0);
            let g6 = dot3(&GRADIENTS[(h6 & 15) as usize], lx, ly - 1.0, lz - 1.0);
            let g7 = dot3(&GRADIENTS[(h7 & 15) as usize], lx - 1.0, ly - 1.0, lz - 1.0);
            let _ = (vx, vy, vz);
            let r = perlin_fade(lx); let s = perlin_fade(fade_y); let t = perlin_fade(lz);
            let x0 = lerp(r, g0, g1); let x1 = lerp(r, g2, g3);
            let x2 = lerp(r, g4, g5); let x3 = lerp(r, g6, g7);
            let y0 = lerp(s, x0, x1); let y1 = lerp(s, x2, x3);
            lerp(t, y0, y1)
        }
        #[cfg(not(target_feature = "avx"))]
        { self.sample_section(sx, sy, sz, lx, ly, lz, fade_y) }
    }
    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 { self.sample_ys(x, y, z, 0.0, 0.0) }
    pub fn sample_ys(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64) -> f64 {
        let d = x + self.origin_x;
        let e = y + self.origin_y;
        let f = z + self.origin_z;
        let i = floor_d(d);
        let j = floor_d(e);
        let k = floor_d(f);
        let g = d - i as f64;
        let h = e - j as f64;
        let l = f - k as f64;
        let n: f64;
        if y_scale != 0.0 {
            let m = if y_max >= 0.0 && y_max < h { y_max } else { h };
            n = floor_d(m / y_scale + 1.0e-7) as f64 * y_scale;
        } else {
            n = 0.0;
        }
        self.sample_section(i, j, k, g, h - n, l, h)
    }
    fn sample_section(&self, sx: i32, sy: i32, sz: i32, lx: f64, ly: f64, lz: f64, fade_y: f64) -> f64 {
        let i = self.map(sx) as i32;
        let j = self.map(sx + 1) as i32;
        let k = self.map(i + sy) as i32;
        let l = self.map(i + sy + 1) as i32;
        let m = self.map(j + sy) as i32;
        let n = self.map(j + sy + 1) as i32;
        let d = grad(self.map(k + sz) as i32, lx, ly, lz);
        let e = grad(self.map(m + sz) as i32, lx - 1.0, ly, lz);
        let f = grad(self.map(l + sz) as i32, lx, ly - 1.0, lz);
        let g = grad(self.map(n + sz) as i32, lx - 1.0, ly - 1.0, lz);
        let h = grad(self.map(k + sz + 1) as i32, lx, ly, lz - 1.0);
        let o = grad(self.map(m + sz + 1) as i32, lx - 1.0, ly, lz - 1.0);
        let p = grad(self.map(l + sz + 1) as i32, lx, ly - 1.0, lz - 1.0);
        let q = grad(self.map(n + sz + 1) as i32, lx - 1.0, ly - 1.0, lz - 1.0);
        let r = perlin_fade(lx);
        let s = perlin_fade(fade_y);
        let t = perlin_fade(lz);
        let x0 = lerp(r, d, e);
        let x1 = lerp(r, f, g);
        let x2 = lerp(r, h, o);
        let x3 = lerp(r, p, q);
        let y0 = lerp(s, x0, x1);
        let y1 = lerp(s, x2, x3);
        lerp(t, y0, y1)
    }
}
fn grad(hash: i32, x: f64, y: f64, z: f64) -> f64 {
    dot3(&GRADIENTS[(hash & 15) as usize], x, y, z)
}

// ---- OctavePerlinNoiseSampler（C++ noise.h L114-242）----
pub struct OctavePerlinNoiseSampler {
    octave_samplers: Vec<Option<PerlinNoiseSampler>>,
    first_octave: i32,
    amplitudes: Vec<f64>,
    persistence: f64,
    lacunarity: f64,
    max_value: f64,
}
impl OctavePerlinNoiseSampler {
    pub fn maintain_precision(v: f64) -> f64 {
        // Java OctavePerlinNoiseSampler.maintainPrecision：(long)(v/33554432.0+0.5)*33554432.0
        let l = (v / 3.3554432e7 + 0.5) as i64;
        v - l as f64 * 3.3554432e7
    }
    // legacy 构造（createLegacy：直接消费 random，非 splitter 派生；用于 old_blended_noise/InterpolatedNoiseDF）
    // 对齐 C++ noise.h L153-182：先无条件 new PerlinNoiseSampler(random)（可能丢弃），再对 kx=j-1..0 逐级建或 skip(262)
    pub fn new_legacy(random: &mut XoroshiroRandom, first_octave: i32, amplitudes: &[f64]) -> Self {
        let i = amplitudes.len() as i32;
        let j = -first_octave;
        let mut octave_samplers: Vec<Option<PerlinNoiseSampler>> = (0..i).map(|_| None).collect();
        // Java: 无论如何都 new PerlinNoiseSampler(random)（消费 random），可能丢弃
        let first_pn = PerlinNoiseSampler::new(random);
        if j >= 0 && j < i {
            let d = amplitudes[j as usize];
            if d != 0.0 {
                octave_samplers[j as usize] = Some(first_pn);
            }
        }
        let mut kx = j - 1;
        while kx >= 0 {
            if kx < i {
                let e = amplitudes[kx as usize];
                if e != 0.0 {
                    octave_samplers[kx as usize] = Some(PerlinNoiseSampler::new(random));
                } else {
                    random.skip(262);
                }
            } else {
                random.skip(262);
            }
            kx -= 1;
        }
        let lacunarity = 2.0f64.powf(-j as f64);
        let persistence = 2.0f64.powf(i as f64 - 1.0) / (2.0f64.powf(i as f64) - 1.0);
        let max_value = Self::get_total_amplitude(&octave_samplers, amplitudes, persistence, 2.0);
        OctavePerlinNoiseSampler { octave_samplers, first_octave, amplitudes: amplitudes.to_vec(), persistence, lacunarity, max_value }
    }
    // 便捷：IntStream.rangeClosed(a, b) 全 1 振幅（对齐 C++ noise.h L185-188）
    pub fn range_closed_amplitudes(from: i32, to: i32) -> Vec<f64> {
        let n = to - from + 1;
        vec![1.0f64; n.max(0) as usize]
    }
    // 对齐 C++ noise.h L190-193：idx = size - 1 - octave
    pub fn get_octave(&self, octave: i32) -> Option<&PerlinNoiseSampler> {
        let idx = self.octave_samplers.len() as i32 - 1 - octave;
        if idx >= 0 && idx < self.octave_samplers.len() as i32 {
            self.octave_samplers[idx as usize].as_ref()
        } else { None }
    }
    // 对齐 C++ noise.h L195-197：method_40556(d) = getTotalAmplitude(d + 2.0)
    pub fn method_40556(&self, d: f64) -> f64 { self.total_amplitude_scale(d + 2.0) }
    fn total_amplitude_scale(&self, scale: f64) -> f64 {
        Self::get_total_amplitude(&self.octave_samplers, &self.amplitudes, self.persistence, scale)
    }
    pub fn new(random: &mut XoroshiroRandom, first_octave: i32, amplitudes: &[f64]) -> Self {
        let mut octave_samplers: Vec<Option<PerlinNoiseSampler>> = Vec::new();
        let j = -first_octave;
        let splitter = random.next_splitter();
        for k in 0..amplitudes.len() {
            if amplitudes[k] != 0.0 {
                let l = first_octave + k as i32;
                let mut rnd = splitter.split_str(&format!("octave_{}", l));
                octave_samplers.push(Some(PerlinNoiseSampler::new(&mut rnd)));
            } else {
                octave_samplers.push(None);
            }
        }
        let lacunarity = 2.0f64.powf(-j as f64);
        let i = amplitudes.len() as f64;
        let persistence = 2.0f64.powf(i - 1.0) / (2.0f64.powf(i) - 1.0);
        let max_value = Self::get_total_amplitude(&octave_samplers, amplitudes, persistence, 2.0);
        OctavePerlinNoiseSampler { octave_samplers, first_octave, amplitudes: amplitudes.to_vec(), persistence, lacunarity, max_value }
    }
    fn get_total_amplitude(oct: &[Option<PerlinNoiseSampler>], amp: &[f64], mut pers: f64, scale: f64) -> f64 {
        let mut d = 0.0;
        for i in 0..oct.len() {
            if oct[i].is_some() { d += amp[i] * scale * pers; }
            pers /= 2.0;
        }
        d
    }
    pub fn get_max_value(&self) -> f64 { self.max_value }
    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut d = 0.0f64;
        let mut e = self.lacunarity;
        let mut f = self.persistence;
        for i in 0..self.octave_samplers.len() {
            if let Some(pn) = &self.octave_samplers[i] {
                let g = pn.sample(Self::maintain_precision(x * e), Self::maintain_precision(y * e), Self::maintain_precision(z * e));
                d += self.amplitudes[i] * g * f;
            }
            e *= 2.0;
            f /= 2.0;
        }
        d
    }
}

// ---- DoublePerlinNoiseSampler（C++ noise.h L245-288）----
#[derive(Clone)]
pub struct NoiseParameters { pub first_octave: i32, pub amplitudes: Vec<f64> }
pub struct DoublePerlinNoiseSampler {
    amplitude: f64,
    first_sampler: OctavePerlinNoiseSampler,
    second_sampler: OctavePerlinNoiseSampler,
    max_value: f64,
}
impl DoublePerlinNoiseSampler {
    pub const DOMAIN_SCALE: f64 = 1.0181268882175227;
    fn create_amplitude(octaves: i32) -> f64 { 0.1 * (1.0 + 1.0 / (octaves + 1) as f64) }
    pub fn new(random: &mut XoroshiroRandom, params: &NoiseParameters) -> Self {
        let first_sampler = OctavePerlinNoiseSampler::new(random, params.first_octave, &params.amplitudes);
        let second_sampler = OctavePerlinNoiseSampler::new(random, params.first_octave, &params.amplitudes);
        let mut j = i32::MAX; let mut k = i32::MIN;
        for l in 0..params.amplitudes.len() {
            if params.amplitudes[l] != 0.0 {
                j = j.min(l as i32);
                k = k.max(l as i32);
            }
        }
        let amplitude = 0.16666666666666666 / Self::create_amplitude(k - j);
        let max_value = (first_sampler.get_max_value() + second_sampler.get_max_value()) * amplitude;
        DoublePerlinNoiseSampler { amplitude, first_sampler, second_sampler, max_value }
    }
    pub fn get_max_value(&self) -> f64 { self.max_value }
    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        let d = x * Self::DOMAIN_SCALE;
        let e = y * Self::DOMAIN_SCALE;
        let f = z * Self::DOMAIN_SCALE;
        (self.first_sampler.sample(x, y, z) + self.second_sampler.sample(d, e, f)) * self.amplitude
    }
}
