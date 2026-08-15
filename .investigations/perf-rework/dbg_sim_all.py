# 模拟全 1024 点输出（与 e2e coords 一致）→ sim_all1024.txt
import json, importlib.util, sys, runpy
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sim = runpy.run_path(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dbg_full_sim.py')
eval_df = sim['eval_df']; nodes = sim['nodes']
out = []
for i in range(1024):
    x = i % 64; y = -64 + (i // 64 % 16); z = 0
    r = eval_df(len(nodes) - 1, i, x, y, z)
    out.append('%d %.9f' % (i, r))
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\sim_all1024.txt', 'w', encoding='utf-8').write('\n'.join(out))
print('done', len(out))
