# design_split_grid.py — 分析 split 布局，为「grid 缓存 split 组织」设计提供数据
# 核心问题：grid 1225 节点若要各自 split 是否爆量？8 角点展开在 splitTotal 里占多少？
# 是否能让「同一 cell」的 split 共享（避免每节点独立 splitTotal）？
import struct, json, math, os, collections
sys_stdout = None
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
SPLIT_TOTAL = 8672

# 读 split_dump / coords
splitCoord = struct.unpack('f' * (SPLIT_TOTAL * 1024), open(base + r'\split_dump.bin', 'rb').read())
coords = [tuple(map(int, l.split())) for l in open(base + r'\coords_dump.txt')]
print(f"splitCoord: {len(splitCoord)} floats = {SPLIT_TOTAL} x {len(coords)} samples")
print(f"coords: {len(coords)} samples")

# 关键问题1：同一 cell 的采样点，其 split 数据是否共享（是否一致）？
# cell 定义：一个 cell = (chunkX, cellY, chunkZ, cx, cy, cz)，代表 128 block
def cell_of(x, y, z):
    cx, cz = x // 16, z // 16
    gx, gy, gz = x - cx*16, y + 64, z - cz*16
    return (cx, gx // 4, gy // 8, cz, gz // 4)

# 对每个采样点，找其 cell
cell_samples = collections.defaultdict(list)
for i, (x, y, z) in enumerate(coords):
    cell_samples[cell_of(x, y, z)].append(i)
print(f"\n采样点数 {len(coords)}, 不同 cell 数 {len(cell_samples)}")
print(f"每 cell 采样点分布: {collections.Counter(len(v) for v in cell_samples.values())}")

# 关键问题2：同一 cell 的采样点，split 数据是否相同（每点的 splitTotal 是否重复）？
# 取 cell=0 的多个采样点，比较其 split 数据段
def sample_split(i):
    off = i * SPLIT_TOTAL
    return splitCoord[off:off+SPLIT_TOTAL]

# 找同一 cell 至少 2 个采样点的 cell
shared_cells = {k: v for k, v in cell_samples.items() if len(v) >= 2}
print(f"\n同一 cell 含 >=2 采样点的 cell 数: {len(shared_cells)} / {len(cell_samples)}")
# 比较这些 cell 的各采样点 split 段是否一致
same = 0; diff_cells = 0
for ckey, idxs in list(shared_cells.items())[:8]:
    seg0 = sample_split(idxs[0])
    allsame = all(seg0 == sample_split(i) for i in idxs[1:])
    if allsame: same += 1
    else: diff_cells += 1
print(f"检查 {min(8, len(shared_cells))} 个共享 cell: split 段全部相同={same}, 有差异={diff_cells}")

# 关键问题3：不同 cell 的 split 段是否不同（cell 间是否需独立 split）？
# 比较 cell A 与 cell B 的 split 段
cell_keys = list(cell_samples.keys())[:6]
print(f"\n6 个不同 cell 的 split 段是否互异:")
for i in range(min(4, len(cell_keys))):
    for j in range(i+1, min(5, len(cell_keys))):
        segi = sample_split(cell_samples[cell_keys[i]][0])
        segj = sample_split(cell_samples[cell_keys[j]][0])
        neq = sum(1 for a, b in zip(segi, segj) if a != b)
        print(f"  cell{i} vs cell{j}: 不同 float 数={neq} / {SPLIT_TOTAL}", 
              "(完全相同)" if neq == 0 else "(有差异)")
