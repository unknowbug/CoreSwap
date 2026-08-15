# minimal11 = minimal5 + interp_2..4 完整定义 → 测 interp_2..4 定义是否触发 TDR
comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal5.comp', encoding='utf-8').read()
full = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\final_density.comp', encoding='utf-8').read()

i2 = full.index('float interp_2(int sIdx, int ix, int iy, int iz) {')
i5 = full.index('float eval_density(')
interps_2_4 = full[i2:i5].rstrip()

marker = "void main() {"
comp = comp.replace(marker, interps_2_4 + "\n" + marker, 1)
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal11.comp', 'w', encoding='utf-8').write(comp)
print("minimal11.comp 生成（minimal5 + interp_2..4 完整定义）")
