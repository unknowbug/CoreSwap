# gen_final_density_channels.py — X2 v3（260903-05）：每 channel 一个独立 shader（final_density_ch{k}.comp）。
# 语义：channel_k @ grid corner P = interp_k 的 d000 = eval_df_base_{k}(delegate_root, 0, sIdx, P, P, P)
#   （P 为 cell min-corner → interp trilinear fx=fy=fz=0 ≡ d000；eval_df_base_{k}(ROOT,0,...) 正是
#   interp_N 函数体对 d000 的原调用（dfc_gen.py:406 samples[0]），逐位同源）。
# 【v3 形态史（编译器墙，两次实测）】
#   v1 单 shader main 调 interp_k×5：glslc -O 全内联 → 4MB spv → vkCreateComputePipelines -13 OOM。
#   v2 单 shader main 调 eval_df×5（常量 root）：管线创建驱动编译器 CPU 自旋 >30min（RTX4060 实测）。
#   v3 每 channel 独立 spv，main 单调用 = final shader 同构（227KB 级，已验证可编译）→ 多 pipeline。
#   ⚠️ glslc -O 对本 comp 会挂起（>25min）——用无 -O 编译（实测 5s，229KB）。
# 产物：final_density_ch{0..4}.comp + channels_map.json（interp_order 供 Rust 对拍）
# 用法：先 Push-Location .investigations\perf-rework 再跑
import json, dfc_gen, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(dfdir, ndir)
src = g.gen_shader(fd)

nch = len(g.interp_funcs)
if nch == 0:
    print('[FAIL] interp_funcs empty'); sys.exit(1)

OLD_MAIN = """void main() {{
    uint idx = gl_GlobalInvocationID.x;
    if (idx >= outBuf.density.length()) return;
    int ix = coord.coords[idx * 3 + 0];
    int iy = coord.coords[idx * 3 + 1];
    int iz = coord.coords[idx * 3 + 2];
    outBuf.density[idx] = eval_density(int(idx), ix, iy, iz);
}}"""
needle = OLD_MAIN.replace('{{','{').replace('}}','}')
if needle not in src:
    print('[FAIL] main template not found — generator drift'); sys.exit(1)

order = []
for k, (interp_idx, _) in enumerate(g.interp_funcs):
    root = g.interp_root_pos[interp_idx] if interp_idx < len(g.interp_root_pos) else 0
    new_main = f"""void main() {{
    uint idx = gl_GlobalInvocationID.x;
    if (idx >= outBuf.density.length()) return;
    int ix = coord.coords[idx * 3 + 0];
    int iy = coord.coords[idx * 3 + 1];
    int iz = coord.coords[idx * 3 + 2];
    outBuf.density[idx] = eval_df_base_{interp_idx}({root}, 0, int(idx), ix, iy, iz);
}}"""
    open(f'final_density_ch{k}.comp', 'w', encoding='utf-8').write(src.replace(needle, new_main))
    order.append(interp_idx)
    print(f'[OK] final_density_ch{k}.comp (interp_{interp_idx}, root={root})')

open('channels_map.json', 'w', encoding='utf-8').write(json.dumps({"nch": nch, "interp_order": order}, indent=1))
print('[hint] for k in 0..4: glslc --target-spv=spv1.0 final_density_ch{k}.comp -o <gpu-assets>/final_density_ch{k}.spv  （勿加 -O，会挂起）')
