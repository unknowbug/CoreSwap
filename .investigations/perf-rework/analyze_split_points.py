"""C1: 分析 final_density 树的两棵子树（interp_5 factor 链 vs noodle 链）的函数归属，
决定拆 shader 的切分点。hook gen_shader 记录每个 spline/registry/interp 节点的依赖。
"""
import json, sys, os, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen

DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]

g = dfc_gen.DfcGen(DFDIR, NDIR)
# 跑完整生成
g.gen(fd)
g.gen_shader(fd)

print("=== eval_density 顶层结构 ===")
print("min(arg1, arg2): arg1=squeeze(0.64*interp_5), arg2=noodle")
print()
print("=== interp 实例 ===")
for idx, samples in g.interp_funcs:
    print(f"  interp_{idx}: {len(samples)} corner samples")
    # sample 里含 normal_noise_N(sIdx) 调用 → 提取引用
    refs = set()
    for s in samples:
        for m in re.finditer(r'(normal_noise_(\d+)|interp_noise_(\d+)|spline_eval\((\d+)|df_\w+|interp_(\d+))', s):
            refs.add(m.group(0))
    print(f"    refs: {sorted(refs)[:8]} ...")
print()
print("=== registry 函数 ===")
for fname, fexpr in g.registry_defs:
    refs = set(re.findall(r'(normal_noise_\d+|interp_noise_\d+|interp_\d+|spline_eval\(\d+|df_\w+)', fexpr))
    print(f"  {fname}: {sorted(refs)[:10]}")
print()
print("=== spline coords (4 种) ===")
for ct, expr in enumerate(g.spline_coords):
    print(f"  coordType {ct}: {expr[:100]}")
