# gen_final_density.py —— 生成 final_density GPU 资产（shader + CpuBackend）
# 产物输出：.investigations/perf-rework/（工作区诊断用）+ versions/1.20.1/cpp/worldgen/gpu-assets/（I3 构建用）
# 用法：先 Push-Location .investigations\perf-rework 再跑（F4 教训：相对路径写 CWD）
import json, dfc_gen, sys, os
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(dfdir, ndir)
# 方案1：统一到 gen_df 收集（结构共享：噪声 slot 化 + flat_cache 共享 + 角点运行时查表）。
# gen_cpu 基于 gen_df 的收集（coord_chains/normal_chain_index 等），不再用 gen() 旧路径。
root = g.gen_df(fd)
open('cpu_backend.h', 'w', encoding='utf-8').write(g.gen_cpu(fd))
open('final_density.comp', 'w', encoding='utf-8').write(g.gen_shader(fd))
print('final_density: noise_instances =', len(g.noise_instances), 'split_total =', g.split_total,
      'interp =', len(g.interp_funcs), 'df_nodes =', len(g.df_nodes), 'noise_slots =', len(g.noise_slots),
      'per_sample =', g.per_sample)
# I3：同步到 worldgen gpu-assets（构建用；spv 需 glslc 编译后复制，这里只复制 cpu_backend.h）
gpu_assets = r'E:\PYTHON\CoreSwap\versions\1.20.1\cpp\worldgen\gpu-assets'
os.makedirs(gpu_assets, exist_ok=True)
import shutil
shutil.copyfile('cpu_backend.h', os.path.join(gpu_assets, 'cpu_backend.h'))
print('[I3] cpu_backend.h synced to gpu-assets')
