# add_dontinline2.py —— 给 SPIR-V 非 entry 函数加 FunctionControl DontInline（正确位置，非 OpDecorate）
import subprocess, sys, re, tempfile, os

sdk = r"C:\VulkanSDK\1.4.357.0"
spvdis = f"{sdk}\\Bin\\spirv-dis.exe"
spvas = f"{sdk}\\Bin\\spirv-as.exe"

def dis(inp):
    return subprocess.run([spvdis, inp], capture_output=True, text=True).stdout

def asm(dis_text, outp):
    with tempfile.NamedTemporaryFile('w', suffix='.spvasm', delete=False, encoding='utf-8') as f:
        f.write(dis_text)
        tmp = f.name
    r = subprocess.run([spvas, tmp, '--target-env', 'vulkan1.1', '-o', outp], capture_output=True, text=True)
    if r.returncode != 0:
        print("spirv-as 错误:", r.stderr[:800])
    os.unlink(tmp)

inp, outp = sys.argv[1], sys.argv[2]
text = dis(inp)
lines = text.split('\n')

# 非 entry 函数的 OpFunction 行：FunctionControl None -> DontInline（bit 1 = 0x2）
out_lines = []
changed = 0
for ln in lines:
    m = re.match(r'(\s*%\S+ = OpFunction \S+) (None) (\S+)', ln)
    if m and '%main' not in ln:
        out_lines.append(m.group(1) + ' DontInline ' + m.group(3))
        changed += 1
    else:
        out_lines.append(ln)

asm('\n'.join(out_lines), outp)
print(f"给 {changed} 个非 entry 函数加了 FunctionControl DontInline -> {outp}")
