#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
V1 等价门工具：两份 jar 条目级 sha256 清单对拍（CoreSwap 共享 Java 适配核抽取 260912-01）

用法:
    python jar_manifest_diff.py <pre.tsv> <post.tsv> [--expect <regex>]... [--out <report.txt>]

判定规则（计划 §5 V1）:
    - 条目按名字对齐；逐条目比 sha256。
    - 命中 --expect 正则的差异条目归入「预期差异」（如 mixin/ 类、refmap），
      但**仍然全部列出**——「预期」不等于「豁免」，审查者必须看到全量清单。
    - 退出码: 0 = 无非预期差异; 1 = 存在非预期差异; 2 = 用法错误。

输出报告含: jar sha / 条目数 / classes 数 / 非 META-INF 数 / 相同数 / 差异数 /
差异明细（OLD/NEW sha + 条目名 + 归类）/ 新增条目 / 删除条目 / 结论。
"""
import io
import re
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")


def load(path):
    meta = {}
    rows = {}
    with io.open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if not line:
                continue
            if line.startswith("#"):
                parts = line[1:].strip().split(None, 1)
                if len(parts) == 2:
                    meta[parts[0]] = parts[1]
                continue
            digest, name = line.split("\t", 1)
            rows[name] = digest
    return meta, rows


def main(argv):
    expect = []
    out_path = None
    args = []
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == "--expect":
            i += 1
            expect.append(re.compile(argv[i]))
        elif a == "--out":
            i += 1
            out_path = argv[i]
        else:
            args.append(a)
        i += 1
    if len(args) != 2:
        print(__doc__)
        return 2

    pre_meta, pre = load(args[0])
    post_meta, post = load(args[1])

    same, changed, added, removed = [], [], [], []
    for name, digest in sorted(pre.items()):
        if name in post:
            if post[name] == digest:
                same.append(name)
            else:
                changed.append((name, digest, post[name]))
        else:
            removed.append(name)
    for name in sorted(post):
        if name not in pre:
            added.append(name)

    def classify(name):
        for rx in expect:
            if rx.search(name):
                return "EXPECTED"
        return "UNEXPECTED"

    unexpected = [c for c in changed if classify(c[0]) == "UNEXPECTED"]

    L = []
    L.append("# V1 jar 条目级对拍（CoreSwap 共享 Java 适配核抽取 260912-01）")
    L.append("# pre  = %s  jar_sha256=%s entries=%s classes=%s non_meta=%s"
             % (args[0], pre_meta.get("jar_sha256"), pre_meta.get("entries"),
                pre_meta.get("classes"), pre_meta.get("non_meta")))
    L.append("# post = %s  jar_sha256=%s entries=%s classes=%s non_meta=%s"
             % (args[1], post_meta.get("jar_sha256"), post_meta.get("entries"),
                post_meta.get("classes"), post_meta.get("non_meta")))
    L.append("# expect_patterns = %s" % (expect and [r.pattern for r in expect] or []))
    L.append("")
    L.append("相同条目 = %d" % len(same))
    L.append("差异条目 = %d（预期 %d / 非预期 %d）"
             % (len(changed), len(changed) - len(unexpected), len(unexpected)))
    L.append("新增条目 = %d" % len(added))
    L.append("删除条目 = %d" % len(removed))
    L.append("")
    if changed:
        L.append("## 差异明细（全量，逐条归类）")
        for name, a, b in changed:
            L.append("[%s] %s\n    pre =%s\n    post=%s" % (classify(name), name, a, b))
        L.append("")
    if added:
        L.append("## 新增条目")
        for name in added:
            L.append("+ %s  sha=%s" % (name, post[name]))
        L.append("")
    if removed:
        L.append("## 删除条目")
        for name in removed:
            L.append("- %s  sha(pre)=%s" % (name, pre[name]))
        L.append("")
    L.append("## 结论")
    L.append("V1 verdict = %s" % ("PASS（无非预期差异）" if not unexpected and not added and not removed
                                  else "FAIL（存在非预期差异/增删条目，须逐条声明或修正）"))
    text = "\n".join(L) + "\n"
    if out_path:
        with io.open(out_path, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        print("[OK] wrote %s" % out_path)
    else:
        print(text)
    return 0 if (not unexpected and not added and not removed) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
