# check_op_vs_sim.py —— 验证 op_noise GPU 逻辑 vs sim normal_noise(0)（同源）
# 用 sim 的 normal_noise(0, sIdx) 复刻 op_noise 的输入（split 行 + perm）→ 对比 op_probe 的 CPU 参照
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

# 复刻 op_noise 的 GPU 逻辑：读 split 行（实例 0 splitBase=0, n=9, 12 floats/octave 两套）
# 用 sim 的 splitCoord + NORMAL[0] 参数，手动实现 op_noise 的循环
import math

def op_noise_replica(sIdx):
    mm = sim.NORMAL[0]
    n = mm['n']; octBase = mm['octBase']; splitBase = mm['splitBase']
    persistence = mm['persistence']; amplitude = mm['amplitude']; amps = mm['amps']
    # 读 sim.splitCoord（sIdx 索引全量）
    d = 0.0; f = persistence
    for i in range(n):
        b = sIdx * sim.SPLIT_TOTAL + splitBase + i*6
        ix = int(sim.splitCoord[b]); iy = int(sim.splitCoord[b+1]); iz = int(sim.splitCoord[b+2])
        gx = sim.splitCoord[b+3]; gy = sim.splitCoord[b+4]; gz = sim.splitCoord[b+5]
        ns = sim.pn_sample3_f32(octBase+i, ix, iy, iz, gx, gy, gz)
        d += amps[i] * ns * f
        f /= 2.0
    d2 = 0.0; f = persistence
    for i in range(n):
        b = sIdx * sim.SPLIT_TOTAL + splitBase + 6*n + i*6
        ix = int(sim.splitCoord[b]); iy = int(sim.splitCoord[b+1]); iz = int(sim.splitCoord[b+2])
        gx = sim.splitCoord[b+3]; gy = sim.splitCoord[b+4]; gz = sim.splitCoord[b+5]
        ns = sim.pn_sample3_f32(octBase+n+i, ix, iy, iz, gx, gy, gz)
        d2 += amps[i] * ns * f
        f /= 2.0
    return (d + d2) * amplitude

# 对比 sim.normal_noise(0, sIdx)（sim 官方实现）
print('sim normal_noise(0) vs 复刻（同源应一致）：')
for sIdx in range(5):
    a = sim.normal_noise(0, sIdx)
    b = op_noise_replica(sIdx)
    print(f'  sIdx={sIdx} normal_noise={a:.6f} replica={b:.6f} diff={abs(a-b):.2e}')
