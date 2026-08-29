# check_features.py — 对比 FEATURES 放置位置与 vanilla FULL 参照
# 检查 Rust 放置 ore 的位置，vanilla 对应位置是否也是 ore（gravel/gold/tuff 等）。
import struct

def be32(b, i):
    v, i2 = struct.unpack('>i', b[i:i+4])[0], i+4
    return v, i2
def be16(b, i):
    v, i2 = struct.unpack('>H', b[i:i+2])[0], i+2
    return v, i2
def be64(b, i):
    v, i2 = struct.unpack('>q', b[i:i+8])[0], i+8
    return v, i2

path = r"E:\python\MC\data\vanilla_-8248318472910187742_4_-288_-256_FULL.bak.blocks"
bd = open(path, 'rb').read()
i = 0
magic, i = be32(bd, i); seed, i = be64(bd, i)
size, i = be32(bd, i); ox, i = be32(bd, i); oz, i = be32(bd, i)
miny, i = be32(bd, i); height, i = be32(bd, i)

chunks = {}
for c in range(size*size):
    cx, i = be32(bd, i); cz, i = be32(bd, i)
    blocks = []
    for k in range(16*16*height):
        v, i = be16(bd, i); blocks.append(v)
    chunks[(cx, cz)] = blocks
    for b in range(256):
        bl, i = be16(bd, i)
        if bl > 0: i += bl

def block_at(cx, cz, wx, wy, wz):
    if (cx, cz) not in chunks: return -1
    lx = wx & 15; lz = wz & 15
    ly = wy - miny
    if ly < 0 or ly >= height: return -1
    return chunks[(cx, cz)][ly*256 + lz*16 + lx]

# ore feature → 期望的 block id
ore_ids = {
    'ore_gravel': {37},        # gravel
    'ore_gold_buried': {39, 40},  # gold_ore / deepslate_gold_ore
    'ore_tuff': {909},         # tuff
    'ore_redstone': {242, 243},   # redstone_ore / deepslate_redstone_ore
}

placements = []
for line in open(r"E:\PYTHON\CoreSwap\WorldgenRust\feature_placements.txt"):
    line = line.strip()
    if not line: continue
    parts = line.split()
    if len(parts) != 4: continue
    placements.append((parts[0], int(parts[1]), int(parts[2]), int(parts[3])))

# 统计 ore 放置是否匹配 vanilla（vanilla 对应位置也是同种 ore）
match = 0; total = 0
for fid, x, y, z in placements:
    if fid == 'minecraft:freeze_top_layer': continue  # 温度 >=0 不冻结，忽略
    if fid == 'minecraft:underwater_magma': continue   # magma 特判
    cx = x >> 4; cz = z >> 4
    v = block_at(cx, cz, x, y, z)
    short = fid.split(':')[-1]
    expected = ore_ids.get(short, set())
    total += 1
    is_match = v in expected
    if is_match: match += 1
    print(f"  {short:20s} ({x},{y},{z})  vanilla id={v}  {'MATCH' if is_match else 'NO-MATCH'}")

print(f"\n=== ore 放置匹配统计: {match}/{total} ===")
