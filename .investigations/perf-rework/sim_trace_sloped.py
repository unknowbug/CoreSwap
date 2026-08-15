# sim_trace_sloped.py —— D23：trace sloped_cheese 链（sim 的 eval_df_base 逐节点）
# 对 corner0 (784,160,-408)，打印 interp_4 delegate 求值的每个节点值，对比参照 sloped=-2.664。
import sys, os, json, struct
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

# 复制 eval_df_base 并加 trace
nodes = sim.nodes
N = len(nodes)
DF_CONSTANT, DF_Y, DF_NOISE, DF_OLD_BLENDED, DF_SPLINE, DF_INTERP = range(6)
def trace_eval_df_base(root, corner, sIdx, ix, iy, iz):
    closure = sim.g.df_nodes
    # 用 sim 的闭包结构：找到 root 的闭包（interp_4 的 delegate）
    # 简化：直接跑 sim 的 eval_df_base 但逐个节点手动——这里改为打印全树后序求值
    # interp_4 的 root 闭包 = g.interp_roots[4] 的闭包
    # 从 dbg_full_sim 的 eval_df_base 逻辑复制（读 sim 源码对应行）
    # 手动求值：按节点数组后序（子先父后）
    val = [0.0] * N
    # 找 interp_4 闭包（root 可达集）
    import re
    reach = set()
    def visit(i):
        if i < 0 or i >= N or i in reach: return
        reach.add(i)
        n = nodes[i]
        if n['type'] == sim.DF_WEIRD:
            visit(n['a1']); return
        visit(n['a1']); visit(n['a2']); visit(n['a3'])
    visit(root)
    order = sorted(reach)
    for i in order:
        n = nodes[i]
        t, a1, a2, a3 = n['type'], n['a1'], n['a2'], n['a3']
        f0, f1, f2, f3 = n['f0'], n['f1'], n['f2'], n['f3']
        if t == DF_CONSTANT: r = f0
        elif t == DF_Y: r = float(iy)
        elif t in (sim.DF_NOISE, sim.DF_SHIFTED_NOISE): r = sim.normal_noise(sim.NOISE_SLOT_BASE(a1) + corner * sim.NOISE_SLOT_STRIDE(a1), sIdx)
        elif t == DF_OLD_BLENDED: r = sim.interp_noise(sim.NOISE_SLOT_BASE(a1) + corner * sim.NOISE_SLOT_STRIDE(a1), sIdx)
        elif t == DF_SPLINE:
            sv = sim.spline_eval_py(a1, corner, sIdx, ix, iy, iz)
            r = sv[0] if not isinstance(sv, tuple) or not isinstance(sv[0], str) else 0.0
        elif t == sim.DF_Y_CLAMPED:
            tt = max(0.0, min(1.0, (float(iy) - f0) / (f1 - f0))) if f1 != f0 else 0.0
            r = f2 + tt * (f3 - f2)
        elif t == sim.DF_ABS: r = abs(val[a1])
        elif t == sim.DF_SQUARE: r = val[a1]**2
        elif t == sim.DF_CUBE: r = val[a1]**3
        elif t == sim.DF_HALF_NEG: v = val[a1]; r = v if v > 0 else v * 0.5
        elif t == sim.DF_QUARTER_NEG: v = val[a1]; r = v if v > 0 else v * 0.25
        elif t == sim.DF_SQUEEZE: v = val[a1]; c = max(-1.0, min(1.0, v)); r = c/2 - c**3/24
        elif t == sim.DF_CLAMP: r = max(f0, min(f1, val[a1]))
        elif t == sim.DF_RANGE_CHOICE:
            inp = val[a1]
            r = val[a2] if (inp >= f0 and inp < f1) else val[a3]
        elif t == sim.DF_WEIRD:
            v = val[a1]; d = sim.ws_scale_py(int(f0), v)
            r = d * abs(sim.normal_noise(sim.NOISE_SLOT_BASE(a2) + corner * sim.NOISE_SLOT_STRIDE(a2), sIdx))
        elif t == sim.DF_BLEND_DENSITY: r = val[a1]
        elif t == sim.DF_FLAT_CACHE: r = val[a1]
        elif t == sim.DF_ADD: r = val[a1] + val[a2]
        elif t == sim.DF_MUL: r = val[a1] * val[a2]
        elif t == sim.DF_MIN: r = min(val[a1], val[a2])
        elif t == sim.DF_MAX: r = max(val[a1], val[a2])
        else: r = 0.0
        val[i] = r
        # 打印所有节点（含常量/叶子），找分叉点
        names = {0:'CONST',1:'Y',2:'NOISE',3:'OLD_BLENDED',4:'SPLINE',5:'INTERP',6:'ADD',7:'MUL',8:'MIN',9:'MAX',10:'ABS',11:'SQUARE',12:'CUBE',13:'HALF_NEG',14:'QUARTER_NEG',15:'SQUEEZE',16:'CLAMP',17:'RANGE_CHOICE',18:'Y_CLAMPED',19:'SHIFTED_NOISE',20:'BLEND_DENSITY',21:'FLAT_CACHE',22:'WEIRD'}
        print(f'  node[{i}] {names.get(t,str(t))} a1={a1} a2={a2} f0={f0:.4f} f1={f1:.4f} f2={f2:.4f} f3={f3:.4f} -> {r:.6f}')
    return val[root]

print('=== trace interp_4 delegate corner0 (784,160,-408) ===')
root = sim.g.interp_roots[4]
v = trace_eval_df_base(root, 0, 0, 784, 160, -408)
print(f'corner0 delegate = {v}')
print('参照 sloped_cheese(784,160,-408) = -2.664, fd = -0.458')
