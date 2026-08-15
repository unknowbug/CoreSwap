# 方案1f 完整：eval_df 的 DF_INTERP 分支改 r=0（不调 interp）→ 隔离「调 interp」vs「分支存在」
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
new = """        if (t == 5) {
            r = 0.0;
            valBuf[PER_SAMPLE * sIdx + SLOT_OF_T[ci]] = r;
            continue;
        }
"""
assert old in comp
comp = comp.replace(old, new)
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\test_r0.comp', 'w', encoding='utf-8').write(comp)
print('test_r0 生成（21 循环 + r=0 分支）')
