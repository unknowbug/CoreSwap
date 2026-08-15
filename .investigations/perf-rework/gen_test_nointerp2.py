# 方案1f 完整 shader：删 eval_df 的 DF_INTERP 分支（t==5 无匹配走 else → r=0）→ 测分支是否存在导致 TDR
comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\final_density.comp', encoding='utf-8').read()
old = """        if (t == 5) {
            if (CA1_T[ci] == 0) r = interp_0(sIdx, ix, iy, iz);
            else if (CA1_T[ci] == 1) r = interp_1(sIdx, ix, iy, iz);
            else if (CA1_T[ci] == 2) r = interp_2(sIdx, ix, iy, iz);
            else if (CA1_T[ci] == 3) r = interp_3(sIdx, ix, iy, iz);
            else r = interp_4(sIdx, ix, iy, iz);
            valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[ci]] = r;
            continue;
        }
"""
new = ""
assert old in comp, 'DF_INTERP branch not found'
comp = comp.replace(old, new)
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\test_nointerp2.comp', 'w', encoding='utf-8').write(comp)
print('test_nointerp2 生成（eval_df 无 DF_INTERP 分支）')
