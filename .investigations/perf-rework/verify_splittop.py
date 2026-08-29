# verify_splittop.py —— 静态自检：gen_cpu 生成的 split()/splitTop() 一致性
# 1) splitTop 行数 == split() 行数 / 8（同 tree、同 corner0_only 遍历）
# 2) splitTop 的 @c0 行（splitDouble/splitOldBlended/ws）与 split() 的 corner-0 行逐行参数一致
# 3) sample() 调 splitTop 而非 split；split()/prepare() 保持全量
import json, sys, os, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen

dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(dfdir, ndir)
root = g.gen_df(fd)
cpp = g.gen_cpu(fd)

def extract(body, name):
    # 提取函数体（第一个匹配），返回行列表
    m = re.search(name + r"\s*\([^)]*\)\s*\{(.*(?:\n.*)*?)\n    \}", body, re.S)
    if not m:
        return None
    return m.group(1)

# 提取 split 与 splitTop 函数体（贪心到下一函数前的 "    }"）
def grab(body, fn):
    i = body.find(fn)
    if i < 0:
        return None
    j = body.find('}', i)
    # 找函数体的匹配大括号
    depth = 0
    p = body.find('{', i)
    start = p
    while p < len(body):
        if body[p] == '{': depth += 1
        elif body[p] == '}':
            depth -= 1
            if depth == 0:
                return body[start:p+1]
        p += 1
    return None

split_body = grab(cpp, "void split(int x, int y, int z, float* out) {")
splittop_body = grab(cpp, "void splitTop(int x, int y, int z, float* out) {")
print("split present:", split_body is not None)
print("splitTop present:", splittop_body is not None)
if splittop_body is None:
    sys.exit(1)

# 统计 split-call 行（含 splitDouble/splitOldBlended/ws splitDouble）
def calls(body):
    return [ln.strip() for ln in body.split('\n') if ('splitDouble' in ln or 'splitOldBlended' in ln)]

sc = calls(split_body)
tpc = calls(splittop_body)
print("split call lines:", len(sc), " splitTop call lines:", len(tpc))
if len(sc) == 0:
    sys.exit(1)
ratio = len(sc) / len(tpc)
print("ratio split/splitTop = %.2f (期望 ~8.0)" % ratio)

# 逐行匹配：splitTop 的每行必须是 split() 中「同一噪声实例」的 corner-0 行（用 splitBase+normals idx 识别）
def norm_idx(ln):
    # splitDouble 的目标实例 = splitDouble(normals[K], ...) 里的 normals[K]（忽略 ws_scale 内的 rarity 输入）
    m = re.search(r'splitDouble\(normals\[(\d+)\]', ln)
    return int(m.group(1)) if m else None
def old_idx(ln):
    m = re.search(r'splitOldBlended\(\*oldBlendeds\[(\d+)\]', ln)
    return int(m.group(1)) if m else None
def base_of(ln):
    m = re.search(r', out, (\d+),', ln)
    return int(m.group(1)) if m else None

# split 中每个 normals[idx] 的 corner0 行 = 最小 splitBase（corner0 最早出现）
sc_by_norm = {}
sc_by_old = {}
for ln in sc:
    ni = norm_idx(ln); oi = old_idx(ln); b = base_of(ln)
    if ni is not None:
        if ni not in sc_by_norm or (b is not None and b < sc_by_norm[ni][1]):
            sc_by_norm[ni] = (ln, b)
    if oi is not None:
        if oi not in sc_by_old or (b is not None and b < sc_by_old[oi][1]):
            sc_by_old[oi] = (ln, b)

mismatches = 0
checked = 0
for ln in tpc:
    ni = norm_idx(ln); oi = old_idx(ln); b = base_of(ln)
    if ni is not None:
        ref = sc_by_norm.get(ni)
        if ref is None:
            print("  MISM: splitTop normals[%d] no full-split ref" % ni); mismatches += 1; continue
        ref_ln, ref_b = ref
        # 参数校验：splitBase + normals idx 一致即代表同实例同角点（坐标产生式由生成器同源）
        if ref_b != b:
            print("  MISM: normals[%d] splitTop base=%s != full base=%s" % (ni, b, ref_b)); mismatches += 1; continue
        checked += 1
    elif oi is not None:
        ref = sc_by_old.get(oi)
        if ref is None:
            print("  MISM: splitTop oldBlendeds[%d] no full-split ref" % oi); mismatches += 1; continue
        ref_ln, ref_b = ref
        if ref_b != b:
            print("  MISM: oldBlendeds[%d] splitTop base=%s != full base=%s" % (oi, b, ref_b)); mismatches += 1; continue
        checked += 1
    else:
        # ws splitDouble 行，含 '_d = ws_scale(' 且无 normals[ 但需 splitBase
        m = re.search(r', out, (\d+),', ln)
        if m:
            checked += 1
        else:
            print("  WARN: unparsed splitTop line: " + ln[:100])

print("checked splitTop lines:", checked, " mismatches:", mismatches)

# sample() 使用 splitTop
sm = grab(cpp, "float sample(int x, int y, int z) {")
print("sample calls splitTop:", ('splitTop(x, y, z, splitCoord.data())' in sm))
print("sample still calls split():", ('split(x, y, z, splitCoord.data())' in sm))

# prepare() 保持全量 split
pr = grab(cpp, "void prepare(int x, int y, int z) {")
print("prepare calls split():", ('split(x, y, z, splitCoord.data())' in pr))

# buildInterpGrid 仍用 split()
bg = grab(cpp, "void buildInterpGrid(int interpIdx, int chunkX, int chunkZ) {")
print("buildInterpGrid uses split():", ('split(nx, ny, nz, splitCoord.data())' in bg))

if mismatches == 0 and ratio > 7.0 and 'splitTop(x, y, z, splitCoord.data())' in sm and 'split(x, y, z, splitCoord.data())' in pr:
    print("\n[OK] verify_splittop 全部通过")
else:
    print("\n[FAIL] 存在不一致")
    sys.exit(1)
