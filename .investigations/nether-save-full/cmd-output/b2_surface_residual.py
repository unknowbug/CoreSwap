# -*- coding: utf-8 -*-
# b2_surface_residual.py — SURFACE 口径：vanilla SURFACE 参照 vs 纯 Rust rlib dump
# 载体：SURFACE 参照（无 carvers/features，BlockProbe 默认口径） vs .tmp/b1-rlib-blocks.bin（纯 Rust noise+surface）
# §9.7 声明：此百分比与存档口径（93.8988%）载体不同不可比
import struct, json, collections, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
d = json.load(open(r'E:\PYTHON\CoreSwap\.tmp-coreswap-data\blocks.json')); i2n = {v: k for k, v in d.items()}
rlib = struct.unpack('<%di' % (1048576), open(r'E:\PYTHON\CoreSwap\.tmp\b1-rlib-blocks.bin', 'rb').read())
ref = open(r'E:\PYTHON\CoreSwap\.tmp-coreswap-data\vanilla_8576294172403134396_4_3200_3208_nether.blocks', 'rb').read()
magic, seed = struct.unpack('>iq', ref[:12])
size = struct.unpack('>i', ref[12:16])[0]
oX, oZ = struct.unpack('>ii', ref[16:24])
min_y, height = struct.unpack('>ii', ref[24:32])
print('[ref] magic=%08x seed=%d size=%d origin=(%d,%d) min_y=%d height=%d' % (magic & 0xffffffff, seed, size, oX, oZ, min_y, height))
assert seed == 8576294172403134396, 'SEED MISMATCH'
n = 256 * height; pos = 32
pairs = collections.Counter(); fam = collections.Counter()
match = total = 0
ex = {}
for ci in range(size * size):
    wx, wz = struct.unpack('>ii', ref[pos:pos+8]); pos += 8
    ids = list(struct.unpack('>%dh' % n, ref[pos:pos+2*n])); pos += 2*n
    for _ in range(256):
        ln = struct.unpack('>h', ref[pos:pos+2])[0]; pos += 2 + ln
    base = (((wx - 200) * 4 + (wz - 200)) * n)
    for k in range(n):
        rn = i2n.get(ids[k], '?'); rib = i2n.get(rlib[base + k], '?')
        total += 1
        if rn == rib:
            match += 1
        else:
            pairs[(rn, rib)] += 1
            a = rn.replace('minecraft:', ''); b = rib.replace('minecraft:', '')
            if a == 'air': fam['ref_air_rust_solid'] += 1
            elif b == 'air' or b == 'cave_air': fam['ref_solid_rust_air'] += 1
            else: fam['solid_solid'] += 1
            if len(ex.setdefault((rn, rib), [])) < 3:
                yy = k // 256; z = (k % 256) // 16; x = k % 16
                ex[(rn, rib)].append((wx * 16 + x, yy, wz * 16 + z))
print('SURFACE ref vs pure-Rust rlib : %d/%d = %.4f%%' % (match, total, 100.0 * match / total))
print('families:', dict(fam))
print('\ntop mismatches:')
for p, c in pairs.most_common(15):
    print('  %s -> %s x%d  ex=%s' % (p[0].replace('minecraft:', ''), p[1].replace('minecraft:', ''), c, ex.get(p)))
