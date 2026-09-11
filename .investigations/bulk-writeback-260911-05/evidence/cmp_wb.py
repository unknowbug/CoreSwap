# -*- coding: utf-8 -*-
"""C 线 Tier 1 比对（260911-05）：两臂 [WG-CONTENT-WB] 逐 chunk hash 全等判定 + [WG-BULKWB] 自证/汇总提取。"""
import re, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

BASE = r"E:\PYTHON\CoreSwap\.tmp\vivo-stutter-260911-02\ab-threads\logs"
BASE2 = r"E:\PYTHON\CoreSwap\.tmp\a2-260911-05\logs"
if len(sys.argv) >= 5:
    # 用法: cmp_wb.py <tagA> <logA> <tagB> <logB>
    ARMS = [(sys.argv[1], sys.argv[2]), (sys.argv[3], sys.argv[4])]
else:
    ARMS = [("c-old", os.path.join(BASE, "c-old.log")), ("c-bulk", os.path.join(BASE, "c-bulk.log"))]

RE_WB = re.compile(r"\[WG-CONTENT-WB\] chunk\((-?\d+),(-?\d+)\) hash=([0-9a-f]+) dh=([0-9a-f]+) nz=(\d+)")
RE_BULK = re.compile(r"\[WG-BULKWB\](.*)")
RE_PERBLOCK = re.compile(r"\[WG-PERBLOCK\](.*)")
RE_DLL = re.compile(r"\[CppBridge\] dll=.*?sha256=([0-9a-f]+)")
RE_CHUNKS = re.compile(r"Processed: (\d+) chunks")


def load(path):
    wb, bulk, perblock, dll, chunks = {}, [], [], None, None
    if not os.path.exists(path):
        return None
    for line in open(path, encoding="utf-8", errors="replace"):
        m = RE_WB.search(line)
        if m:
            wb[(int(m.group(1)), int(m.group(2)))] = (m.group(3), m.group(4), int(m.group(5)))
        m = RE_BULK.search(line)
        if m:
            bulk.append(m.group(1).strip())
        m = RE_PERBLOCK.search(line)
        if m:
            perblock.append(m.group(1).strip())
        if dll is None:
            m = RE_DLL.search(line)
            if m:
                dll = m.group(1)
        m = RE_CHUNKS.search(line)
        if m:
            chunks = int(m.group(1))
    return {"wb": wb, "bulk": bulk, "perblock": perblock, "dll": dll, "chunks": chunks}


data = {}
for tag, path in ARMS:
    d = load(path)
    if d is None:
        print(f"[FAIL] 日志缺失: {path}")
        sys.exit(2)
    data[tag] = d
    print(f"[{tag}] dll={d['dll'][:16] if d['dll'] else '-'}  chunks(Chunky)={d['chunks']}  "
          f"[WG-CONTENT-WB] 行数={len(d['wb'])}  [WG-BULKWB] 行数={len(d['bulk'])}")

TA, TB = ARMS[0][0], ARMS[1][0]
a, b = data[TA], data[TB]
ka, kb = set(a["wb"]), set(b["wb"])
common = ka & kb
print(f"\nchunk 集: old={len(ka)} bulk={len(kb)} common={len(common)} only_old={len(ka-kb)} only_bulk={len(kb-ka)}")

mismatch, nz_mismatch, dh_mismatch = [], [], []
for k in sorted(common):
    ha, dha, na = a["wb"][k]
    hb, dhb, nb = b["wb"][k]
    if ha != hb:
        mismatch.append((k, ha, hb, na, nb))
    if dha != dhb:
        dh_mismatch.append((k, dha, dhb))
    if na != nb:
        nz_mismatch.append((k, na, nb))

print(f"\n=== Tier 1 判据：逐 chunk 方块状态 hash 全等（零差） ===")
print(f"hash 不等 chunk 数 = {len(mismatch)}   nz 不等 chunk 数 = {len(nz_mismatch)}")
for k, ha, hb, na, nb in mismatch[:10]:
    print(f"  [DIFF] chunk{k} old={ha} bulk={hb} nz {na}->{nb}")
for k, na, nb in nz_mismatch[:10]:
    print(f"  [NZ]   chunk{k} nz {na} -> {nb}")

print(f"\n=== Tier 2 判据：派生字段（三计数）指纹全等（零差） ===")
print(f"dh 不等 chunk 数 = {len(dh_mismatch)}")
for k, dha, dhb in dh_mismatch[:10]:
    print(f"  [DH-DIFF] chunk{k} old={dha} bulk={dhb}")

print(f"\n=== [WG-BULKWB] 自证/汇总 ===")
for tag in (TA, TB):
    for line in data[tag]["bulk"]:
        print(f"  [{tag}] {line}")
    for line in data[tag]["perblock"]:
        print(f"  [{tag}] [WG-PERBLOCK] {line}")

t1 = (len(mismatch) == 0 and len(nz_mismatch) == 0 and len(common) > 0
      and len(ka - kb) == 0 and len(kb - ka) == 0)
t2 = len(dh_mismatch) == 0 and len(common) > 0
verdict = "PASS" if (t1 and t2) else "FAIL"
print(f"\n>>> Tier 1 (方块状态) = {'PASS' if t1 else 'FAIL'}   "
      f"Tier 2 (派生字段) = {'PASS' if t2 else 'FAIL'}   "
      f"综合 = {verdict}   (common={len(common)}, hash_diff={len(mismatch)}, "
      f"dh_diff={len(dh_mismatch)}, nz_diff={len(nz_mismatch)}, "
      f"only_old={len(ka-kb)}, only_bulk={len(kb-ka)})")
sys.exit(0 if verdict == "PASS" else 1)
