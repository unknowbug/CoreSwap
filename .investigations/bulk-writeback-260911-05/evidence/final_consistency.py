# -*- coding: utf-8 -*-
"""C 线收尾三源一致性核对（judge 基线）：artifacts 快照 ↔ 证据文件 ↔ 记录/结论载体。
只读，不改任何文件。核 6 类：① 切片 A 9 行数值 ② Tier 1/2 三类差 ③ 前提实测 ④ 稀释归属 ⑤ Tier 4 ⑥ 三载体措辞是否已改判。
"""
import re, os, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
R = r"E:\PYTHON\CoreSwap"
EV = os.path.join(R, ".investigations", "bulk-writeback-260911-05", "evidence")
REC = os.path.join(R, ".investigations", "bulk-writeback-260911-05", "record-260911-05.md")
VER = os.path.join(R, ".artifacts", "bulk-writeback-260911-05", "verdict-260911-05.md")
IDX = os.path.join(R, ".artifacts", "index.yaml")
ok = bad = 0


def chk(name, cond, detail=""):
    global ok, bad
    if cond:
        ok += 1; print(f"[OK]   {name} {detail}")
    else:
        bad += 1; print(f"[FAIL] {name} {detail}")


def grab(path, pat):
    m = re.search(pat, open(path, encoding="utf-8", errors="replace").read())
    return m.group(1) if m else None


def slice_a(path):
    """切片 A 的百分比在标题的**下一行**（'=== 切片 A：… ===\\n\\nblocks=… (0.0249%)'）"""
    t = open(path, encoding="utf-8", errors="replace").read()
    m = re.search(r"切片 A：[^\n]*\n(?:[^\n]*\n)*?[^\n]*?\((\d+\.\d+)%\)", t)
    return m.group(1) if m else None


print("=== ① 切片 A 九行：证据文件 vs 记录 §5.4 表 ===")
exp = {"slice-ctrl1-old_old2.txt": "0.0249", "slice-ctrl2-old2_old3.txt": "0.0284", "slice-ctrl3-old_old3.txt": "0.0239",
       "slice-bulk1_bulk2.txt": "0.0352", "slice-bulk1_bulk3.txt": "0.0358", "slice-bulk2_bulk3.txt": "0.0272",
       "slice-test-old_bulk.txt": "0.0348", "slice-nr-old_bulk-norandom.txt": "0.0311"}
rec = open(REC, encoding="utf-8").read()
for f, v in exp.items():
    p = os.path.join(EV, f)
    if not os.path.exists(p):
        chk(f, False, "证据文件缺失"); continue
    got = slice_a(p)
    chk(f, got == v, f"证据={got} 期望={v} 记录含={v in rec}")

print("\n=== ② Tier 1/2：cmp-tier12 三类差 ===")
for dim in ("overworld", "nether", "end"):
    p = os.path.join(EV, f"cmp-tier12-{dim}.txt")
    if not os.path.exists(p):
        chk(dim, False, "缺失"); continue
    t = open(p, encoding="utf-8", errors="replace").read()
    chk(f"cmp-tier12-{dim}", "hash_diff=0" in t and "dh_diff=0" in t and "nz_diff=0" in t and "only_old=0" in t and "only_bulk=0" in t,
        (grab(p, r"common=(\d+)") or "?") + " common")

print("\n=== ③ 前提实测（wbcheck） ===")
p = os.path.join(EV, "wbcheck-premise.txt")
t = open(p, encoding="utf-8", errors="replace").read() if os.path.exists(p) else ""
chk("stale_nonair=0", "stale_nonair=0" in t, grab(p, r"air_skipped=(\d+)") or "")
chk("记录引用该数字", "40,329,368" in rec)

print("\n=== ④ 稀释归属 ===")
p = os.path.join(EV, "tier3-attribution.txt")
t = open(p, encoding="utf-8", errors="replace").read() if os.path.exists(p) else ""
chk("both_empty diff=0", bool(re.search(r"both_empty ===  chunk=1854  blocks=182255616  diff=0", t)))
chk("any_content 0.0661%", "0.0661%" in t)
chk("100% 落在有内容 chunk", "任一侧有内容」chunk 的比例 = 100.00%" in t)

print("\n=== ⑤ Tier 4 两轮 + 字段名差异 ===")
p = os.path.join(EV, "tier4-round1-daemon.txt")
t = open(p, encoding="utf-8", errors="replace").read() if os.path.exists(p) else ""
chk("第 1 轮旧值 2070297", "2070297" in t)
chk("第 1 轮新值 1052606", "1052606" in t)
chk("字段名 pack=（非 build）", "pack=30403" in t)

print("\n=== ⑥ 三载体措辞是否已改判（M1） ===")
# §15.4 要求被取代的原文保留不改 ⇒ 允许「不可判」出现，但必须**伴随取代/保留标记**。
# 另：§3.3/§3.4 是**错误记录本体**（叙述「不该这么写」），E4 教训是**规则条款**——这些不是结论定性，合法。
SUP = ("原文保留不改", "保留不改", "已被取代", "判决取代", "原 §5.4", "已被上方判决取代", "原表述", "原结论", "写成「不可判」", "不可达/不可判")


def c_block(text):
    """index.yaml 只扫 C 块（从 '260911-05 C 块' 到文件末），排除其它工作块条目"""
    i = text.find("260911-05 C 块")
    return text[i:] if i >= 0 else text


def rec_conclusion(text):
    """record 只扫结论区：§1 摘要 + §5（排除 §3.3/§3.4 错误记录本体）"""
    a = text.find("## 1. 结论摘要")
    b = text.find("## 6. R8 / R9")
    return text[a:b] if a >= 0 and b > a else text


for name, path, scope in (("record", REC, rec_conclusion), ("verdict", VER, lambda t: t), ("index.yaml", IDX, c_block)):
    t = open(path, encoding="utf-8", errors="replace").read()
    chk(f"{name}: 含「判据未满足」", "判据未满足" in t)
    chk(f"{name}: 含 bulk × bulk / bulk×bulk", ("bulk × bulk" in t or "bulk×bulk" in t))
    sc = scope(t)
    unmarked = []
    for m in re.finditer("不可判", sc):
        w = sc[max(0, m.start() - 400):m.end() + 400]
        if not any(s in w for s in SUP):
            unmarked.append(sc[max(0, m.start() - 60):m.end() + 60].replace("\n", " "))
    chk(f"{name}: 结论区未标记的「不可判」= 0", not unmarked, f"({len(unmarked)} 处)" + ("".join("\n         " + u for u in unmarked[:3])))
    tot = len(re.findall("不可判", t))
    print(f"       [INFO] {name} 全文「不可判」共 {tot} 处，结论区 {len(re.findall('不可判', sc))} 处（其余为错误记录本体/其它工作块）")

print(f"\n===== 合计 OK={ok} FAIL={bad} =====")
