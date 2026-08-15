# check_interp_roles.py —— 列出 5 个 interp 实例的角色（delegate_root 节点 + 内容树类型）
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import importlib.util

spec = importlib.util.spec_from_file_location('sim', r'dbg_full_sim.py')
sim = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sim)

nodes = sim.nodes
N = len(nodes)
TYPES = {}
for k, v in sim.__dict__.items():
    if k.startswith('DF_'):
        TYPES[v] = k

print(f'df_nodes={N}')
for idx, root in enumerate(sim.g.interp_roots):
    # 收集该 delegate_root 闭包的类型构成
    from collections import Counter
    reach = set()
    def visit(i):
        if i < 0 or i >= N or i in reach: return
        reach.add(i)
        n = nodes[i]
        visit(n.get('a1', -1)); visit(n.get('a2', -1)); visit(n.get('a3', -1))
    visit(root)
    cnt = Counter(TYPES.get(nodes[i]['type'], '?') for i in reach)
    print(f'interp[{idx}] root={root} nodes={len(reach)} types={dict(cnt)}')
