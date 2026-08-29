import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
coords = [tuple(map(int, l.split())) for l in open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\coords_dump.txt')]
xs = sorted({c[0] for c in coords})
ys = sorted({c[1] for c in coords})
zs = sorted({c[2] for c in coords})
print("N samples:", len(coords))
print("x range:", xs[0], "..", xs[-1], " unique x count:", len(xs))
print("y range:", ys[0], "..", ys[-1], " unique y count:", len(ys))
print("z range:", zs[0], "..", zs[-1], " unique z count:", len(zs))
# chunk coverage
chunks = sorted({(c[0]//16, c[2]//16) for c in coords})
print("distinct (chunkX,chunkZ):", len(chunks), chunks[:20])
# y levels (minY=-64)
ysabs = sorted({c[1]+64 for c in coords})  # gy
print("gy (y+64) unique:", ysabs)
print("gy//8 (cy) unique:", sorted({c[1]//8 for c in coords}))
# gx coverage per x
print("gx (x%16) unique:", sorted({c[0]%16 for c in coords}))
print("gz (z%16) unique:", sorted({c[2]%16 for c in coords}))
print("cx (gx//4) unique:", sorted({(c[0]%16)//4 for c in coords}))
print("cz (gz//4) unique:", sorted({(c[2]%16)//4 for c in coords}))
print("cy (gy//8) unique:", sorted({(c[1]+64)//8 for c in coords}))
