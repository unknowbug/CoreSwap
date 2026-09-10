import re, sys, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
base = r"E:\PYTHON\CoreSwap\.tmp\perf-reg-260910-06\logs"
rx = re.compile(r"\[WG-CONTENT\] chunk\((-?\d+),(-?\d+)\) hash=([0-9a-f]+) nz=(\d+)")
def load(tag):
    d = collections.defaultdict(set)
    nz = {}
    for line in open(f"{base}\\{tag}.log", encoding="utf-8", errors="replace"):
        m = rx.search(line)
        if m:
            k = (int(m.group(1)), int(m.group(2)))
            d[k].add(m.group(3)); nz[k] = int(m.group(4))
    return d, nz
a, an = load("osS-r2")   # sync
b, bn = load("oaS-r2")   # async
print(f"sync  chunks={len(a)}  async chunks={len(b)}")
multi_a = {k: v for k, v in a.items() if len(v) > 1}
multi_b = {k: v for k, v in b.items() if len(v) > 1}
print(f"within-arm multiple-distinct-hash: sync={len(multi_a)} async={len(multi_b)}")
common = sorted(set(a) & set(b))
only_a = len(set(a) - set(b)); only_b = len(set(b) - set(a))
mm = [k for k in common if a[k] != b[k]]
print(f"common={len(common)}  only_sync={only_a}  only_async={only_b}  hash_mismatch={len(mm)}")
if mm:
    for k in mm[:10]:
        print("  MISMATCH", k, "sync=", a[k], "async=", b[k], "nz", an[k], bn[k])
else:
    print("=> 逐 chunk 原生输出指纹 100% 相同（common 集内）")
