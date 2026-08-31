# -*- coding: utf-8 -*-
# parse_mca_chunk.py — 解析 DIM-1 region 的指定 chunk，统计各 section 的 palette 块构成。
# 用法：python parse_mca_chunk.py <mca路径> <chunk_x> <chunk_z>
import sys, zlib, struct

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

def read_mca_chunk(path, cx, cz):
    idx = ((cx & 31) + (cz & 31) * 32) * 4
    with open(path, "rb") as f:
        data = f.read()
    if len(data) < idx + 5:
        return None
    off = int.from_bytes(data[idx:idx+3], "big")
    cnt = data[idx+3]
    if off == 0 or cnt == 0:
        return None
    start = off * 4096
    length = int.from_bytes(data[start:start+4], "big")
    comp = data[start+4]
    raw = data[start+5:start+4+length]
    if comp == 1:
        import gzip
        raw = gzip.decompress(raw)
    else:
        raw = zlib.decompress(raw)
    return raw

def parse_nbt_named(data):
    pos = [0]
    def tag_type():
        t = data[pos[0]]; pos[0] += 1; return t
    def read_name():
        ln = struct.unpack(">H", data[pos[0]:pos[0]+2])[0]; pos[0] += 2
        s = data[pos[0]:pos[0]+ln].decode("utf-8", "replace"); pos[0] += ln
        return s
    def payload(t):
        if t == 1:
            v = struct.unpack(">b", data[pos[0]:pos[0]+1])[0]; pos[0] += 1; return v
        if t == 2:
            v = struct.unpack(">h", data[pos[0]:pos[0]+2])[0]; pos[0] += 2; return v
        if t == 3:
            v = struct.unpack(">i", data[pos[0]:pos[0]+4])[0]; pos[0] += 4; return v
        if t == 4:
            v = struct.unpack(">q", data[pos[0]:pos[0]+8])[0]; pos[0] += 8; return v
        if t == 5:
            v = struct.unpack(">f", data[pos[0]:pos[0]+4])[0]; pos[0] += 4; return v
        if t == 6:
            v = struct.unpack(">d", data[pos[0]:pos[0]+8])[0]; pos[0] += 8; return v
        if t == 7:
            ln = struct.unpack(">i", data[pos[0]:pos[0]+4])[0]; pos[0] += 4
            v = data[pos[0]:pos[0]+ln]; pos[0] += ln; return bytearray(v)
        if t == 8:
            return read_name()
        if t == 9:
            it = tag_type(); ln = struct.unpack(">i", data[pos[0]:pos[0]+4])[0]; pos[0] += 4
            return [payload(it) for _ in range(ln)]
        if t == 10:
            d = {}
            while True:
                ct = tag_type()
                if ct == 0:
                    return d
                nm = read_name()
                d[nm] = payload(ct)
        if t == 11:
            ln = struct.unpack(">i", data[pos[0]:pos[0]+4])[0]; pos[0] += 4
            v = list(struct.unpack(">%di" % ln, data[pos[0]:pos[0]+4*ln])); pos[0] += 4*ln; return v
        if t == 12:
            ln = struct.unpack(">i", data[pos[0]:pos[0]+4])[0]; pos[0] += 4
            v = list(struct.unpack(">%dq" % ln, data[pos[0]:pos[0]+8*ln])); pos[0] += 8*ln; return v
        raise ValueError("tag %d @ %d" % (t, pos[0]))
    t = tag_type()
    root_name = read_name()
    return root_name, payload(t)

def unpack_states(longs, bits, count):
    per = 64 // bits
    out = []
    for L in longs:
        u = L & 0xFFFFFFFFFFFFFFFF
        for k in range(per):
            if len(out) >= count:
                return out
            out.append((u >> (k * bits)) & ((1 << bits) - 1))
    while len(out) < count:
        out.append(0)
    return out

def main():
    path, cx, cz = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    raw = read_mca_chunk(path, cx, cz)
    if raw is None:
        print("chunk (%d,%d) 在 region 中不存在" % (cx, cz))
        return
    root_name, root = parse_nbt_named(raw)
    secs = root.get("sections", [])
    print("root keys:", [k for k in root.keys()][:12])
    print("Status:", root.get("Status", "?"), " yPos:", root.get("yPos", "?"))
    print("sections:", len(secs))
    if secs:
        print("sec keys:", list(secs[0].keys()))
    for s in secs:
        y = s.get("Y", "?")
        bs = s.get("block_states", {}) or {}
        pal = bs.get("palette", [])
        names = [p.get("Name", "?") if isinstance(p, dict) else str(p) for p in pal]
        data = bs.get("data")
        if data is None or not pal:
            first = names[0] if names else "empty"
            print("Y=%-4s %s x4096" % (y, first.split(":")[-1]))
            continue
        bits = max(4, (len(pal) - 1).bit_length())
        L = data if isinstance(data, list) else list(struct.unpack(">%dq" % len(data), bytes(data)))
        idxs = unpack_states(L, bits, 4096)
        counts = {}
        for ix in idxs:
            nm = names[ix] if ix < len(names) else "?"
            counts[nm] = counts.get(nm, 0) + 1
        top = sorted(counts.items(), key=lambda kv: -kv[1])[:4]
        print("Y=%-4s %s" % (y, " | ".join("%s x%d" % (n.split(":")[-1], c) for n, c in top)))

main()

