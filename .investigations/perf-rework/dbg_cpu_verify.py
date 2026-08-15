import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]

spec = importlib.util.spec_from_file_location("m", r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect()
g.gen_df(fd)
hdr = g.gen_cpu(fd)   # 完整流程：先分配 normal_vec_index 再 _gen_split_lines
print(f"gen_cpu OK, header {len(hdr)} chars")

# 统计 split 行里的 normal 引用
import re
rows = re.findall(r'splitDouble\(normals\[(\d+)\], .*?, out, (\d+), (\d+)\)', hdr)
n_refs = {}
for ni, sb, n in rows:
    n_refs.setdefault(int(ni), []).append((int(sb), int(n)))
print(f"splitDouble 行数: {len(rows)}, 覆盖 normals 索引数: {len(n_refs)}")
# continentalness = normals[0]? 查 manifest 顺序
manifest = g.gen_noise_manifest()
for i, ni in enumerate(manifest["normal_instances"][:5]):
    print(f"  normals[{i}] noise_key={ni['noise_key']} splitBase={ni['splitBase']} n={ni['n']}")
# 检查 normals[0] 是否有 split 行
if 0 in n_refs:
    print(f"normals[0] split 行: {n_refs[0]}")
else:
    print("normals[0] 无 split 行 !!!")
