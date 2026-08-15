"""C v2: 生成角点级拆分 shader（8 corner + interp + noodle + merge）+ glslc 编译。"""
import json, sys, os, subprocess
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen

DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
OUT = r'E:\PYTHON\CoreSwap\.investigations\perf-rework'

settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]

g = dfc_gen.DfcGen(DFDIR, NDIR)
g.gen(fd)
result = g.gen_split_shaders(fd)

GLSLC = r"C:\VulkanSDK\1.4.357.0\Bin\glslc.exe"
SPIRVDIS = r"C:\VulkanSDK\1.4.357.0\Bin\spirv-dis.exe"

print("=== generated shaders ===")
for name, (src, ids) in result.items():
    path = os.path.join(OUT, f"{name}.comp")
    with open(path, 'w', encoding='utf-8') as f:
        f.write(src)
    print(f"  {name}: {len(ids)} noises, {src.count(chr(10))} lines, {len(src)} bytes")

print("\n=== glslc compile + function count ===")
for name, (src, ids) in result.items():
    spv = os.path.join(OUT, f"{name}.spv")
    r = subprocess.run([GLSLC, os.path.join(OUT, f"{name}.comp"), "-o", spv], capture_output=True, text=True)
    if r.returncode != 0:
        print(f"  {name}: COMPILE FAIL")
        for line in r.stderr.splitlines()[:8]:
            print(f"    {line}")
    else:
        d = subprocess.run([SPIRVDIS, spv], capture_output=True, text=True)
        funcs = d.stdout.count("OpFunction ")
        size = os.path.getsize(spv) / 1024
        print(f"  {name}: OK, {funcs} functions, {size:.0f} KB")
