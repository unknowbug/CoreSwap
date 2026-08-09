# -*- coding: utf-8 -*-
"""打印 (-244,55..62,-256) 的 o/p/q 对应随机点坐标（r/s/t），供 C++ ESTDUMP 验证各列 est"""
import sys, hashlib
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

MASK64 = 0xFFFFFFFFFFFFFFFF
MASK32 = 0xFFFFFFFF

def as_u64(v): return v & MASK64
def as_s64(v):
    v &= MASK64
    return v - 0x10000000000000000 if v >= 0x8000000000000000 else v
def rotl64(v, r):
    v &= MASK64
    return ((v << r) | (v >> (64 - r))) & MASK64
def mul64(a, b): return (a * b) & MASK64

def mix_stafford13(seed):
    seed &= MASK64
    seed = mul64(seed ^ (seed >> 30), as_u64(-4658895280553007687))
    seed = mul64(seed ^ (seed >> 27), as_u64(-7723592293110705685))
    return (seed ^ (seed >> 31)) & MASK64

def create_xoroshiro_seed_long(seed):
    l = as_u64(seed) ^ 7640891576956012809
    m = (l + as_u64(-7046029254386353131)) & MASK64
    return mix_stafford13(l), mix_stafford13(m)

def create_xoroshiro_seed_str(s):
    h = hashlib.md5(s.encode("utf-8")).digest()
    return int.from_bytes(h[0:8], "big"), int.from_bytes(h[8:16], "big")

class Xoroshiro128:
    def __init__(self, lo, hi):
        self.lo, self.hi = lo & MASK64, hi & MASK64
        if (self.lo | self.hi) == 0:
            self.lo = as_u64(-7046029254386353131)
            self.hi = 7640891576956012809
    def next(self):
        l, m = self.lo, self.hi
        n = (rotl64(l + m, 17) + l) & MASK64
        m ^= l
        self.lo = (rotl64(l, 49) ^ m ^ (m << 21)) & MASK64
        self.hi = rotl64(m, 28)
        return n

def hashXYZ(x, y, z):
    xi = (x * 3129871) & MASK32
    if xi >= 0x80000000: xi -= 0x100000000
    l = (xi ^ (z * 116129781) ^ y) & MASK64
    l = (l * l * 42317861 + l * 11) & MASK64
    return as_s64(l) >> 16

class Splitter:
    def __init__(self, lo, hi):
        self.lo, self.hi = lo & MASK64, hi & MASK64
    def split_xyz(self, x, y, z):
        return Xoroshiro128(as_u64(hashXYZ(x, y, z)) ^ self.lo, self.hi)
    def split_str(self, s):
        lo, hi = create_xoroshiro_seed_str(s)
        return Xoroshiro128(lo ^ self.lo, hi ^ self.hi)

def next_int(rng, bound):
    l = rng.next() & MASK32
    m = (l * bound) & MASK64
    n = m & MASK32
    if n < bound:
        i = ((0x100000000 - bound) % bound)
        while n < i:
            l = rng.next() & MASK32
            m = (l * bound) & MASK64
            n = m & MASK32
    return (m >> 32) & MASK32

def build_aquifer_splitter(world_seed):
    lo, hi = create_xoroshiro_seed_long(world_seed)
    rng = Xoroshiro128(lo, hi)
    rd = Splitter(rng.next(), rng.next())
    aq = rd.split_str("minecraft:aquifer")
    return Splitter(aq.next(), aq.next())

WORLD = -8248318472910187742
sp = build_aquifer_splitter(WORLD)
for wy in (55, 56, 57, 58, 59, 60, 61, 62):
    wx, wz = -244, -256
    l = wx // 16 if wx >= 0 else -((-wx + 15) // 16)  # floorDiv(wx-5,16) 用下面正式计算
    l = (wx - 5) // 16  # floorDiv 语义（Python // 即 floor）
    m = (wy + 1) // 12
    n = (wz - 5) // 16
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
                pts.append((dx * dx + dy * dy + dz * dz, (bx, by, bz), (x, y, z)))
    pts.sort(key=lambda t: t[0])
    r, s, t = pts[0], pts[1], pts[2]
    print(f"y={wy}: o={r[0]} r点={r[1]} cell={r[2]} | p={s[0]} s点={s[1]} cell={s[2]} | q={t[0]} t点={t[1]} cell={t[2]}")
