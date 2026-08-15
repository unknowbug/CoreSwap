import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
g.gen(fd)
g.gen_shader(fd)
from collections import Counter
cnt = Counter(idx for idx, _ in g.node_funcs)
print("idx -> 出现次数 (前 10):", cnt.most_common(10))
# 检查 body 是否相同（重复 idx 的 body）
first_dup = [i for i, c in cnt.items() if c > 1][0]
bodies = [body for idx, body in g.node_funcs if idx == first_dup]
print(f"idx {first_dup} 的 {len(bodies)} 个 body 是否相同:", all(b == bodies[0] for b in bodies))
if len(bodies) >= 2 and bodies[0] != bodies[1]:
    print("  body0:", bodies[0][:100])
    print("  body1:", bodies[1][:100])
