# minimal6b = 完整版但删 interp_1..4 定义 + DF_INTERP 只留 interp_0
comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\final_density.comp', encoding='utf-8').read()
lines = comp.split('\n')

# 1) DF_INTERP 分支 5 路 → 1 路（在 eval_df 定义内）
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
assert branch_old in comp
comp = comp.replace(branch_old, branch_new)

# 2) 删 interp_1..4 定义（行 648-720，从 'float interp_1(' 到 'float eval_density(' 前）
comp = comp.replace('float interp_1(int sIdx, int ix, int iy, int iz) {', '@@@INTERP1@@@', 1)
parts = comp.split('@@@INTERP1@@@')
assert len(parts) == 2
comp = parts[0] + parts[1].split('float eval_density(')[0] + 'float eval_density(' + parts[1].split('float eval_density(')[1]

# 3) 删 interp_1..4 前向声明（行 153-156）
for i in range(1, 5):
    comp = comp.replace(f'float interp_{i}(int sIdx, int ix, int iy, int iz);\n', '', 1)

open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal6b.comp', 'w', encoding='utf-8').write(comp)
print("minimal6b.comp 生成")
