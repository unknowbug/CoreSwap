# patch <src.spv> → <src>_noinline.spv（全函数 DontInline）
import re, subprocess, os, sys
sdk = r'C:\VulkanSDK\1.4.357.0\Bin'
base = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto'
src_name = sys.argv[1] if len(sys.argv) > 1 else 'minimal4.spv'
src_path = os.path.join(base, src_name)
out_path = os.path.join(base, src_name.replace('.spv', '_noinline.spv'))
asm = os.path.join(base, 'tmp.asm')
r = subprocess.run([os.path.join(sdk, 'spirv-dis.exe'), src_path, '-o', asm], capture_output=True)
lines = open(asm, encoding='utf-8').read().split('\n')
name_map = {}
for ln in lines:
    mm = re.match(r'OpName %(\S+) "([^"]+)"', ln.strip())
    if mm:
        name_map[mm.group(1)] = mm.group(2)
chg = 0
for i, ln in enumerate(lines):
    s = ln.strip()
    mm = re.match(r'%(\S+) = OpFunction %\S+ None %', s)
    if mm and mm.group(1) in name_map:
        nm = name_map[mm.group(1)].split('(')[0]
        if nm != 'main':
            lines[i] = s.replace(' None ', ' DontInline ')
            chg += 1
open(asm, 'w', encoding='utf-8').write('\n'.join(lines))
r = subprocess.run([os.path.join(sdk, 'spirv-as.exe'), asm, '-o', out_path], capture_output=True)
print(f'{src_name}: patch {chg} funcs, as rc={r.returncode}, out={os.path.basename(out_path)}')
if r.returncode != 0:
    print(r.stderr.decode(errors='replace')[:500])
