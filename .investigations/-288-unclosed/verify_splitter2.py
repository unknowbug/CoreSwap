# -*- coding: utf-8 -*-
"""复现 Java aquifer 随机点派生链（修正版：全 64 位无符号补码运算）。
验证 (-244,58,-256) 的 o/p/q 是否 = C++ trace 的 90/99/115。
"""
import sys, hashlib
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

MASK64 = 0xFFFFFFFFFFFFFFFF
MASK32 = 0xFFFFFFFF

def as_u64(v):
    return v & MASK64

def as_s64(v):
    v &= MASK64
    return v - 0x10000000000000000 if v >= 0x8000000000000000 else v

def rotl64(v, r):
    v &= MASK64
    return ((v << r) | (v >> (64 - r))) & MASK64

def mul64(a, b):
    return (a * b) & MASK64

def mix_stafford13(seed):
    # Java: seed = (seed ^ seed >>> 30) * C1; seed = (seed ^ seed >>> 27) * C2; return seed ^ seed >>> 31
    seed &= MASK64
    seed = mul64(seed ^ (seed >> 30), as_u64(-4658895280553007687))
    seed = mul64(seed ^ (seed >> 27), as_u64(-7723592293110705685))
    return (seed ^ (seed >> 31)) & MASK64

def create_xoroshiro_seed_long(seed):
    # Java: createUnmixedXoroshiroSeed(seed).mix()
    l = as_u64(seed) ^ 7640891576956012809
    m = (l + as_u64(-7046029254386353131)) & MASK64
    return mix_stafford13(l), mix_stafford13(m)

def create_xoroshiro_seed_str(s):
    h = hashlib.md5(s.encode("utf-8")).digest()
    lo = int.from_bytes(h[0:8], "big")
    hi = int.from_bytes(h[8:16], "big")
    return lo, hi

class Xoroshiro128:
    def __init__(self, lo, hi):
        self.lo = lo & MASK64
        self.hi = hi & MASK64
        if (self.lo | self.hi) == 0:
            self.lo = as_u64(-7046029254386353131)
            self.hi = 7640891576956012809
    def next(self):
        l = self.lo
        m = self.hi
        n = (rotl64(l + m, 17) + l) & MASK64
        m ^= l
        self.lo = (rotl64(l, 49) ^ m ^ (m << 21)) & MASK64
        self.hi = rotl64(m, 28)
        return n  # 64 位无符号

def hashXYZ(x, y, z):
    # Java MathHelper.hashCode(x,y,z): long l = x*3129871 ^ z*116129781L ^ y (x*3129871 是 int 溢出)
    xi = (x * 3129871) & MASK32
    if xi >= 0x80000000:
        xi -= 0x100000000  # 符号扩展为 long 的 32 位值
    l = (xi ^ (z * 116129781) ^ y) & MASK64
    l = (l * l * 42317861 + l * 11) & MASK64
    return as_s64(l) >> 16  # 算术右移（带符号返回）

class Splitter:
    def __init__(self, lo, hi):
        self.lo = lo & MASK64
        self.hi = hi & MASK64
    def split_xyz(self, x, y, z):
        l = hashXYZ(x, y, z)
        m = (as_u64(l) ^ self.lo) & MASK64
        return Xoroshiro128(m, self.hi)
    def split_str(self, s):
        lo, hi = create_xoroshiro_seed_str(s)
        return Xoroshiro128(lo ^ self.lo, hi ^ self.hi)

def next_int(rng, bound):
    # Java Xoroshiro128PlusPlusRandom.nextInt(bound): toUnsignedLong((int)next()) * bound 高 32 位 + 拒绝采样
    l = rng.next() & MASK32
    m = (l * bound) & MASK64
    n = m & MASK32
    if n < bound:
        i = ((0x100000000 - bound) % bound)  # Integer.remainderUnsigned(~bound+1, bound)
        while n < i:
            l = rng.next() & MASK32
            m = (l * bound) & MASK64
            n = m & MASK32
    return (m >> 32) & MASK32

def floor_div(a, b):
    return a // b  # Python // 即 floor division（Java Math.floorDiv 语义）

def build_aquifer_splitter(world_seed):
    lo, hi = create_xoroshiro_seed_long(world_seed)
    rng = Xoroshiro128(lo, hi)
    randomDeriver = Splitter(rng.next(), rng.next())
    aq = randomDeriver.split_str("minecraft:aquifer")
    return Splitter(aq.next(), aq.next())

def aquifer_oppq(wx, wy, wz, world_seed):
    sp = build_aquifer_splitter(world_seed)
    l = floor_div(wx - 5, 16)
    m = floor_div(wy + 1, 12)
    n = floor_div(wz - 5, 16)
    pts = []
    for u in range(2):
        for v in range(-1, 2):
            for w in range(2):
                x, y, z = l + u, m + v, n + w
                rng = sp.split_xyz(x, y, z)
                bx = x * 16 + next_int(rng, 10)
                by = y * 12 + next_int(rng, 9)
                bz = z * 16 + next_int(rng, 10)
                dx, dy, dz = bx - wx, by - wy, bz - wz
                pts.append((dx * dx + dy * dy + dz * dz, (x, y, z), (bx, by, bz)))
    pts.sort(key=lambda t: t[0])
    return pts, (l, m, n)

WORLD = -8248318472910187742
# 先验证 splitter 种子链
sp = build_aquifer_splitter(WORLD)
print(f"aquifer splitter: lo={sp.lo:#018x} hi={sp.hi:#018x}")

for wy in (55, 56, 57, 58, 59, 60, 61, 62):
    pts, cell = aquifer_oppq(-244, wy, -256, WORLD)
    o, p, q = pts[0][0], pts[1][0], pts[2][0]
    print(f"(-244,{wy},-256) cell=({cell[0]},{cell[1]},{cell[2]}) o={o} p={p} q={q} | 最近: {pts[0][1]}->{pts[0][2]}")
print("\nC++ trace_aqf_1.txt 参照: (55)o=54,p=101,q=126 (56)o=67,p=104,q=126 (57)o=82,p=107,q=109 (58)o=90,p=99,q=115 (59)o=75,p=106,q=118 (60)o=62,p=99,q=136 (61)o=51,p=94,q=149 (62)o=42,p=91,q=142")
