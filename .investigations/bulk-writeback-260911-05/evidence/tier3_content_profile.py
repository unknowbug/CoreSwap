# -*- coding: utf-8 -*-
"""C 线 Tier 3 附测：非接管 chunk 是否**完整生成**（影响 §5 切片解读与覆盖声明）。

廉价判据（不解码 4096 位）：看每个 chunk 的 section 里 palette 是否只含 air。
  - 若某 chunk 所有 section 的 palette 都只有 air（或无 section）⇒ 未生成内容（partial/EMPTY 状态）
  - 若有非 air palette 条目 ⇒ 有内容
解析器复用 B1 已修正工具（exec 前缀，零复制）。
"""
import sys, os
from collections import Counter
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
ROOT = r"E:\PYTHON\CoreSwap"
TOOL = os.path.join(ROOT, ".investigations", "e5-recompute-260911-05", "diff_arms_fixed.py")
LOG = os.path.join(ROOT, ".investigations", "bulk-writeback-260911-05", "evidence", "raw-c-old.log")

src = open(TOOL, encoding="utf-8").read()
ns = {}
exec(compile(src[:src.index("van=load_world(sys.argv[1])")], TOOL, "exec"), ns)
load_world = ns["load_world"]

import re
RE_WB = re.compile(r"\[WG-CONTENT-WB\] chunk\((-?\d+),(-?\d+)\)")
take = set()
for ln in open(LOG, encoding="utf-8", errors="replace"):
    m = RE_WB.search(ln)
    if m:
        take.add((int(m.group(1)), int(m.group(2))))
print(f"[接管集] {len(take)} chunk")

W = load_world(os.path.join(ROOT, ".tmp", "c-260911-05", "world-old"))
print(f"[世界] {len(W)} chunk")


def profile(key):
    """返回 (section 总数, 有非 air palette 的 section 数, 非 air palette 条目总数)"""
    nsec = ncontent = nentries = 0
    for s in W[key].get("sections", []):
        bs = s.get("block_states")
        if not bs:
            continue
        nsec += 1
        pal = bs.get("palette") or []
        names = [e.get("Name", "?") if isinstance(e, dict) else str(e) for e in pal]
        nonair = [n for n in names if n != "minecraft:air"]
        if nonair:
            ncontent += 1
            nentries += len(nonair)
    return nsec, ncontent, nentries


for label, keys in (("切片A 走过接管", sorted(take & set(W))), ("切片B 未走过接管", sorted(set(W) - take))):
    if not keys:
        continue
    empty = 0; content = 0; tot_sec = 0; tot_cs = 0; tot_en = 0
    hist = Counter()
    for k in keys:
        ns_, nc_, ne_ = profile(k)
        tot_sec += ns_; tot_cs += nc_; tot_en += ne_
        if nc_ == 0:
            empty += 1
        else:
            content += 1
        hist[min(nc_, 8)] += 1
    n = len(keys)
    print(f"\n=== {label}（{n} chunk）===")
    print(f"  有内容 chunk = {content} ({100*content/n:.1f}%)   全 air/无 section chunk = {empty} ({100*empty/n:.1f}%)")
    print(f"  section 总数 = {tot_sec}   含非 air palette 的 section = {tot_cs} ({100*tot_cs/max(1,tot_sec):.1f}%)   非 air palette 条目 = {tot_en}")
    print(f"  每 chunk「有内容 section 数」直方图（截断到 8）: {dict(sorted(hist.items()))}")
