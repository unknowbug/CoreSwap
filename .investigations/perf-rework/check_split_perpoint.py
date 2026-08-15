# check_split_perpoint.py —— 验证：GPU interp 内容树能否用「每点坐标 split + 固定 corner 实例」算任意点
# 若 eval_df_base(corner=0, 点P的split) == eval_df_base(corner=c, 角点split) 在 P 处一致
# → GPU 可改「每点 split + 共享实例」→ 方案 C 复活（1225 网格角点可独立算）
# sim 的 eval_df_base 用 corner 参数决定噪声实例（NOISE_SLOT_BASE + corner*STRIDE），
# 但 split 数据是全局的（split_dump.bin 按 sIdx 全量）——corner 只换实例索引，不换 split！
# 所以 sim 里 corner=c 的值差异 = 实例的 split 行差异（实例的 splitBase 不同）
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

# 看 sim 的 eval_df_base 噪声行：NOISE_SLOT_BASE(a1) + corner * NOISE_SLOT_STRIDE(a1)
# NOISE_SLOT_BASE(slot) = slot 的 base（0,8,16...），STRIDE = 1
# normal_noise(noiseIdx=base + corner*1, sIdx) → NORMAL[base+corner] 实例
# 每个实例的 splitBase 不同（corner0 的实例 splitBase 对应 corner0 角点坐标的拆分）
# 关键问题：实例 c0 和 c1 的 split 行是否「同一坐标的不同拆分」？
# 看 NORMAL 实例的 splitBase 和坐标链（gen_noise_manifest 有坐标链）

# 直接验证：interp 内容树 corner=0 实例在「点 P」用「P 的拆分」——但 sim 没有「每点拆分」切换
# 只能验证：corner 实例的 split 行对应哪个坐标（看 manifest 或 dump）
# 换思路：GPU 完整树（fill）已经用「每点 split + 全局实例」算任意坐标 = 正确（e2e 验证）
# interp 内容树若也用「每点 split + 实例」→ 应同样正确（同一结构）
# 验证：sim 的 eval_df_base 用 sIdx 的全局 split（每点自己的拆分）+ corner 实例
# 对比：eval_df_base(corner=c) vs eval_df_base(corner=0) 在「同一 sIdx（同一 split）」——
# 若实例参数相同（仅 splitBase 不同），差异 = 实例 split 行的坐标不同
# 用两个不同 sIdx（不同点）验证实例行为
def eval_base(interp_idx, corner, sIdx, ix, iy, iz):
    root = sim.g.interp_roots[interp_idx]
    r = sim.eval_df_base(root, corner, sIdx, ix, iy, iz)
    return r if not (isinstance(r, tuple) and isinstance(r[0], str)) else None

# 同一坐标不同 sIdx（不同拆分）不同 corner —— 若「坐标→实例→值」固定，则 corner 只影响实例
print('同一坐标 (8,-56,8)，不同 sIdx（不同点拆分）不同 corner 的 interp[0] 值：')
for sIdx in (0, 1, 2):
    vals = [eval_base(0, c, sIdx, 8, -56, 8) for c in range(8)]
    print(f'  sIdx={sIdx} c0={vals[0]:.6f} c7={vals[7]:.6f} range={max(vals)-min(vals):.2e}')
