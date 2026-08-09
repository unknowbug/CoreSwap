# -*- coding: utf-8 -*-
"""细化核心差异候选：stone->gravel / stone->dirt / stone->air 的 y 分布 + biome 分布
判定：surface 规则差（浅层 y>40）vs ore_gravel/洞穴 FEATURE（深层 y<30）vs carvers
"""
import json, sys, re, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

DATA = r"E:\PYTHON\CoreSwap\versions\1.20.1\data"
blocks = json.load(open(DATA + r"\blocks.json", encoding="utf-8"))
id2name = {v: k for k, v in blocks.items()}

LINE = re.compile(r"MISMATCH chunk\((-?\d+),(-?\d+)\) pos\((-?\d+),(-?\d+),(-?\d+)\) got=(\d+) vanilla=(\d+) biome=(.*)$")
rows = []
with open(r"E:\PYTHON\CoreSwap\.investigations\-288-unclosed\m300515_run1.txt", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line.startswith("MISMATCH"):
            continue
        m = LINE.match(line)
        if not m:
            continue
        cx, cz, x, y, z, gid, vid, biome = m.groups()
        rows.append((int(x), int(y), int(z), int(gid), int(vid), biome))

def shortname(bid):
    nm = id2name.get(bid, f"id{bid}")
    return nm.split(":")[1] if ":" in nm else nm

TARGETS = [("stone", "gravel"), ("deepslate", "gravel"), ("stone", "dirt"), ("deepslate", "dirt"),
           ("stone", "air"), ("deepslate", "air"), ("stone", "cave_air"), ("deepslate", "cave_air"),
           ("stone", "water"), ("deepslate", "water")]

for g, v in TARGETS:
    sel = [(x, y, z, b) for x, y, z, gid, vid, b in rows if shortname(gid) == g and shortname(vid) == v]
    if not sel:
        continue
    ys = collections.Counter(y for _, y, _, _ in sel)
    bm = collections.Counter(b for _, _, _, b in sel)
    print(f"== {g} -> {v}: {len(sel)} 块 ==")
    print("  y 分布 top12:", dict(sorted(ys.items(), key=lambda kv: -kv[1])[:12]))
    print("  biome:", dict(bm.most_common(4)))
    # y 带划分
    deep = sum(n for y, n in ys.items() if y < 30)
    surf = sum(n for y, n in ys.items() if y >= 30)
    print(f"  深层(y<30)={deep} 浅层(y>=30)={surf}")
