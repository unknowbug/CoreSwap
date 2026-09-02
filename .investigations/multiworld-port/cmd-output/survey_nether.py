# -*- coding: utf-8 -*-
# survey_nether.py — 玩家活动区 nether chunk 纵向轮廓普查（超平坦签名检测）
# 用法: python survey_nether.py <region_dir> <cx0> <cz0> <cx1> <cz1>
import sys, struct, collections

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
_src = open(r"E:\PYTHON\CoreSwap\.investigations\multiworld-port\parse_mca_chunk.py", encoding="utf-8").read()
_src = _src.replace("\nmain()\n", "\n")
_ns = {"__name__": "pm"}
exec(compile(_src, "parse_mca_chunk.py", "exec"), _ns)
read_mca_chunk = _ns["read_mca_chunk"]; parse_nbt_named = _ns["parse_nbt_named"]; unpack_states = _ns["unpack_states"]
import os

region_dir, cx0, cz0, cx1, cz1 = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
profiles = collections.Counter()
status_count = collections.Counter()
for cx in range(cx0, cx1 + 1):
    for cz in range(cz0, cz1 + 1):
        f = os.path.join(region_dir, "r.%d.%d.mca" % (cx >> 5, cz >> 5))
        if not os.path.exists(f):
            continue
        raw = read_mca_chunk(f, cx, cz)
        if raw is None:
            continue
        _, root = parse_nbt_named(raw)
        st = root.get("Status", "?")
        status_count[st] += 1
        prof = []
        for s in root.get("sections", []):
            bs = s.get("block_states", {}) or {}
            pal = bs.get("palette", [])
            data = bs.get("data")
            if data is None or not pal:
                prof.append(0)
                continue
            bits = max(4, (len(pal) - 1).bit_length())
            L = data if isinstance(data, list) else list(struct.unpack(">%dq" % len(data), bytes(data)))
            idxs = unpack_states(L, bits, 4096)
            nonair = 0
            for ix in idxs:
                p = pal[ix]
                nm = p.get("Name", "") if isinstance(p, dict) else ""
                if nm not in ("minecraft:air", "minecraft:cave_air", "minecraft:void_air"):
                    nonair += 1
            prof.append(nonair)
        # 指纹：每 section 非空数量的分桶（0 / 1-1000 / 1001-3000 / 3001+）
        fp = tuple("".join("0" if v == 0 else "L" if v <= 1000 else "M" if v <= 3000 else "H" for v in [p_]) for p_ in prof)
        profiles[(st, tuple(prof if len(prof) < 20 else prof[:20]))] = profiles.get((st, tuple(prof)), 0) + 1

print("status 分布:", dict(status_count))
print("top 轮廓指纹（status, 每section非空块数）:")
for (st, prof), n in profiles.most_common(8):
    print("  x%-4d %s %s" % (n, st, prof))
