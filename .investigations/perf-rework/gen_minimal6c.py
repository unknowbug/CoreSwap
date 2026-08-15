# minimal6c = minimal6 但 interp_1..4 定义改为 return 0（不调 eval_df_base）→ 验证「多调用内联」机制
import re
comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal6.comp', encoding='utf-8').read()

# 用正则替换 interp_1..4 函数体为 return 0.0f
def replace_interp(m):
    name = m.group(1)
    return f"float {name}(int sIdx, int ix, int iy, int iz) {{ return 0.0f; }}"

comp = re.sub(r'float interp_([1-4])\(int sIdx, int ix, int iy, int iz\) \{.*?\n\}', replace_interp, comp, flags=re.S)
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal6c.comp', 'w', encoding='utf-8').write(comp)
print("minimal6c.comp 生成")
