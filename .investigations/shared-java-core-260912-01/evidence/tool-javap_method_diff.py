#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Method-level javap comparison (260912-01 V1b/V3 evidence helper).

usage: python javap_method_diff.py <pre.javap.txt> <post.javap.txt> [--verbose]

Normalizes each `javap -c -p` method body by stripping bytecode offsets and
constant-pool / bootstrap indices, so that a method that is instruction-for-
instruction identical still compares equal even when its offsets shifted.
Reports per method: SAME / CHANGED / ADDED / REMOVED, plus the mnemonic
sequence length. Exit code 0 always (classification tool, not a gate).
"""
import io
import re
import sys

HDR = re.compile(r"^  (?!\s)[^ ].*\(.*\).*;$")
OFF = re.compile(r"^\s*[0-9a-f]+:\s*")
IDX = re.compile(r"#\d+")


def norm_line(line):
    line = line.rstrip()
    line = OFF.sub("", line)
    line = IDX.sub("#", line)
    line = re.sub(r"\b[0-9a-f]+(?=\s*//)", "OFF", line)
    # constant-pool width artifacts: javac picks ldc_w/goto_w purely by index width
    line = re.sub(r"\bldc_w\b", "ldc", line)
    line = re.sub(r"\bldc2_w\b", "ldc2", line)
    line = re.sub(r"\bgoto_w\b", "goto", line)
    return line.strip()


def parse(path):
    methods = {}
    cur = None
    with io.open(path, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.rstrip("\n")
            if HDR.match(line):
                cur = line.strip()
                methods.setdefault(cur, [])
                continue
            if cur is not None:
                if line.startswith("  ") or line == "":
                    s = norm_line(line)
                    if s:
                        methods[cur].append(s)
                else:
                    cur = None
    return methods


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    pre, post = parse(argv[0]), parse(argv[1])
    verbose = "--verbose" in argv
    names = list(pre.keys()) + [k for k in post if k not in pre]
    same = changed = added = removed = 0
    print("method-level instruction comparison (offsets/constant indices normalized)")
    print("pre methods=%d  post methods=%d" % (len(pre), len(post)))
    for name in names:
        a, b = pre.get(name), post.get(name)
        if a is None:
            added += 1
            print("[ADDED  ] %s  (instr=%d)" % (name, len(b)))
        elif b is None:
            removed += 1
            print("[REMOVED] %s  (instr=%d)" % (name, len(a)))
        elif a == b:
            same += 1
            if verbose:
                print("[SAME   ] %s  (instr=%d)" % (name, len(a)))
        else:
            changed += 1
            diffs = sum(1 for x, y in zip(a, b) if x != y) + abs(len(a) - len(b))
            print("[CHANGED] %s  (pre instr=%d post instr=%d, differing lines=%d)"
                  % (name, len(a), len(b), diffs))
            if verbose:
                for x, y in zip(a, b):
                    if x != y:
                        print("      pre : %s" % x)
                        print("      post: %s" % y)
    print("summary: same=%d changed=%d added=%d removed=%d" % (same, changed, added, removed))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
