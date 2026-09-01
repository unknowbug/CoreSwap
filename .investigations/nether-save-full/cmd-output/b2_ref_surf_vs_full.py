# -*- coding: utf-8 -*-
# b2_ref_surf_vs_full.py — SURFACE 参照 vs FULL 参照差异（验证口径生效 + features 贡献量化）
import struct, json, collections, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
d = json.load(open(r'E:\PYTHON\CoreSwap\.tmp-coreswap-data\blocks.json')); i2n = {v: k for k, v in d.items()}

def load(p):
    b = open(p, 'rb').read()
    size = struct.unpack('>i', b[12:16])[0]
    height = struct.unpack('>ii', b[24:32])[1]
    n = 256 * height; pos = 32; out = []
    for _ in range(size * size):
        wx, wz = struct.unpack('>ii', b[pos:pos+8]); pos += 8
        out.append((wx, wz, list(struct.unpack('>%dh' % n, b[pos:pos+2*n])))); pos += 2*n
        for _ in range(256):
            ln = struct.unpack('>h', b[pos:pos+2])[0]; pos += 2 + ln
    return out

surf = load(r'E:\PYTHON\CoreSwap\.tmp-coreswap-data\vanilla_8576294172403134396_4_3200_3208_nether.blocks')
full = load(r'E:\PYTHON\CoreSwap\.tmp-coreswap-data\vanilla_8576294172403134396_4_3200_3208_nether.blocks.full')
cnt_s = collections.Counter(); cnt_f = collections.Counter(); pairs = collections.Counter(); diff = 0
for (wx, wz, si), (_, _, fi) in zip(surf, full):
    for a, f in zip(si, fi):
        na = i2n.get(a, '?'); nf = i2n.get(f, '?')
        cnt_s[na] += 1; cnt_f[nf] += 1
        if na != nf:
            diff += 1; pairs[(nf, na)] += 1
print('SURFACE vs FULL ref: diff=%d / %d (%.4f%% identical)' % (diff, len(surf)*256*256, 100.0*(1-diff/(len(surf)*256*256))))
print('\ntop FULL->SURFACE differences (feature/carver contribution):')
for p, c in pairs.most_common(10):
    print('  %s -> %s x%d' % (p[0].replace('minecraft:',''), p[1].replace('minecraft:',''), c))
print('\nSURFACE ref totals (top):')
for k, v in cnt_s.most_common(8):
    print('  %s %d (FULL %d)' % (k, v, cnt_f[k]))
