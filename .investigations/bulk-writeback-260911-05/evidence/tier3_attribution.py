# -*- coding: utf-8 -*-
"""C 线 Tier 3 差分归属（严格版）：把 A/B 两世界间的块差分，归到
  ① 两侧都无内容（空 chunk）的 chunk   ② 任一侧有内容的 chunk
用于把「空 chunk 稀释分母」这一说法做成逐块可核的归因，而非按单臂 profile 推断。

廉价判据 = section palette 是否只含 air（不解码）；仅对「任一侧有内容」的 chunk 解码。
解析器复用 B1 已修正工具（exec 前缀，零复制）。
"""
import sys, os, re
from collections import Counter
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
ROOT = r"E:\PYTHON\CoreSwap"
TOOL = os.path.join(ROOT, ".investigations", "e5-recompute-260911-05", "diff_arms_fixed.py")
src = open(TOOL, encoding="utf-8").read()
ns = {}
exec(compile(src[:src.index("van=load_world(sys.argv[1])")], TOOL, "exec"), ns)
load_world = ns["load_world"]

A = load_world(os.path.join(ROOT, ".tmp", "c-260911-05", sys.argv[1]))
B = load_world(os.path.join(ROOT, ".tmp", "c-260911-05", sys.argv[2]))
common = sorted(set(A) & set(B))
print(f"[世界] A={len(A)} B={len(B)} common={len(common)}  (A={sys.argv[1]} B={sys.argv[2]})")


def has_content(ch):
    for s in ch.get("sections", []):
        bs = s.get("block_states")
        if not bs:
            continue
        for e in (bs.get("palette") or []):
            nm = e.get("Name") if isinstance(e, dict) else str(e)
            if nm and nm != "minecraft:air":
                return True
    return False


def decode(bs):
    pal = [e.get("Name", "?") if isinstance(e, dict) else str(e) for e in bs["palette"]]
    data = bs.get("data")
    if data is None:
        return [pal[0]] * 4096
    bits = max(4, (len(pal) - 1).bit_length()); per = 64 // bits
    out = []
    for li in range(4096):
        v = (data[li // per] >> ((li % per) * bits)) & ((1 << bits) - 1)
        out.append(pal[v] if v < len(pal) else "?")
    return out


stat = {"both_empty": [0, 0, 0], "any_content": [0, 0, 0]}  # [chunks, blocks, diff]
pairs = {"both_empty": Counter(), "any_content": Counter()}
for key in common:
    a, b = A[key], B[key]
    ca, cb = has_content(a), has_content(b)
    bucket = "any_content" if (ca or cb) else "both_empty"
    stat[bucket][0] += 1
    va = {s.get("Y"): s.get("block_states") for s in a.get("sections", []) if s.get("Y") is not None}
    vb = {s.get("Y"): s.get("block_states") for s in b.get("sections", []) if s.get("Y") is not None}
    for cy in set(va) | set(vb):
        x, y = va.get(cy), vb.get(cy)
        if x is None and y is None:
            continue
        stat[bucket][1] += 4096
        if x == y:
            continue
        dx = ["minecraft:air"] * 4096 if x is None else decode(x)
        dy = ["minecraft:air"] * 4096 if y is None else decode(y)
        if dx == dy:
            continue
        for i in range(4096):
            if dx[i] != dy[i]:
                stat[bucket][2] += 1
                pairs[bucket][(dx[i], dy[i])] += 1

print()
for k in ("both_empty", "any_content"):
    n, blocks, diff = stat[k]
    pct = 100 * diff / max(1, blocks)
    print(f"=== {k} ===  chunk={n}  blocks={blocks}  diff={diff} ({pct:.4f}%)")
    for (x, y), c in pairs[k].most_common(5):
        print(f"    {x} -> {y}: {c}")
tot_b = stat["both_empty"][1] + stat["any_content"][1]
tot_d = stat["both_empty"][2] + stat["any_content"][2]
print(f"\n合计 blocks={tot_b} diff={tot_d} ({100*tot_d/max(1,tot_b):.4f}%)")
print(f"差分中落在「任一侧有内容」chunk 的比例 = {100*stat['any_content'][2]/max(1,tot_d):.2f}%")
print(f"差分中落在「两侧皆空」chunk 的比例     = {100*stat['both_empty'][2]/max(1,tot_d):.2f}%")
