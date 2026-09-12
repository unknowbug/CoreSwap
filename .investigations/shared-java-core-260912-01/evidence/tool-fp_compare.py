#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Multiset comparison of gate lines (260912-01 V3 helper).

usage: python fp_compare.py <pre.log> <post.log> <tag-regex>

Extracts lines matching the tag regex (e.g. r"\[WG-CONTENT-WB\] chunk"), normalizes
away the log prefix, and compares the multisets. Prints counts and the first few
symmetric differences. Also reports the sorted-sequence diff (order-sensitive) so a
"same set, different order" outcome is visible.
"""
import io
import re
import sys
from collections import Counter


def load(path, tag):
    rx = re.compile(tag)
    out = []
    with io.open(path, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            m = rx.search(line)
            if m:
                out.append(line[m.start():].strip())
    return out


def main(argv):
    pre_log, post_log, tag = argv[0], argv[1], argv[2]
    a, b = load(pre_log, tag), load(post_log, tag)
    ca, cb = Counter(a), Counter(b)
    only_a = sorted((ca - cb).elements())
    only_b = sorted((cb - ca).elements())
    print("tag        = %s" % tag)
    print("pre lines  = %d   post lines = %d" % (len(a), len(b)))
    print("common     = %d" % len(list((ca & cb).elements())))
    print("only-pre   = %d   only-post  = %d" % (len(only_a), len(only_b)))
    print("multiset identical = %s" % ("YES" if not only_a and not only_b else "NO"))
    print("sorted-sequence identical = %s" % ("YES" if sorted(a) == sorted(b) else "NO"))
    if not (len(a) == len(b) and sorted(a) == sorted(b)):
        print("raw-sequence identical = %s (order differs or count differs)" %
              ("YES" if a == b else "NO"))
    for label, items in (("only-pre ", only_a), ("only-post", only_b)):
        for x in items[:5]:
            print("  %s %s" % (label, x))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
