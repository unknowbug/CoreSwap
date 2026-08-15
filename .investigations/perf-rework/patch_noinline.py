# 给 eval_df_base/eval_df/interp 设 FunctionControl DontInline（SPIR-V 层防内联）
import re, subprocess, os
sdk = r'C:\VulkanSDK\1.4.357.0\Bin'
src = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\final_density.spv'
asm = src + '.asm'
out = src + '.noinline.spv'

# dis
r = subprocess.run([os.path.join(sdk, 'spirv-dis.exe'), src, '-o', asm], capture_output=True)
assert r.returncode == 0, r.stderr

lines = open(asm, encoding='utf-8').read().split('\n')
# 全局 OpName 映射
name_map = {}
for ln in lines:
    m = re.match(r'OpName %(\S+) "([^"]+)"', ln.strip())
    if m:
        name_map[m.group(1)] = m.group(2)
targets = {'eval_df_base', 'eval_df', 'interp_0', 'interp_1', 'interp_2', 'interp_3', 'interp_4'}
changed = 0
for i, ln in enumerate(lines):
    s = ln.strip()
    m = re.match(r'%(\S+) = OpFunction %\S+ None %', s)
    if m and m.group(1) in name_map:
        nm = name_map[m.group(1)].split('(')[0]
        # D16 扩展：所有非 main 函数设 DontInline（normal_noise/spline_eval 等被多调用者内联）
        if nm != 'main' and nm not in ('floorDivP',):
            lines[i] = s.replace(' None ', ' DontInline ')
            changed += 1
print(f'patching {changed} functions to DontInline')
# as
src_txt = '\n'.join(lines)
open(asm, 'w', encoding='utf-8').write(src_txt)
r = subprocess.run([os.path.join(sdk, 'spirv-as.exe'), asm, '-o', out], capture_output=True)
print('spirv-as rc=', r.returncode)
if r.returncode != 0:
    print(r.stderr.decode(errors='replace')[:2000])
