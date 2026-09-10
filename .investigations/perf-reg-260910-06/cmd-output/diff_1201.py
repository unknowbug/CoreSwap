"""1.20.1 region 逐块对拍（260910-06）。

与 1.21.6 版 diff_arms.py 的**必须差异**：
  - 1.20.1 chunk NBT **无 xPos/zPos**（knowledge/discovered/build-tooling 发现 #20）⇒ 坐标只能按
    region 文件名 + 槽位推导：cx = rx*32 + (i & 31)，cz = rz*32 + (i >> 5)。
    照抄 1.21.6 版「读 xPos 取键」在 1.20.1 上会把所有 chunk 丢掉 ⇒ common=0 的全量假阴性（工具 bug 伪装成 100% 一致）。
  - 去掉 1.21.6 版硬编码的定点坐标表；改为全域普查 + top pairs + y 分布（定点用 --pts 可选）。

用法：python diff_1201.py <regionA> <regionB> [--pts x,y,z;x,y,z]
"""
import os, sys, zlib, gzip, struct
from collections import Counter

sys.stdout.reconfigure(encoding="utf-8", errors="replace")


class R:
    def __init__(self, b): self.b = b; self.i = 0
    def u1(self):
        v = self.b[self.i]; self.i += 1; return v
    def u2(self):
        v = struct.unpack_from('>H', self.b, self.i)[0]; self.i += 2; return v
    def i4(self):
        v = struct.unpack_from('>i', self.b, self.i)[0]; self.i += 4; return v
    def i8(self):
        v = struct.unpack_from('>q', self.b, self.i)[0]; self.i += 8; return v
    def f4(self):
        v = struct.unpack_from('>f', self.b, self.i)[0]; self.i += 4; return v
    def f8(self):
        v = struct.unpack_from('>d', self.b, self.i)[0]; self.i += 8; return v
    def n(self, n):
        v = self.b[self.i:self.i + n]; self.i += n; return v


def payload(r, t):
    # ⚠️ 与 260910-05 已验证版 diff_arms.py:15-36 **逐行一致**（本块首版手写时把 TAG_Byte 读成 8 字节、
    #    TAG_Byte_Array 读成 i4 长度 ⇒ 解析失步 ⇒ 全域 7542 chunk 只解析出 760 个，且不报错 —— 见 errors E4）。
    if t == 1: return r.u1()
    if t == 2: return r.u2()
    if t == 3: return r.i4()
    if t == 4: return r.i8()
    if t == 5: return r.f4()
    if t == 6: return r.f8()
    if t == 7:
        # ⚠️ 规范：TAG_Byte_Array 长度 = TAG_Int(4B)。260910-05 的「已验证」工具此处读 u1(1B)，
        #    本块 tag 普查（`.tmp/tag7_scan.py`）证明 tag 7 **真实出现**（两载体合计 9046 次）且
        #    会出现非 NBT 合法 tag 号（15/16/18/…/255）= **失步签名** ⇒ 少数 chunk 静默解析错误/被跳过。
        #    本块按规范修正（读 i4），并重跑全部对拍（见 errors E5）。
        n = r.i4(); return r.n(n)
    if t == 8:
        n = r.u2(); return r.n(n).decode('utf-8', 'replace')
    if t == 9:
        et = r.u1(); n = r.i4(); return [payload(r, et) for _ in range(n)]
    if t == 10:
        d = {}
        while True:
            tt = r.u1()
            if tt == 0: return d
            nm = r.n(r.u2()).decode('utf-8', 'replace')
            d[nm] = payload(r, tt)
    if t == 11:
        n = r.i4(); return [r.i4() for _ in range(n)]
    if t == 12:
        n = r.i4(); return [r.i8() for _ in range(n)]
    raise ValueError(f'tag {t}')


def read_nbt(data):
    r = R(data); t = r.u1(); r.n(r.u2())
    return payload(r, t)


def iter_chunks(path):
    """yield (slot_index, chunk_dict)。1.20.1 无 xPos ⇒ 槽位索引是坐标唯一来源。"""
    raw = open(path, 'rb').read()
    if len(raw) < 8192: return
    for i in range(1024):
        off = (raw[i * 4] << 16) | (raw[i * 4 + 1] << 8) | raw[i * 4 + 2]
        if off == 0: continue
        start = off * 4096
        if start + 5 > len(raw): continue
        ln = struct.unpack_from('>I', raw, start)[0]
        comp = raw[start + 4]
        body = raw[start + 5:start + 4 + ln]
        try:
            if comp == 2: body = zlib.decompress(body)
            elif comp == 1: body = gzip.decompress(body)
        except Exception:
            continue
        try:
            yield i, read_nbt(body)
        except Exception:
            continue


def load_world(root):
    """1.20.1：坐标 = region 文件名 + 槽位推导（无 xPos 键）。"""
    w = {}
    rdir = os.path.join(root, 'region')
    if not os.path.isdir(rdir): rdir = root
    for fn in os.listdir(rdir):
        if not fn.endswith('.mca'): continue
        m = fn[2:-4].split('.')
        rx, rz = int(m[0]), int(m[1])
        for slot, nbt in iter_chunks(os.path.join(rdir, fn)):
            data = nbt.get('') if isinstance(nbt, dict) and '' in nbt else nbt
            if not isinstance(data, dict): continue
            cx = rx * 32 + (slot & 31)
            cz = rz * 32 + (slot >> 5)
            w[(cx, cz)] = data
    return w


def block_at(chunk, x, y, z):
    cy = y >> 4
    for sec in chunk.get('sections', []):
        if sec.get('Y') == cy:
            bs = sec.get('block_states')
            if bs is None: return 'minecraft:air'
            pal = bs['palette']; data = bs.get('data')
            ly, lz, lx = y & 15, z & 15, x & 15
            if data is None:
                e = pal[0]; return e.get('Name', '?') if isinstance(e, dict) else str(e)
            idx = lx + lz * 16 + ly * 256
            bits = max(4, (len(pal) - 1).bit_length()); per = 64 // bits
            v = (data[idx // per] >> ((idx % per) * bits)) & ((1 << bits) - 1)
            e = pal[v]
            return e.get('Name', '?') if isinstance(e, dict) else str(e)
    return 'minecraft:air'


def pal_names(pal):
    return [e.get('Name', '?') if isinstance(e, dict) else str(e) for e in pal]


def decode(bs):
    pal = pal_names(bs['palette']); data = bs.get('data')
    if data is None: return [pal[0]] * 4096
    bits = max(4, (len(pal) - 1).bit_length()); per = 64 // bits
    out = []
    for li in range(4096):
        v = (data[li // per] >> ((li % per) * bits)) & ((1 << bits) - 1)
        out.append(pal[v] if v < len(pal) else '?')
    return out


def status_of(chunk):
    return chunk.get('Status') or chunk.get('status') or '?'


def main():
    a_path, b_path = sys.argv[1], sys.argv[2]
    van = load_world(a_path)
    cs = load_world(b_path)
    print(f"A chunks={len(van)} B chunks={len(cs)}")
    common = set(van) & set(cs)
    print(f"common={len(common)}")
    # sanity：坐标包围盒（1.20.1 推导法的自检；范围应为干净的方块/矩形）
    if common:
        xs = [k[0] for k in common]; zs = [k[1] for k in common]
        st = Counter(status_of(van[k]) for k in common)
        print(f"box x {min(xs)}..{max(xs)} z {min(zs)}..{max(zs)} | status(A) top={st.most_common(3)}")

    pts = []
    for i, arg in enumerate(sys.argv):
        if arg == '--pts' and i + 1 < len(sys.argv):
            for tok in sys.argv[i + 1].split(';'):
                if tok.strip():
                    x, y, z = [int(v) for v in tok.split(',')]
                    pts.append((x, y, z))
    if pts:
        print('===定点对拍（±2 柱采样）===')
        for (x, y, z) in pts:
            cx, cz = x >> 4, z >> 4
            if (cx, cz) not in common:
                print(f'({x},{y},{z}) chunk({cx},{cz}) NOT in both'); continue
            for yy in range(y - 2, y + 3):
                bv = block_at(van[(cx, cz)], x & 15, yy, z & 15)
                bc = block_at(cs[(cx, cz)], x & 15, yy, z & 15)
                print(f'  ({x},{yy},{z}) A={bv:<32} B={bc:<32} {"DIFF" if bv != bc else ""}')

    print('===全域差分普查===')
    pairs = Counter(); ydist = Counter(); ndiff = 0; nblock = 0; sec_same = 0; sec_diff = 0
    for key in sorted(common):
        vsecs = {s.get('Y'): s for s in van[key].get('sections', []) if s.get('Y') is not None}
        csecs = {s.get('Y'): s for s in cs[key].get('sections', []) if s.get('Y') is not None}
        for cy in set(vsecs) | set(csecs):
            vs, csn = vsecs.get(cy), csecs.get(cy)
            vb = vs.get('block_states') if vs else None
            cb = csn.get('block_states') if csn else None
            if vb is None and cb is None: continue
            vdec = ['minecraft:air'] * 4096 if vb is None else decode(vb)
            cdec = ['minecraft:air'] * 4096 if cb is None else decode(cb)
            if vdec == cdec:
                sec_same += 1; nblock += 4096; continue
            sec_diff += 1
            for i in range(4096):
                nblock += 1
                if vdec[i] != cdec[i]:
                    ndiff += 1; pairs[(vdec[i], cdec[i])] += 1
                    ydist[cy * 16] += 1
    print(f'blocks={nblock} diff={ndiff} ({100 * ndiff / max(1, nblock):.4f}%) sections same/diff={sec_same}/{sec_diff}')
    # ★ 自检断言（260910-06 复审 C5 新增）：sections/common 必须是该维度的**合法结构值**
    #   （1.20.1 overworld = 24；nether/end = 16）——若 NBT reader 失步，该值会塌掉（E5 实例：11.04 = 真实空间的 45.7%），
    #   从而在**看百分比之前**就把工具判死（旧读法与规范读法在「diff%」上只差 2 倍、不易察觉）。
    if common:
        spc = (sec_same + sec_diff) / len(common)
        flag = "OK" if abs(spc - round(spc)) < 0.005 and round(spc) in (16, 24) else "SUSPECT"
        print(f'[SELFCHECK] sections/chunk={spc:.2f} (expect 24=overworld / 16=nether,end) -> {flag}'
              f'  [失步签名：远低于期望值或非整数值]')
    print('top pairs:')
    for (a, b), c in pairs.most_common(20):
        print(f'  {a} -> {b}: {c}')
    # 注：section 的 Y 在 NBT 里是无符号 4 字节；无 below-zero retrogen 时 y 标签 4032-4080 实为 cy −4..−1（负 y 环绕）。
    print('y-section dist (cy*16；Y 无符号读，4032-4080 段 = cy −4..−1):', sorted(ydist.items()))


if __name__ == '__main__':
    main()
