# check_interp_bandwidth.py —— 估算方案 C（GPU 网格角点）带宽
# 5 个 interp × 1225 网格角点/chunk；每点需要的 split floats = 该 interp 内容树的噪声实例数 × 每实例 floats
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

nodes = sim.nodes
N = len(nodes)

# 每个 interp delegate_root 的闭包，收集其 DF_NOISE/DF_SHIFTED_NOISE/DF_OLD_BLENDED 的噪声 slot
# （每噪声实例 split 需要多少 floats？看 split_total=8672 对应 200 实例——查 sim 的 split 布局）
# CpuBackend.split 产出 splitCoord[splitTotal] 每点——splitTotal 是全局的（所有噪声实例共享槽位）
# 关键：GPU shader 的 split 输入是「每点 splitTotal floats」——无论哪个 interp！
# 所以每点上传 = splitTotal(8672) floats，与内容树无关！

# 但等等——interp 角点的 split 可能只用其中一部分？看 eval_df_base 的 NOISE_SLOT_BASE 索引
# 实际：所有 interp 共享同一个 splitCoord buffer（按 sIdx 索引），每点全量 8672

# 带宽估算：
split_total = sim.g.split_total if hasattr(sim.g, 'split_total') else 8672
print(f'split_total={split_total} floats/点')

# 方案 C：GPU 算 5 interp × 1225 网格角点 = 6125 点/chunk
# 但每点仍需要完整 split（8672 floats）——因为 shader 按 sIdx 查 splitCoord[sIdx*splitTotal + ...]
# 角点数 6125 × 8672 × 4B = 212MB/chunk？ 还是说不同 interp 的角点共享 split？
# 关键：sIdx 是「采样点索引」——5 interp × 1225 角点 = 6125 个不同采样点 → 6125 个 split 行
n_pts = 5 * 1225
upload = n_pts * split_total * 4
print(f'方案C: {n_pts} 点/chunk × {split_total} floats × 4B = {upload/1e6:.1f} MB/chunk')

# 对比逐 block 方案：98304 点 × 8672 × 4B
upload2 = 98304 * split_total * 4
print(f'逐block: 98304 点 × {split_total} × 4B = {upload2/1e6:.0f} MB/chunk')

# 对比 wg_fill_density（I5 可行）：768 点/chunk
upload3 = 768 * split_total * 4
print(f'I5网格: 768 点 × {split_total} × 4B = {upload3/1e6:.1f} MB/chunk')

# 关键疑问：interp 内容树是否真的需要全量 8672 floats/点？
# 只有内容树用到的噪声实例才需要——但 shader 的 split 布局是全局的（按 sIdx 全量）
# 若 GPU 能只上传内容树用到的部分 → 大幅减量。但当前 shader 结构是全局 split buffer。
