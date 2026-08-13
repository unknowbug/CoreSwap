import sys, re, statistics

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

def analyze(path):
    lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    segs = []
    pending = []
    b22 = 0
    for ln in lines:
        m = re.search(r'\[A\] threads=\s*(\d+)', ln)
        if m:
            segs.append(("[A] threads=" + m.group(1), pending))
            pending = []
            continue
        m = re.search(r'\[B\] workers=\s*(\d+)', ln)
        if m:
            if m.group(1) == "22":
                b22 += 1
                label = "[B] workers=22#" + str(b22)
            else:
                label = "[B] workers=" + m.group(1)
            segs.append((label, pending))
            pending = []
            continue
        if "[B] 模拟实机" in ln or "[BENCH]" in ln:
            pending = []
            continue
        m = re.search(r'\[PROF\] chunk\([^)]*\): density=([\d.]+)ms', ln)
        if m:
            pending.append(float(m.group(1)))
    if pending:
        segs.append(("(tail)", pending))

    print("=== density wall (ms) per thread config ===")
    print(f"{'segment':20s} {'n':>4s} {'median':>8s} {'mean':>8s} {'min':>8s} {'max':>8s}")
    for label, vals in segs:
        if not vals:
            print(f"{label:20s} {'0':>4s}")
            continue
        med = statistics.median(vals)
        mean = statistics.mean(vals)
        print(f"{label:20s} {len(vals):4d} {med:8.1f} {mean:8.1f} {min(vals):8.1f} {max(vals):8.1f}")

if __name__ == "__main__":
    analyze(sys.argv[1])
