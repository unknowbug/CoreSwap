# minimal6 = 完整 final_density.comp 但只留 interp_0（删 interp_1..4 + 5 路分支）→ 定位 interp_1..4
import re
comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\final_density.comp', encoding='utf-8').read()

# 1) eval_df/eval_df_base 的 DF_INTERP 5 路分支 → 只留 interp_0
branch_old = """        if (t == 5) {
            if (DF_A1[i] == 0) r = interp_0(sIdx, ix, iy, iz);
            else if (DF_A1[i] == 1) r = interp_1(sIdx, ix, iy, iz);
            else if (DF_A1[i] == 2) r = interp_2(sIdx, ix, iy, iz);
            else if (DF_A1[i] == 3) r = interp_3(sIdx, ix, iy, iz);
            else r = interp_4(sIdx, ix, iy, iz);
            valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[i]] = r;
            continue;
        }"""
branch_new = """        if (t == 5) {
            r = interp_0(sIdx, ix, iy, iz);
            valBuf[(sIdx * 9 + 8) * VAL_SLOTS + SLOT_OF[i]] = r;
            continue;
        }"""
assert branch_old in comp, "branch not found"
comp = comp.replace(branch_old, branch_new)

# 不删 interp_1..4 定义（只改分支）——保留完整文件

open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal6.comp', 'w', encoding='utf-8').write(comp)
print("minimal6.comp 生成")
