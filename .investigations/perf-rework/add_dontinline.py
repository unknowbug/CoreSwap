# add_dontinline.py —— 给 SPIR-V 所有非 entry 函数加 DontInline decoration（阻止驱动内联）
import subprocess, sys, re

sdk = r"C:\VulkanSDK\1.4.357.0"
spvdis = f"{sdk}\\Bin\\spirv-dis.exe"
spvas = f"{sdk}\\Bin\\spirv-as.exe"

def dis(inp):
    return subprocess.run([spvdis, inp], capture_output=True, text=True).stdout

def asm(dis_text, outp):
    import tempfile, os
    with tempfile.NamedTemporaryFile('w', suffix='.spvasm', delete=False, encoding='utf-8') as f:
        f.write(dis_text)
        tmp = f.name
    r = subprocess.run([spvas, tmp, '--target-env', 'vulkan1.1spv1.4', '-o', outp], capture_output=True, text=True)
    if r.returncode != 0:
        print("spirv-as 错误:", r.stderr[:500])
    os.unlink(tmp)

inp, outp = sys.argv[1], sys.argv[2]
text = dis(inp)
lines = text.split('\n')

# 收集非 entry 函数 ID（OpFunction 行，跳过 main）
funcs = []
for ln in lines:
    m = re.match(r'\s*%(\w+) = OpFunction\b', ln)
    if m and m.group(1) != 'main':
        funcs.append(m.group(1))

# 在第一个 OpEntryPoint 行之后插入 OpDecorate ... DontInline
out_lines = []
inserted = False
for ln in lines:
    out_lines.append(ln)
    if not inserted and ln.strip().startswith('OpEntryPoint'):
        for fn in funcs:
            out_lines.append(f'OpDecorate %{fn} DontInline')
        inserted = True

asm('\n'.join(out_lines), outp)
print(f"给 {len(funcs)} 个函数加了 DontInline -> {outp}")
