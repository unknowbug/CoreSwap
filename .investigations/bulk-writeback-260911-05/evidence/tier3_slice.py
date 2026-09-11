# -*- coding: utf-8 -*-
"""C 线 Tier 3 切片（260911-05）：把 region 差分**限制在真正走过 bulk 写回的 chunk**上。

动机：全域差分 0.0277% 且 top pairs 全是 FEATURES 阶段产物（sculk/kelp/seagrass/leaves/矿石），
而 Tier 1/2 已证 NOISE 后逐 chunk 状态（方块 + 三计数）全等 ⇒ 必须分辨「写回引起」还是
「同实现跨 run 固有抖动」。本脚本用 [WG-CONTENT-WB] 的 chunk 坐标做切片。

解析器**逐字节复用** B1 已修正的对拍工具（exec 其前缀源码，零复制、口径可比）：
  .investigations/e5-recompute-260911-05/diff_arms_fixed.py
"""
import sys, os, re
from collections import Counter
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = r"E:\PYTHON\CoreSwap"
TOOL = os.path.join(ROOT, ".investigations", "e5-recompute-260911-05", "diff_arms_fixed.py")
LOG = os.path.join(ROOT, ".tmp", "vivo-stutter-260911-02", "ab-threads", "logs", "c-bulk.log")

# ---- 复用工具解析器（exec 前缀，保证与 B1 口径同一份代码） ----
src = open(TOOL, encoding="utf-8").read()
cut = src.index("van=load_world(sys.argv[1])")
ns = {}
exec(compile(src[:cut], TOOL, "exec"), ns)
load_world = ns["load_world"]

# ---- 取走过 bulk 写回的 chunk 坐标（两臂 chunk 集与逐 chunk hash 已由 Tier 1 证全等） ----
RE_WB = re.compile(r"\[WG-CONTENT-WB\] chunk\((-?\d+),(-?\d+)\) hash=([0-9a-f]+) dh=([0-9a-f]+)")
takeover = {}
for line in open(LOG, encoding="utf-8", errors="replace"):
    m = RE_WB.search(line)
    if m:
        takeover[(int(m.group(1)), int(m.group(2)))] = m.group(3)
print(f"[切片] 走过 bulk 写回的 chunk = {len(takeover)} 个（[WG-CONTENT-WB] 行）")

A = load_world(os.path.join(ROOT, ".tmp", "c-260911-05", sys.argv[1]))
B = load_world(os.path.join(ROOT, ".tmp", "c-260911-05", sys.argv[2]))
common = set(A) & set(B)
print(f"[世界] A={len(A)} B={len(B)} common={len(common)}  (A={sys.argv[1]}, B={sys.argv[2]})")


def pal_names(pal):
    return [e.get("Name", "?") if isinstance(e, dict) else str(e) for e in pal]


def decode(bs):
    pal = pal_names(bs["palette"]); data = bs.get("data")
    if data is None:
        return [pal[0]] * 4096
    bits = max(4, (len(pal) - 1).bit_length()); per = 64 // bits
    out = []
    for li in range(4096):
        v = (data[li // per] >> ((li % per) * bits)) & ((1 << bits) - 1)
        out.append(pal[v] if v < len(pal) else "?")
    return out


def census(chunks, label):
    ndiff = nblock = sec_same = sec_diff = 0
    pairs = Counter()
    for key in sorted(chunks):
        if key not in common:
            continue
        cx, cz = key
        vsecs = {s.get("Y"): s for s in A[key].get("sections", []) if s.get("Y") is not None}
        csecs = {s.get("Y"): s for s in B[key].get("sections", []) if s.get("Y") is not None}
        for cy in set(vsecs) | set(csecs):
            vs, csn = vsecs.get(cy), csecs.get(cy)
            vb = vs.get("block_states") if vs else None
            cb = csn.get("block_states") if csn else None
            if vb is None and cb is None:
                continue
            # 快路径：原始 NBT 相等 ⇒ 解码必然相等（不改变判定语义，只省时间）
            if vb == cb:
                sec_same += 1; nblock += 4096; continue
            vdec = ["minecraft:air"] * 4096 if vb is None else decode(vb)
            cdec = ["minecraft:air"] * 4096 if cb is None else decode(cb)
            if vdec == cdec:
                sec_same += 1; nblock += 4096; continue
            sec_diff += 1
            for i in range(4096):
                nblock += 1
                if vdec[i] != cdec[i]:
                    ndiff += 1; pairs[(vdec[i], cdec[i])] += 1
    pct = 100 * ndiff / max(1, nblock)
    print(f"\n=== {label} ===")
    print(f"blocks={nblock} diff={ndiff} ({pct:.4f}%) sections same/diff={sec_same}/{sec_diff}")
    for (a, b), c in pairs.most_common(8):
        print(f"  {a} -> {b}: {c}")
    return ndiff, nblock


only_tk = set(takeover) & common
print(f"\n[切片] 走过写回且两世界都有的 chunk = {len(only_tk)} / {len(common)}")
nd1, nb1 = census(only_tk, "切片 A：仅走过 bulk 写回的 chunk")
nd2, nb2 = census(common - only_tk, "切片 B：未走过本代码路径的 chunk（对照）")
print(f"\n>>> 若切片 A 的 diff=0 ⇒ 写回路径在最终存档层亦无可测差异，全域残差来自未走本路径的 chunk")
