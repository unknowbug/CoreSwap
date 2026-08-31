# -*- coding: utf-8 -*-
# compare_save_vs_ref.py — M14 修复验证：Rust 写入的存档 chunk vs vanilla BlockProbe 参照（raw id 域逐位）
# 用法: python compare_save_vs_ref.py <vanilla.blocks> <blocks.json> <mca> <cx> <cz> <minY> <height>
import sys, zlib, struct, json, collections

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
_src = open(r"E:\PYTHON\CoreSwap\.investigations\multiworld-port\parse_mca_chunk.py", encoding="utf-8").read()
_src = _src.replace("\nmain()\n", "\n")
_ns = {"__name__": "parse_mca_chunk"}
exec(compile(_src, "parse_mca_chunk.py", "exec"), _ns)
read_mca_chunk = _ns["read_mca_chunk"]; parse_nbt_named = _ns["parse_nbt_named"]; unpack_states = _ns["unpack_states"]

ref_path, blocks_path, mca_path, cx, cz, min_y, height = (
    sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5]), int(sys.argv[6]), int(sys.argv[7]))

id2name = {}
for name, rid in json.load(open(blocks_path, encoding="utf-8")).items():
    id2name[rid] = name

ref = open(ref_path, "rb").read()
magic, seed = struct.unpack(">iq", ref[:12])
size, oX, oZ, refMinY, refHeight = struct.unpack(">iiiii", ref[12:32])
pos = 32
cx, cz = oX // 16, oZ // 16  # origin 为块坐标
n = refHeight * 256
ref_ids = list(struct.unpack(">%dh" % n, ref[pos:pos+2*n])); pos += 2*n

# save side
raw = read_mca_chunk(mca_path, cx, cz)
_, root = parse_nbt_named(raw)
save = [None] * n  # index (y-min_y)*256 + z*16 + x
for s in root.get("sections", []):
    y0 = s.get("Y", 0) * 16
    bs = s.get("block_states", {}) or {}
    pal = bs.get("palette", [])
    names = [p.get("Name", "?") if isinstance(p, dict) else str(p) for p in pal]
    data = bs.get("data")
    if data is None or not pal:
        nm = names[0] if names else "minecraft:air"
        for i in range(4096):
            yy = y0 + (i >> 8); z = (i >> 4) & 15; x = i & 15
            if min_y <= yy < min_y + height:
                save[(yy - min_y) * 256 + z * 16 + x] = nm
        continue
    bits = max(4, (len(pal) - 1).bit_length())
    L = data if isinstance(data, list) else list(struct.unpack(">%dq" % len(data), bytes(data)))
    idxs = unpack_states(L, bits, 4096)
    for i, ix in enumerate(idxs):
        yy = y0 + (i >> 8); z = (i >> 4) & 15; x = i & 15
        if min_y <= yy < min_y + height:
            save[(yy - min_y) * 256 + z * 16 + x] = names[ix]

total = 0; ok = 0; mism = collections.Counter()
for i in range(n):
    r = ref_ids[i]; sname = save[i]
    rname = id2name.get(r, "raw:%d" % r)
    if sname is None: sname = "minecraft:air"
    total += 1
    if rname == sname: ok += 1
    else: mism[(rname, sname)] += 1

print("[OK] %d/%d = %.4f%%" % (ok, total, 100.0 * ok / total))
print("top mismatches (vanilla -> save):")
for (a, b), c in mism.most_common(12):
    print("  %s -> %s x%d" % (a, b, c))
