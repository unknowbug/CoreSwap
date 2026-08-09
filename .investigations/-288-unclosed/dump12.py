# -*- coding: utf-8 -*-
"""打印 (-244,58,-256) 的 12 邻居完整点，对比 C++ o=90/p=99/q=115"""
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
        self.lo = lo & MASK64
        self.hi = hi & MASK64
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

def floor_div(a, b):
    return -((-a) // b) if (a < 0) != (b < 0) else a // b

WORLD = -8248318472910187742
lo, hi = create_xoroshiro_seed_long(WORLD)
rd = Splitter(Xoroshiro128(lo, hi).next(), Xoroshiro128(lo, hi).next())
# 注意：next 有状态，重新构建
rng0 = Xoroshiro128(lo, hi)
rd = Splitter(rng0.next(), rng0.next())
aq = rd.split_str("minecraft:aquifer")
sp = Splitter(aq.next(), aq.next())
print(f"randomDeriver lo={rd.lo:#018x} hi={rd.hi:#018x}")
print(f"aquiferSplitter lo={sp.lo:#018x} hi={sp.hi:#018x}")

wx, wy, wz = -244, 58, -256
l = floor_div(wx - 5, 16)
m = floor_div(wy + 1, 12)
n = floor_div(wz - 5, 16)
print(f"cell base: l={l} m={m} n={n}")

pts = []
for u in range(2):
    for v in range(-1, 2):
        for w in range(2):
            x, y, z = l + u, m + v, n + w
            rng = sp.split_xyz(x, y, z)
            r1 = next_int(rng, 10)
            r2 = next_int(rng, 9)
            r3 = next_int(rng, 10)
            bx = x * 16 + r1
            by = y * 12 + r2
            bz = z * 16 + r3
            dx, dy, dz = bx - wx, by - wy, bz - wz
            d2 = dx * dx + dy * dy + dz * dz
            pts.append((d2, (x, y, z), (bx, by, bz), (r1, r2, r3)))
            print(f"cell({x},{y},{z}) rnd=({r1},{r2},{r3}) pos=({bx},{by},{bz}) d2={d2}")

pts.sort(key=lambda t: t[0])
print(f"\nsorted top5: {[(p[0], p[1]) for p in pts[:5]]}")
print("C++ 参照: o=90 p=99 q=115 (最近3个距离平方)")
