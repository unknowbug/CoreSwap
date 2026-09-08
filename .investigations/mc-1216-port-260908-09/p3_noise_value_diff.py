# 260908-09 P2/P3: value-level noise_params diff (parsed, not string) 1.20.1 vs 1.21.6
import sys, json
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
a = json.load(open(r"E:\PYTHON\CoreSwap\versions\1.20.1\data\noise_params.json", encoding="utf-8"))
b = json.load(open(r"E:\PYTHON\CoreSwap\versions\1.21.6\data\noise_params.json", encoding="utf-8"))
diffs = []
for k in sorted(set(a) | set(b)):
    va, vb = a.get(k), b.get(k)
    if va != vb:
        diffs.append((k, va, vb))
print(f"noise_params value diffs: {len(diffs)}")
for k, va, vb in diffs:
    print(" ", k, "\n    1.20.1:", va, "\n    1.21.6:", vb)
