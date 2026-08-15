"""C1b: 建立 noise 实例 → 归属（factor 链 vs noodle 链）映射。
hook gen_shader 记录：interp_5（factor）的角点采样引用哪些噪声，
df_overworld_caves_noodle 的 interp_1-4 引用哪些。用 spline_coord 也归属 factor。
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
g.gen(fd)
shader = g.gen_shader(fd)

# 读生成的 shader：每个 interp_N 函数体的角点采样里引用的噪声
# 从 g.interp_funcs 拿 samples 文本，提取噪声引用
factor_noises = set()   # interp_5 + interp_0（若被 factor 引用）的噪声
noodle_noises = set()   # interp_1..4 的噪声
all_interp_refs = {}

for idx, samples in g.interp_funcs:
    refs = set()
    for s in samples:
        for m in re.finditer(r'(normal_noise_(\d+)|interp_noise_(\d+))', s):
            refs.add(m.group(0))
    all_interp_refs[idx] = refs
    print(f"interp_{idx}: {len(refs)} noise refs, sample={sorted(refs)[:6]}...")

# factor 链 = interp_5（eval_density 引用）+ spline_coord 引用的噪声
# noodle 链 = interp_1..4（df_overworld_caves_noodle 引用）
factor_interps = {5}
noodle_interps = {1, 2, 3, 4}

# interp_0 被谁引用？搜 shader 文本
for m in re.finditer(r'(?:^|\W)(interp_\d)\(', shader):
    pass
callers = {}
for idx in all_interp_refs:
    callers[idx] = []
for m in re.finditer(r'(\w+)\([^)]*interp_(\d)\(', shader):
    caller, callee = m.group(1), int(m.group(2))
    if caller not in ('float',):
        callers[callee].append(caller)

print("\n=== interp callers ===")
for idx in sorted(all_interp_refs):
    print(f"  interp_{idx} called by: {sorted(set(callers[idx]))}")

# 归属
for idx, refs in all_interp_refs.items():
    if idx in factor_interps:
        factor_noises |= refs
    elif idx in noodle_interps:
        noodle_noises |= refs

# spline_coord 引用的噪声也归 factor（spline 在 factor 链）
for m in re.finditer(r'case \d+: \((normal_noise_(\d+)|[^)]*normal_noise_(\d+))', shader):
    pass
# 简单：搜 spline_coord 函数体的噪声引用
sc_start = shader.find('float spline_coord(')
sc_end = shader.find('\n}', sc_start)
sc_body = shader[sc_start:sc_end]
for m in re.finditer(r'(normal_noise_\d+)', sc_body):
    factor_noises.add(m.group(0))

print(f"\nfactor 链噪声数: {len(factor_noises)}")
print(f"noodle 链噪声数: {len(noodle_noises)}")
print(f"交集: {factor_noises & noodle_noises}")
print(f"总数: {len(factor_noises | noodle_noises)}")
