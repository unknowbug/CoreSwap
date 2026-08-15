# minimal7 = minimal5 + interp_1..4 空定义（return 0）→ 隔离「定义存在」vs「调 eval_df_base」
comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal5.comp', encoding='utf-8').read()

# 在 main 前插入 interp_1..4 空定义
stub = """
float interp_1(int sIdx, int ix, int iy, int iz) { return 0.0f; }
float interp_2(int sIdx, int ix, int iy, int iz) { return 0.0f; }
float interp_3(int sIdx, int ix, int iy, int iz) { return 0.0f; }
float interp_4(int sIdx, int ix, int iy, int iz) { return 0.0f; }
"""
marker = "void main() {"
comp = comp.replace(marker, stub + "\n" + marker, 1)
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal7.comp', 'w', encoding='utf-8').write(comp)
print("minimal7.comp 生成")
