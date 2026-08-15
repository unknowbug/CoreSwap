# check_sim_vs_e2e.py —— sim（dbg_full_sim.py）全量对拍最新 e2e-A5 输出（D23 修复后 GPU）
import re, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

# 读 e2e-A5 输出中的 gpu 值
gpu = {}
for line in open(r'cmd-output\e2e-A5-20260815-135509.txt', encoding='utf-8'):
    m = re.match(r'\[DBG\] i=(\d+) pos=\(([-\d]+),([-\d]+),([-\d]+)\) gpu=([-\d.e]+) cpu=([-\d.e]+)', line)
    if m:
        i = int(m.group(1)); gpu[i] = (float(m.group(5)), float(m.group(6)))
print('e2e gpu points:', len(gpu))

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

maxd = 0.0; worst = None; bad = 0; n = 0
for i in sorted(gpu):
    x, y, z = sim.coords[i]
    r = sim.eval_df(sim.N - 1, i, x, y, z)
    if isinstance(r, tuple):
        print('ERR', i, r); continue
    d = abs(r - gpu[i][0])
    n += 1
    if d > maxd: maxd = d; worst = (i, x, y, z, r, gpu[i][0])
    if d > 1e-4: bad += 1
print(f'sim vs e2e gpu: n={n} maxDiff={maxd:.3e} bad(>1e-4)={bad}')
if worst: print('worst:', worst)
