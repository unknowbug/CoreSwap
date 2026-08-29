# dbg_gen_shader_check.py —— 确认 gen_shader(GLSL) 仍可生成（未被 gen_cpu_sampling 修改影响）。
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen

dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
s = g.gen_shader(fd)
c = g.gen_cpu(fd)   # calls gen_cpu_sampling (modified)
print("gen_shader bytes:", len(s))
print("gen_cpu bytes:", len(c))
print("shader has closure arrays (CTYPE_0):", "CTYPE_0[" in s or "CLOSURE_0_LEN" in s)
print("shader has eval_df_base_0:", "eval_df_base_0" in s)
print("shader has DF_NODES const:", "const int DF_NODES" in s)
print("OK: both generators run")
