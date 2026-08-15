# gen_exp_shaders.py —— 生成最小实验 shader（判别驱动编译瓶颈 H1 循环长度 / H2 分支数 / H3 const数组 vs SSBO）
import os, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
outdir = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\exp'

def make_table(n, seed=0):
    # 模拟 NORMAL_PACK 访问模式：TBL[CA1[ci]*3+k]，表长 n 个 int
    return ', '.join(str((i * 7 + seed) % 100) for i in range(n))

def make_ctype(n, nbranch):
    return ', '.join(str(i % nbranch) for i in range(n))

def make_ca1(n):
    return ', '.join(str(i % 50) for i in range(n))

def gen(loop_len, nbranch, use_ssbo):
    name = f"exp_l{loop_len}_b{nbranch}_{'ssbo' if use_ssbo else 'const'}"
    tbl_decl = ""
    tbl_use = ""
    binding_src = ""
    if use_ssbo:
        tbl_decl = "layout(std430, binding = 1) buffer Tbl { int v[]; };\n"
        tbl_use = "v"
        binding_src = "    int tblInit[600] = int[](" + make_table(600) + ");\n"
    else:
        tbl_decl = "const int TBL[600] = int[](" + make_table(600) + ");\n"
        tbl_use = "TBL"
    ctype = make_ctype(loop_len, nbranch)
    ca1 = make_ca1(loop_len)
    branches = []
    for b in range(nbranch):
        if b == nbranch - 1:
            # 最后一支：动态索引大数组（模拟 NORMAL_PACK 访问）
            branches.append(f"        else if (t == {b}) r = float({tbl_use}[CA1[ci] * 3 + 1]);")
        else:
            branches.append(f"        else if (t == {b}) r = float({b}) * 0.5;")
    branch_src = "\n".join(branches)
    src = f"""#version 460
layout(local_size_x = 64) in;
layout(std430, binding = 0) buffer Out {{ float outVal[]; }};
{tbl_decl}const int CTYPE[{loop_len}] = int[]({ctype});
const int CA1[{loop_len}] = int[]({ca1});
void main() {{
    uint gid = gl_GlobalInvocationID.x;
    float acc = 0.0;
    for (int ci = 0; ci < {loop_len}; ci++) {{
        int t = CTYPE[ci];
        float r = 0.0;
        if (t == 0) r = 0.5;
{branch_src}
        acc += r;
    }}
    outVal[gid] = acc;
}}
"""
    return name, src

os.makedirs(outdir, exist_ok=True)
variants = [
    (134, 20, False),   # 基准：近似 CLOSURE_0
    (30, 20, False),    # H1 循环长度
    (10, 20, False),    # H1 循环长度（更小）
    (30, 5, False),     # H2 分支数
    (30, 20, True),     # H3 SSBO
    (134, 20, True),    # H3 SSBO（大循环）
    (134, 5, False),    # H1+H2 交叉
    (10, 5, False),     # 最小对照
]
manifest = []
for loop_len, nbranch, use_ssbo in variants:
    name, src = gen(loop_len, nbranch, use_ssbo)
    with open(os.path.join(outdir, name + '.comp'), 'w', encoding='utf-8') as f:
        f.write(src)
    manifest.append(name)
print('generated', len(variants), 'shaders:')
for m in manifest:
    print(' ', m)
