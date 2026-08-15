import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

def spline_depth(node, cur=1):
    """计算 spline 树的最大嵌套深度（nested spline value 链）。"""
    if not isinstance(node, dict):
        return cur - 1
    if "coordinate" in node and "points" in node:
        mx = cur
        for p in node.get("points", []):
            v = p.get("value")
            if isinstance(v, dict) and "coordinate" in v and "points" in v:
                mx = max(mx, spline_depth(v, cur + 1))
        return mx
    return cur - 1

# 检查所有 density function json 的 spline 深度
import os
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function\overworld'
mx_all = 0
deepest = None
for fn in os.listdir(dfdir):
    if not fn.endswith('.json'): continue
    try:
        d = json.load(open(os.path.join(dfdir, fn), encoding='utf-8'))
    except Exception:
        continue
    # 递归找所有 spline 节点
    def walk(n):
        global mx_all, deepest
        if isinstance(n, dict):
            if "coordinate" in n and "points" in n:
                dd = spline_depth(n, 1)
                if dd > mx_all:
                    mx_all = dd
                    deepest = fn
            for v in n.values():
                walk(v)
        elif isinstance(n, list):
            for v in n:
                walk(v)
    walk(d)
print(f"最大 spline 嵌套深度: {mx_all} (文件 {deepest})")
print(f"spline_eval 栈容量: 24 → {'够' if mx_all*2+1 <= 24 else '不够!'}")
