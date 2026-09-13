#!/usr/bin/env python3
"""cmp_260913-01.py — 三臂 × 三维 多重集对比（260913-01）

判据 C-1：三臂两两 × 三维，**两层各自**逐 chunk 多重集 diff = 0。
纪律（承 260911-05/260912-02）：
  - **序列不承载判据**（异步流水线顺序不确定）⇒ 只用 multiset
  - 两层（[WG-CONTENT] Rust buf 层 / [WG-CONTENT-WB] Java 读回层）**不得跨层比对**
  - 空载体不得报 diff=0 通过（#130）：common=0 时显式标 EMPTY-CARRIER
"""
import sys, os, glob
from collections import Counter

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

OUT = r"E:\PYTHON\CoreSwap\.tmp\d4i-260913-01"

DIMS = {"ow": "overworld", "nt": "the_nether", "en": "the_end"}
LAYERS = {"content": ".content.txt", "wb": ".wb.txt"}
# 三臂形态（正式九臂）
ARMS = ["default", "perblock", "bulk"]


def load(path):
    if not os.path.exists(path):
        return None
    with open(path, encoding="utf-8") as f:
        lines = [ln.strip() for ln in f if ln.strip()]
    return Counter(lines)


def compare(a_lines, b_lines):
    """返回 (common, only_a, only_b, verdict)"""
    if a_lines is None or b_lines is None:
        return (0, 0, 0, "MISSING-FILE")
    only_a = a_lines - b_lines
    only_b = b_lines - a_lines
    common = sum((a_lines & b_lines).values())
    n_only_a, n_only_b = sum(only_a.values()), sum(only_b.values())
    if common == 0 and n_only_a == 0 and n_only_b == 0:
        verdict = "EMPTY-CARRIER"          # 两边都空 ⇒ 拒绝出结论 (#130)
    elif n_only_a == 0 and n_only_b == 0:
        verdict = "EQUAL"
    else:
        verdict = "DIFF"
    return (common, n_only_a, n_only_b, verdict)


def main():
    print("=" * 100)
    print("260913-01 判据 C-1：三臂两两 × 三维 × 两层 多重集对比")
    print("=" * 100)
    all_ok = True
    for dk, dname in DIMS.items():
        for lk, lsuf in LAYERS.items():
            print(f"\n--- dim={dname:12s} layer={lk:8s} ---")
            files = {a: os.path.join(OUT, f"fp-1.21.6-{dk}-{a}{lsuf}") for a in ARMS}
            data = {a: load(p) for a, p in files.items()}
            for a in ARMS:
                n = len(data[a]) if data[a] is not None else -1
                print(f"    {a:10s} lines={n}")
            pairs = [("default", "perblock"), ("default", "bulk"), ("perblock", "bulk")]
            for a, b in pairs:
                common, oa, ob, verdict = compare(data[a], data[b])
                mark = "OK " if verdict == "EQUAL" else "!! "
                if verdict != "EQUAL":
                    all_ok = False
                print(f"    {mark}{a:10s} vs {b:10s}: common={common:5d} only_a={oa:5d} only_b={ob:5d}  => {verdict}")
    print("\n" + "=" * 100)
    print(f"总判定 C-1 = {'满足 (全部 EQUAL)' if all_ok else '未满足 / 需核查'}")
    print("=" * 100)
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
