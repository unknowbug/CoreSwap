#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
V6 工具：「一个类只有一个家」双向核对（CoreSwap 共享 Java 适配核抽取 260912-01）

规则（计划 §3 D-1 / §5 V6）：
    - 每个类要么住在共享源 `java-core/src/main/java`，要么住在某版本的
      `versions/<ver>/java/src/main/java`，**不得两处同时存在**（重复 = 编译期歧义/双真相源）。
    - 本工具按「包路径 + 类名（含嵌套类文件的顶层归属）」求两集合的交集与差集。

用法:
    python class_home_check.py [--root E:\\PYTHON\\CoreSwap] [--shared java-core/src/main/java] [--out report.txt]

输出: 共享类数 / 每版分版类数 / 交叠类清单（应为空）/ 共享源中位于 mixin 包的类（应为空）/
      分版树中仍在 `wg/bench/` 下、且名字出现在共享清单里的残留重复文件。
退出码: 0 = 无交叠（PASS）; 1 = 存在交叠/违规; 2 = 用法错误。
"""
import io
import os
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

VERSIONS = ["1.20.1", "1.21.6"]


def rel_classes(root):
    """返回 {相对路径(去 .java): 绝对路径}"""
    out = {}
    if not os.path.isdir(root):
        return out
    for dirpath, _dirnames, filenames in os.walk(root):
        for fn in filenames:
            if not fn.endswith(".java"):
                continue
            full = os.path.join(dirpath, fn)
            out[os.path.relpath(full, root).replace("\\", "/")] = full
    return out


def main(argv):
    root = "E:\\PYTHON\\CoreSwap"
    shared_rel = "java-core/src/main/java"
    out_path = None
    i = 0
    while i < len(argv):
        if argv[i] == "--root":
            i += 1
            root = argv[i]
        elif argv[i] == "--shared":
            i += 1
            shared_rel = argv[i]
        elif argv[i] == "--out":
            i += 1
            out_path = argv[i]
        else:
            print(__doc__)
            return 2
        i += 1

    shared_root = os.path.join(root, *shared_rel.split("/"))
    shared = rel_classes(shared_root)
    per_ver = {v: rel_classes(os.path.join(root, "versions", v, "java", "src", "main", "java")) for v in VERSIONS}

    L = []
    L.append("# V6 「一个类只有一个家」核对（260912-01）")
    L.append("# root   = %s" % root)
    L.append("# shared = %s（%d 个 .java）" % (shared_rel, len(shared)))
    for v in VERSIONS:
        L.append("# %s = versions/%s/java/src/main/java（%d 个 .java）" % (v, v, len(per_ver[v])))
    L.append("")

    overlap_all = {}
    for v in VERSIONS:
        inter = sorted(set(shared) & set(per_ver[v]))
        overlap_all[v] = inter
    L.append("## 交叠类（共享 × 分版；应为空）")
    any_overlap = False
    for v in VERSIONS:
        if overlap_all[v]:
            any_overlap = True
            for name in overlap_all[v]:
                L.append("[%s] %s" % (v, name))
        else:
            L.append("[%s] （空）" % v)
    L.append("")

    L.append("## 共享源中位于 mixin 包的类（应为空；KB #12：mixin 包禁非 mixin 类）")
    viol = sorted(n for n in shared if "/mixin/" in n)
    L.extend(["[VIOLATION] " + n for n in viol] or ["（空）"])
    L.append("")

    L.append("## 共享类清单（%d）" % len(shared))
    for n in sorted(shared):
        L.append("  " + n)
    L.append("")
    L.append("## 分版独有类清单")
    for v in VERSIONS:
        only = sorted(set(per_ver[v]) - set(shared))
        L.append("[%s] %d 个" % (v, len(only)))
        for n in only:
            L.append("  " + n)
    L.append("")

    verdict = (not any_overlap) and (not viol)
    L.append("## 结论")
    L.append("V6 verdict = %s" % ("PASS（无交叠、无 mixin 包违规）" if verdict
                                  else "FAIL（交叠 %d 项 / mixin 包违规 %d 项）"
                                  % (sum(len(x) for x in overlap_all.values()), len(viol))))
    text = "\n".join(L) + "\n"
    if out_path:
        with io.open(out_path, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        print("[OK] wrote %s" % out_path)
    else:
        print(text)
    return 0 if verdict else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
