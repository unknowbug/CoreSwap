# gen_spline_diag.py —— A 方案 spline 剩余成本二分（fixed_node / coord_const）
# 量化 spline_eval 里「动态 node 索引」 vs 「coord switch 分派」 vs 「spline_eval 结构」各自成本
import json, subprocess, os, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
sdk = r'C:\VulkanSDK\1.4.357.0'
outdir = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\diag'
os.makedirs(outdir, exist_ok=True)

for variant in ('full', 'fixed_node', 'coord_const', 'fixed_node_coord_const', 'coord_slot0', 'coord_case0'):
    os.environ['DFC_DIAG'] = variant
    import importlib.util
    spec = importlib.util.spec_from_file_location('dfc_gen_v', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    g = m.DfcGen(dfdir, ndir)
    g.gen_df(fd)
    src = g.gen_shader(fd)
    comp = os.path.join(outdir, f'{variant}.comp')
    spv = os.path.join(outdir, f'{variant}.spv')
    open(comp, 'w', encoding='utf-8').write(src)
    r = subprocess.run([os.path.join(sdk, 'Bin', 'glslc.exe'), comp, '-o', spv], capture_output=True, text=True)
    print(f'{variant}: glslc rc={r.returncode} spv={"%.1fKB" % (os.path.getsize(spv)/1024) if os.path.exists(spv) else "FAIL"}')
    if r.returncode != 0:
        print('  ', r.stderr.strip().splitlines()[:3])
os.environ['DFC_DIAG'] = ''
