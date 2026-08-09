# -*- coding: utf-8 -*-
"""-288 未闭合三大子类（海底边界/gravel/表面规则）精确归类 + 量化。
数据源: .investigations/-288-reopen/m288_natural_rows.txt（natural 类 MISMATCH 行，带块名）
输出: 子类坐标集合、y 分布、chunk 分布、代表性样本列清单
"""
import sys, re, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

SRC = r"E:\PYTHON\CoreSwap\.investigations\-288-reopen\m288_natural_rows.txt"

LINE = re.compile(r"chunk\((-?\d+),(-?\d+)\) pos\((-?\d+),(-?\d+),(-?\d+)\) got=(\d+)\((.*?)\) vanilla=(\d+)\((.*?)\) biome=(.*)$")

rows = []
with open(SRC, encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        m = LINE.match(line)
        if not m:
            print("PARSE FAIL:", line)
            continue
        cx, cz, x, y, z, gid, gname, vid, vname, biome = m.groups()
        rows.append(dict(cx=int(cx), cz=int(cz), x=int(x), y=int(y), z=int(z),
                         gid=int(gid), gname=gname, vid=int(vid), vname=vname, biome=biome))

print("total rows:", len(rows))

# ---- 子类分类 ----
seabed, gravel_cls, surf_cls, rest = [], [], [], []

WATER = {"minecraft:water"}
SOLID_SEABED = {"minecraft:stone", "minecraft:dirt", "minecraft:sand", "minecraft:gravel", "minecraft:sandstone"}
GRAVEL = {"minecraft:gravel"}
SURF_PAIR = {"minecraft:sand", "minecraft:sandstone", "minecraft:dirt", "minecraft:grass_block", "minecraft:stone"}

for r in rows:
    g, v = r["gname"], r["vname"]
    if (g in WATER and v in SOLID_SEABED) or (v in WATER and g in SOLID_SEABED):
        seabed.append(r)
    elif g in GRAVEL or v in GRAVEL:
        gravel_cls.append(r)
    elif g in SURF_PAIR and v in SURF_PAIR:
        surf_cls.append(r)
    else:
        rest.append(r)

print("seabed(water<->solid):", len(seabed))
print("gravel:", len(gravel_cls))
print("surface_rules:", len(surf_cls))
print("rest:", len(rest))

def ydist(rs, label):
    c = collections.Counter(r["y"] for r in rs)
    print(f"\n== {label} y 分布 (top20) ==")
    for y, n in sorted(c.items()):
        if n >= 5 or y in (48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63):
            print(f"  y={y}: {n}")

def chunkdist(rs, label):
    c = collections.Counter((r["cx"], r["cz"]) for r in rs)
    print(f"\n== {label} chunk 分布 ==")
    for k, n in sorted(c.items()):
        print(f"  chunk({k[0]},{k[1]}): {n}")

def pairdist(rs, label):
    c = collections.Counter((r["gname"], r["vname"]) for r in rs)
    print(f"\n== {label} pair 分布 ==")
    for (g, v), n in sorted(c.items(), key=lambda kv: -kv[1]):
        print(f"  {g} -> {v}: {n}")

def biometop(rs, label):
    c = collections.Counter(r["biome"] for r in rs)
    print(f"\n== {label} biome 分布 ==")
    for b, n in sorted(c.items(), key=lambda kv: -kv[1]):
        print(f"  {b}: {n}")

ydist(seabed, "seabed")
pairdist(seabed, "seabed")
chunkdist(seabed, "seabed")
biometop(seabed, "seabed")

ydist(gravel_cls, "gravel")
pairdist(gravel_cls, "gravel")
chunkdist(gravel_cls, "gravel")
biometop(gravel_cls, "gravel")

ydist(surf_cls, "surface_rules")
pairdist(surf_cls, "surface_rules")
chunkdist(surf_cls, "surface_rules")
biometop(surf_cls, "surface_rules")

# ---- 代表性样本列（每子类选 y 最集中的 x,z 列）----
def sample_cols(rs, label, n=6):
    cols = collections.Counter((r["x"], r["z"]) for r in rs)
    print(f"\n== {label} 样本列（按命中数 top{n}）==")
    for (x, z), cnt in cols.most_common(n):
        ys = sorted(r["y"] for r in rs if r["x"] == x and r["z"] == z)
        pairs = [(r["y"], r["gname"], r["vname"], r["biome"]) for r in rs if r["x"] == x and r["z"] == z]
        print(f"  col({x},{z}) hit={cnt} ys={ys[0]}..{ys[-1]}")
        for y, g, v, b in sorted(pairs):
            print(f"    y={y} {g} -> {v} [{b}]")

sample_cols(seabed, "seabed")
sample_cols(gravel_cls, "gravel")
sample_cols(surf_cls, "surface_rules")

# ---- rest 里还有什么（防漏）----
print("\n== rest pair 分布 top20 ==")
for (g, v), n in sorted(collections.Counter((r["gname"], r["vname"]) for r in rest).items(),
                        key=lambda kv: -kv[1])[:20]:
    print(f"  {g} -> {v}: {n}")
print("rest total:", len(rest))
