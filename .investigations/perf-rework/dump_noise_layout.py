# dump_noise_layout.py —— 确认 noise_instances 注册顺序（是否按 8 角点展开）与 slot 映射
import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
print('noise_instances total =', len(g.noise_instances))
print('noise_slots =', len(g.noise_slots))
print('--- noise_instances 前 24 个 (kind, params 摘要) ---')
for i, (kind, p) in enumerate(g.noise_instances[:24]):
    key = p.get('_key', p.get('noise', ''))
    suffix = ''
    # 从 key 提取角点后缀 @cN
    import re
    m = re.search(r'@c(\d)', key)
    if m: suffix = f" corner={m.group(1)}"
    print(f'  [{i}] {kind} key={key[:60]}{suffix}')
print('--- noise_slots 前 10 (base/stride) ---')
for i, s in enumerate(g.noise_slots[:10]):
    print(f'  slot[{i}] base={s["base"]} stride={s["stride"]}')
print('--- 全部 noise_slots（base → 实例区间） ---')
for i, s in enumerate(g.noise_slots):
    print(f'  slot[{i}] base={s["base"]} -> 实例 {s["base"]}..{s["base"]+7}')
print('--- 实例 48..64（slot 6-8） ---')
for i in range(48, 64):
    kind, p = g.noise_instances[i]
    print(f'  [{i}] {kind} key={p.get("_key","")[:50]}')
for i in range(160, 200):
    kind, p = g.noise_instances[i]
    ci = g.normal_chain_index.get(p.get('_key',''))
    if ci is not None and ci < len(g.coord_chains):
        c = g.coord_chains[ci]
        print(f'  [{i}] key={p.get("_key","")[:40]} chain_idx={ci} type={c.get("type")} xz_scale={c.get("xz_scale")} y_scale={c.get("y_scale")} flat_cache={c.get("flat_cache")} shift={c.get("shift_x",{}).get("type","")}')
    else:
        print(f'  [{i}] key={p.get("_key","")[:40]} chain_idx={ci} (MISSING)')
print('--- coord_chains 前 8 ---')
for i, c in enumerate(g.coord_chains[:8]):
    print(f'  [{i}] {c}')
