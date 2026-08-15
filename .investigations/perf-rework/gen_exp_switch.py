# gen_exp_switch.py —— 测 switch(动态值) N case 的驱动编译时间（分派本身 vs 函数调用图）
import os, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
outdir = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\exp'
os.makedirs(outdir, exist_ok=True)

def gen(name, ncase, use_funcs):
    cases = []
    if use_funcs:
        funcs = []
        for i in range(ncase):
            funcs.append(f"float f{i}(float x) {{ return {i}.0 + x * 0.5; }}")
            cases.append(f"    case {i}: r = f{i}(x); break;")
        funcs_src = "\n".join(funcs)
        switch_src = "\n".join(cases)
    else:
        funcs_src = ""
        cases = [f"    case {i}: r = {i}.0 + x * 0.5; break;" for i in range(ncase)]
        switch_src = "\n".join(cases)
    src = f"""#version 460
layout(local_size_x = 64) in;
layout(std430, binding = 0) buffer Out {{ float outVal[]; }};
{funcs_src}
void main() {{
    uint gid = gl_GlobalInvocationID.x;
    float x = float(gid) * 0.1;
    float r = 0.0;
    switch (int(gid % {ncase})) {{
{switch_src}
    }}
    outVal[gid] = r + x;
}}
"""
    open(os.path.join(outdir, name + '.comp'), 'w', encoding='utf-8').write(src)

for ncase in (56, 10):
    gen(f'expsw_c{ncase}', ncase, False)
    gen(f'expsw_f{ncase}', ncase, True)
print('done')
