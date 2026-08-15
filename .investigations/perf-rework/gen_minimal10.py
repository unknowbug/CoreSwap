# minimal10 = minimal5 + interp_1 完整定义（提取自完整 comp）→ 测 2 个 interp 是否触发 TDR
comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal5.comp', encoding='utf-8').read()
full = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\final_density.comp', encoding='utf-8').read()

# 提取 interp_1 完整定义
i1 = full.index('float interp_1(int sIdx, int ix, int iy, int iz) {')
i2 = full.index('float interp_2(int sIdx, int ix, int iy, int iz) {')
interp_1_full = full[i1:i2].rstrip()

# 插入到 main 前
marker = "void main() {"
comp = comp.replace(marker, interp_1_full + "\n" + marker, 1)
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal10.comp', 'w', encoding='utf-8').write(comp)
print("minimal10.comp 生成（minimal5 + interp_1 完整定义）")
