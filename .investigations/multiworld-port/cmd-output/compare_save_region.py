# -*- coding: utf-8 -*-
# compare_save_region.py — M16 Full 化：Rust 存档 MCA 区域 vs vanilla FULL 参照（size×size chunks 逐位）
# 用法: python compare_save_region.py <vanilla.blocks> <blocks.json> <mca> <size> <minY> <height>
import sys, zlib, struct, json, collections

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
_src = open(r"E:\PYTHON\CoreSwap\.investigations\multiworld-port\parse_mca_chunk.py", encoding="utf-8").read()
_src = _src.replace("\nmain()\n", "\n")
_ns = {"__name__": "parse_mca_chunk"}
exec(compile(_src, "parse_mca_chunk.py", "exec"), _ns)
read_mca_chunk = _ns["read_mca_chunk"]; parse_nbt_named = _ns["parse_nbt_named"]; unpack_states = _ns["unpack_states"]

ref_path, blocks_path, mca_path, size, min_y, height = (
    sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5]), int(sys.argv[6]))

id2name = {}
for name, rid in json.load(open(blocks_path, encoding="utf-8")).items():
    id2name[rid] = name

ref = open(ref_path, "rb").read()
magic, seed = struct.unpack(">iq", ref[:12])
fsize, oX, oZ, refMinY, refHeight = struct.unpack(">iiiii", ref[12:32])
assert fsize == size, "size mismatch ref=%d arg=%d" % (fsize, size)
pos = 32
n = refHeight * 256

grand_t = grand_ok = 0
report = []
for ci in range(size * size):
    wx, wz = struct.unpack(">ii", ref[pos:pos+8]); pos += 8
    ref_ids = list(struct.unpack(">%dh" % n, ref[pos:pos+2*n])); pos += 2*n
    for _ in range(256):
        ln = struct.unpack(">h", ref[pos:pos+2])[0]; pos += 2 + ln
    cx, cz = wx, wz
    raw = read_mca_chunk(mca_path, cx, cz)
    total = ok = 0
    mism = collections.Counter()
    if raw is None:
        print("[MISS] chunk(%d,%d) not in mca" % (cx, cz))
        grand_t += n
        continue
    _, root = parse_nbt_named(raw)
    save = [None] * n
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
    for i in range(n):
        rname = id2name.get(ref_ids[i], "raw:%d" % ref_ids[i])
        sname = save[i] if save[i] is not None else "minecraft:air"
        total += 1
        if rname == sname: ok += 1
        else: mism[(rname, sname)] += 1
    grand_t += total; grand_ok += ok
    report.append((cx, cz, total, ok, mism))
    print("[chunk(%d,%d)] %d/%d = %.4f%%" % (cx, cz, ok, total, 100.0 * ok / total))

print("=== GRAND TOTAL %d/%d = %.4f%% (ref seed=%d, origin=(%d,%d), size=%d) ===" %
      (grand_ok, grand_t, 100.0 * grand_ok / grand_t if grand_t else 0, seed, oX, oZ, size))
agg = collections.Counter()
for _, _, _, _, m in report: agg.update(m)
print("top mismatches (vanilla -> save):")
for (a, b), c in agg.most_common(15):
    print("  %s -> %s x%d" % (a, b, c))
