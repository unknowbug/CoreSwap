# dump_instance_params.py —— 输出目标实例（continentalness@c0）的参数
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

for idx in [0, 1, 2, 3, 8, 16, 24, 32, 33, 34]:
    if idx in sim.NORMAL:
        mm = sim.NORMAL[idx]
        print(f'NORMAL[{idx}]: n={mm["n"]} octBase={mm["octBase"]} splitBase={mm["splitBase"]} persistence={mm.get("persistence")} amplitude={mm.get("amplitude")}')
        print(f'  amps={mm.get("amps")}')
print('---')
# 实例 0-3 都是 continentalness@c0..c3？确认
for idx in range(4):
    print(f'inst[{idx}] = {sim.g.noise_instances[idx]}')
