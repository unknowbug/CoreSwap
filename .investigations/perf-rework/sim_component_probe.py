# sim_component_probe.py —— D23 生成器侧二分：sim 逐分量 vs CPU 参照分量
# 对错点 (784,160,-408)，对比 sim 的 normal_noise/interp_noise/spline vs wg_sample_named 参照。
# 定位生成器求值错误环节（噪声采样 / interp 角点 / 插值）。
import sys, os, json, struct, subprocess
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import importlib.util
spec = importlib.util.spec_from_file_location('sim', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
sim.splitCoord = struct.unpack('f' * 8672, open(base + r'\split_single.bin', 'rb').read())
sim.coords = [(784, 160, -408)]
sim.SPLIT_TOTAL = 8672
px, py, pz = 784, 160, -408

# 参照：用 wg_sample_named（worldgen DensityBuilder）——编译好的 ref_named_probe 或直接构造？
# 方案：调用一个小的 C++ 探针获取参照分量。这里先打印 sim 各噪声实例输出，供对比。
print('=== sim 噪声实例采样（corner=0） ===')
for idx in [0, 8, 16, 24, 32, 40, 152]:
    v = sim.normal_noise(idx, 0)
    print(f'  normal_noise({idx}) = {v:.9f}')
print('=== sim interp 角点 delegate（interp_4 root, corner 0..7） ===')
minY = -64
chunkX = px // 16; chunkZ = pz // 16
gx = px - chunkX * 16; gy = py - minY; gz = pz - chunkZ * 16
cx = gx // 4; cy = gy // 8; cz = gz // 4
root = sim.g.interp_roots[4]
for c in range(8):
    dx, dy, dz = c & 1, (c >> 1) & 1, (c >> 2) & 1
    ax = chunkX * 16 + (cx + dx) * 4
    ay = minY + (cy + dy) * 8
    az = chunkZ * 16 + (cz + dz) * 4
    v = sim.eval_df_base(root, c, 0, ax, ay, az)
    print(f'  corner{c} ({ax},{ay},{az}) = {v if isinstance(v, float) else v}')
print('=== sim interp_4 最终 ===')
r4 = sim.interp_N(4, 0, px, py, pz)
print(f'  interp_4 = {r4}')
print('=== sim spline_coord（corner 0） ===')
for ct in range(4):
    try:
        v = sim.spline_coord_py(ct, 0, 0, px, py, pz)
        print(f'  coordType{ct} = {v}')
    except Exception as e:
        print(f'  coordType{ct} err: {e}')
