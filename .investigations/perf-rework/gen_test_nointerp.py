# 方案1e 完整 shader：eval_df 的 DF_INTERP 分支改 r=0（不调 interp）→ 测 eval_df 循环本身是否 TDR
comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\final_density.comp', encoding='utf-8').read()
# 只改 eval_df 内的分支（eval_df_base_N 无 DF_INTERP）
old = """        if (t == 5) {
            if (DF_A1[i] == 0) r = interp_0(sIdx, ix, iy, iz);
            else if (DF_A1[i] == 1) r = interp_1(sIdx, ix, iy, iz);
            else if (DF_A1[i] == 2) r = interp_2(sIdx, ix, iy, iz);
            else if (DF_A1[i] == 3) r = interp_3(sIdx, ix, iy, iz);
            else r = interp_4(sIdx, ix, iy, iz);
            valBuf[PER_SAMPLE * sIdx + SLOT_OF[i]] = r;
            continue;
        }"""
new = """        if (t == 5) {
            r = 0.0;
            valBuf[PER_SAMPLE * sIdx + SLOT_OF[i]] = r;
            continue;
        }"""
assert old in comp, 'branch not found'
comp = comp.replace(old, new)
open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\test_nointerp.comp', 'w', encoding='utf-8').write(comp)
print('test_nointerp.comp 生成')
