# -*- coding: utf-8 -*-
"""复现 Java aquifer 随机点派生链，验证 (-244,58,-256) 的 o/p/q 是否 = C++ trace 的 90/99/115。
链: worldSeed -> createXoroshiroSeed(mixStafford13) -> XoroshiroRandom.nextSplitter()
     -> split(Identifier("aquifer")=md5("minecraft:aquifer")) -> nextSplitter()
     -> split(x,y,z)=hashXYZ^lo -> nextInt(10)/nextInt(9)/nextInt(10) -> 距离平方 o/p/q
"""
import sys, hashlib
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

MASK64 = 0xFFFFFFFFFFFFFFFF
MASK32 = 0xFFFFFFFF

def as_s64(v):
    v &= MASK64
    return v - 0x10000000000000000 if v >= 0x8000000000000000 else v

def as_u32(v):
    return v & MASK32

def rotl64(v, r):
    v &= MASK64
    return ((v << r) | (v >> (64 - r))) & MASK64

def mul64(a, b):
    return (a * b) & MASK64

def mix_stafford13(seed):
    seed = as_s64(mul64(seed ^ (seed >> 30) & MASK64, as_s64(-4658895280553007687)))
    seed = as_s64(mul64(seed ^ (seed >> 27) & MASK64, as_s64(-7723592293110705685)))
    return seed ^ (seed >> 31)

def create_unmixed(seed):
    l = as_s64(seed ^ 7640891576956012809)
    m = as_s64(l + as_s64(-7046029254386353131))
    return l, m

def create_xoroshiro_seed_long(seed):
    l, m = create_unmixed(seed)
    return mix_stafford13(l), mix_stafford13(m)

def create_xoroshiro_seed_str(s):
    h = hashlib.md5(s.encode("utf-8")).digest()
    lo = int.from_bytes(h[0:8], "big", signed=True)
    hi = int.from_bytes(h[8:16], "big", signed=True)
    return lo, hi

class Xoroshiro128:
    def __init__(self, lo, hi):
        self.lo = lo & MASK64
        self.hi = hi & MASK64
        if (self.lo | self.hi) == 0:
            self.lo = as_s64(-7046029254386353131)
            self.hi = 7640891576956012809
    def next(self):
        l = self.lo
        m = self.hi
        n = (rotl64(l + m, 17) + l) & MASK64
        m ^= l
        self.lo = (rotl64(l, 49) ^ m ^ (m << 21)) & MASK64
        self.hi = rotl64(m, 28)
        return n  # 无符号表示（Java long 位模式等价）

def hashXYZ(x, y, z):
    # Java: int l = x * 3129871 (int 溢出); long r = l ^ z*116129781L ^ y; r=r*r*42317861L+r*11L; return r>>16 (算术)
    xi = as_u32(x * 3129871)
    if xi >= 0x80000000:
        xi -= 0x100000000
    l = as_s64(xi ^ (z * 116129781) ^ y)
    u = mul64(mul64(l, l), 42317861)
    u = (u + mul64(l, 11)) & MASK64
    return as_s64(u) >> 16  # Python 负数 >> 是算术右移

class Splitter:
    def __init__(self, lo, hi):
        self.lo = lo & MASK64
        self.hi = hi & MASK64
    def split_xyz(self, x, y, z):
        l = hashXYZ(x, y, z)
        m = (l ^ self.lo) & MASK64
        return Xoroshiro128(m, self.hi)
    def split_str(self, s):
        lo, hi = create_xoroshiro_seed_str(s)
        return Xoroshiro128(lo ^ self.lo, hi ^ self.hi)

def next_int(rng, bound):
    # Java Xoroshiro128PlusPlusRandom.nextInt(bound)
    l = rng.next() & MASK32  # (int)impl.next()
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
    return -((-a) // b) if (a < 0) != (b < 0) else a // b

def aquifer_oppq(wx, wy, wz, world_seed):
    # 1) randomProvider.create(seed).nextSplitter()
    lo, hi = create_xoroshiro_seed_long(world_seed)
    rng = Xoroshiro128(lo, hi)
    rd_lo, rd_hi = rng.next(), rng.next()
    randomDeriver = Splitter(rd_lo, rd_hi)
    # 2) randomDeriver.split(Identifier("aquifer")).nextSplitter()
    aq = randomDeriver.split_str("minecraft:aquifer")
    a_lo, a_hi = aq.next(), aq.next()
    aquiferSplitter = Splitter(a_lo, a_hi)
    # 3) apply 内部: cell 坐标
    l = floor_div(wx - 5, 16)
    m = floor_div(wy + 1, 12)
    n = floor_div(wz - 5, 16)
    pts = []
    for u in range(2):
        for v in range(-1, 2):
            for w in range(2):
                x, y, z = l + u, m + v, n + w
                rng = aquiferSplitter.split_xyz(x, y, z)
                bx = x * 16 + next_int(rng, 10)
                by = y * 12 + next_int(rng, 9)
                bz = z * 16 + next_int(rng, 10)
                dx, dy, dz = bx - wx, by - wy, bz - wz
                pts.append((dx * dx + dy * dy + dz * dz, (x, y, z), (bx, by, bz)))
    pts.sort(key=lambda t: t[0])
    return pts, l, m, n, aquiferSplitter

WORLD = -8248318472910187742
for wy in (55, 56, 57, 58, 59, 60, 61, 62):
    pts, l, m, n, sp = aquifer_oppq(-244, wy, -256, WORLD)
    o, p, q = pts[0][0], pts[1][0], pts[2][0]
    print(f"(-244,{wy},-256) cell=({l},{m},{n}) o={o} p={p} q={q}  | 最近点: {pts[0][1]} {pts[0][2]}")
print("\nC++ trace_aqf_1.txt 参照: (55)o=54,p=101,q=126 (56)o=67,p=104,q=126 (57)o=82,p=107,q=109 (58)o=90,p=99,q=115 (59)o=75,p=106,q=118 (60)o=62,p=99,q=136 (61)o=51,p=94,q=149 (62)o=42,p=91,q=142")
