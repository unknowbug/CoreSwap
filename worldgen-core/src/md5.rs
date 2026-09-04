// md5.rs — MD5（RFC 1321），用于 create_xoroshiro_seed_str（Java md5 + Longs.fromBytes 对齐）
const S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
    5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];
const K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];
fn left_rotate(x: u32, c: u32) -> u32 { (x << c) | (x >> (32 - c)) }

pub fn md5(input: &[u8]) -> [u8; 16] {
    let mut msg = input.to_vec();
    let orig_len = (input.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&orig_len.to_le_bytes());

    let mut a0 = 0x67452301u32;
    let mut b0 = 0xefcdab89u32;
    let mut c0 = 0x98badcfeu32;
    let mut d0 = 0x10325476u32;
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 16];
        for i in 0..16 {
            w[i] = u32::from_le_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        let mut a = a0; let mut b = b0; let mut c = c0; let mut d = d0;
        for i in 0..64 {
            let (f, g);
            if i < 16 { f = (b & c) | ((!b) & d); g = i; }
            else if i < 32 { f = (d & b) | ((!d) & c); g = (5 * i + 1) % 16; }
            else if i < 48 { f = b ^ c ^ d; g = (3 * i + 5) % 16; }
            else { f = c ^ (b | (!d)); g = (7 * i) % 16; }
            let t = d;
            d = c; c = b;
            b = b.wrapping_add(left_rotate(a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(w[g]), S[i]));
            a = t;
        }
        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }
    let mut out = [0u8; 16];
    for (i, v) in [a0, b0, c0, d0].iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&v.to_le_bytes());
    }
    out
}

/// md5(str) -> (lo, hi)——Java md5 + Longs.fromBytes（big-endian，xoroshiro.h L94-101）
pub fn md5_lo_hi(seed: &str) -> (u64, u64) {
    let d = md5(seed.as_bytes());
    let mut lo = 0u64;
    let mut hi = 0u64;
    for i in 0..8 { lo = (lo << 8) | d[i] as u64; }
    for i in 0..8 { hi = (hi << 8) | d[8 + i] as u64; }
    (lo, hi)
}
