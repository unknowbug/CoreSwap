import sys, re, os, importlib.util
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
src = open(r"E:\PYTHON\CoreSwap\.investigations\e5-recompute-260911-05\diff_arms_fixed.py", encoding="utf-8").read()
src = src.split("van=load_world")[0]
g = {}
exec(compile(src, "daf", "exec"), g)

# 定点段坐标（与工具内嵌同表）
pts = [(-522,63,139),(-805,57,138),(-500,6,240),(-874,48,-588),(-296,48,-18),
       (-443,60,311),(-556,60,316),(-71,56,-517),(-1531,61,239),(-471,45,-215),
       (-592,42,216),(-527,52,78),(-424,62,74),(-196,47,295)]

def load(d):
    w = {}
    for fn in os.listdir(d):
        if not fn.endswith(".mca"):
            continue
        for nbt in g["iter_chunks"](os.path.join(d, fn)):
            data = nbt.get("") if isinstance(nbt, dict) and "" in nbt else nbt
            if not isinstance(data, dict):
                continue
            p = (data.get("xPos"), data.get("zPos"))
            if p[0] is None:
                continue
            w[p] = data
    return w

def block_at(chunk, x, y, z):
    cy = y >> 4
    for sec in chunk.get("sections", []):
        if sec.get("Y") == cy:
            bs = sec.get("block_states")
            if bs is None:
                return "minecraft:air"
            pal = bs["palette"]; data = bs.get("data")
            ly, lz, lx = y & 15, z & 15, x & 15
            if data is None:
                e = pal[0]; return e.get("Name", "?") if isinstance(e, dict) else str(e)
            idx = lx + lz * 16 + ly * 256
            bits = max(4, (len(pal) - 1).bit_length()); per = 64 // bits
            v = (data[idx // per] >> ((idx % per) * bits)) & ((1 << bits) - 1)
            e = pal[v]; return e.get("Name", "?") if isinstance(e, dict) else str(e)
    return "minecraft:air"

d = r"E:\PYTHON\CoreSwap\.tmp\hang-repro-260910\arms"
van = load(os.path.join(d, "region-vanilla"))
cs = load(os.path.join(d, "region-coreswap"))
common = set(van) & set(cs)
print(f"common={len(common)}")
n = 0; nonair = 0; agree = 0; both_air = 0
for (x, y, z) in pts:
    cx, cz = x >> 4, z >> 4
    if (cx, cz) not in common:
        continue
    for yy in range(y - 2, y + 3):
        n += 1
        bv = block_at(van[(cx, cz)], x & 15, yy, z & 15)
        bc = block_at(cs[(cx, cz)], x & 15, yy, z & 15)
        if bv == bc:
            agree += 1
            if bv == "minecraft:air":
                both_air += 1
        if not (bv == "minecraft:air" and bc == "minecraft:air"):
            nonair += 1
print(f"定点样本总数={n}  两侧一致={agree}  其中两侧都 air={both_air}  至少一侧非 air={nonair}")
print("=> 旧读法把水/深层岩误读为 air 的样本数（估计）= 规范读法非空气样本 - 旧读法非空气样本")
