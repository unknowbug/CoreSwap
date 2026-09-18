#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""CoreSwap 开关对账门（B6-1）：-P→-D 声明面 × sysprop 消费面 机械核对。

背景（260917-06 立项，260918-01 落地）：
  本工程有 100+ 个 gradle -P 开关，映射表（build.gradle 手工维护）与消费点
  （源码 System.getProperty）是**两份手工事实源**，过去靠人记，已致灾多起
  （#8 漏行静默不生效三犯 / #19 点分名 / #47 映射作用域 / #56 覆盖不全）。
  本工具把两者机械对账。

四家族覆盖（objective 明列的 #8/#19/#47/#56 各有对应检查）：
  #8/#56 漏映射/覆盖不全 → DEAD（声明无消费）+ ORPHAN（消费无声明）+ per-version 缺口
  #47    映射作用域     → run.vmArg 是否落在「接收 run 的闭包」之外（负向测试已验证可失败）
  #19    命名不一致     → 同文件内点分/驼峰并存报告（补映射时须与族对齐）

用法：
    python scripts/check_switch_mapping.py                  # 对账 + 摘要
    python scripts/check_switch_mapping.py --json out.json  # 导出机读结果
    python scripts/check_switch_mapping.py --strict         # DEAD∪单版本缺口∪作用域违规 → 非零退出

判据（族内对称性，260917-06 T4 裁决确立）：
    族内存在 >=1 个 -P 映射先例 => 未映射项 = DEFECT（用户自然预期可用 -P）
    族内零 -P 映射先例           => 未映射项 = SCOPED（族设计即 -D 直传，合法旁路）
  另见豁免子句（探针专用路径 + 缺省权威）在 classification 的 reason 中登记。

覆盖面声明（§9.7）：未扫描的载体一律不判 DEAD；新增载体 MUST 同步更新本表。
聚合口径警告：ORPHAN 为 union 口径，会掩盖单版本缺口 —— 请看 per-version 段。
"""
import io
import json
import os
import re
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# ---- 覆盖面：声明面 / 消费面 carrier（新增载体 MUST 同步）----
GRADLE_FILES = [
    r"versions\1.20.1\java\build.gradle",
    r"versions\1.21.6\java\build.gradle",
]
SRC_DIRS = [
    r"versions\1.20.1\java\src",
    r"versions\1.21.6\java\src",
    r"java-core\src",
    r"worldgen-core\src",
]

# ---- 明确未扫描面（§9.7 覆盖面声明）：这些载体不参与判定，未扫一律不判 DEAD ----
NOT_SCANNED = [
    "native/C++ 与 Rust 侧读 env 变量（非 -D sysprop，形态不同，另论）",
    "外部工具直接 -D 传参（不体现在本仓库源码/脚本内）",
    "MC 发行 jar 内的 sysprop 消费点（如 max.bg.threads 疑为 MC Util 侧消费，未反编译核对）",
    "1.21.6 的 python/JS 辅助脚本中的 -D 拼装（若有）",
]

RE_VMARG = re.compile(r'run\.vmArg\s+"-D([A-Za-z0-9_.]+)=')
RE_GETPROP = re.compile(r'System\.getProperty\("([^"]+)"')
RE_GETINT = re.compile(r'Integer\.getInteger\("([^"]+)"')
RE_GETBOOL = re.compile(r'Boolean\.getBoolean\("([^"]+)"')


def _walk(paths, exts):
    out = []
    for rel in paths:
        p = os.path.join(ROOT, rel)
        if os.path.isfile(p):
            out.append(p)
        elif os.path.isdir(p):
            for root, _, files in os.walk(p):
                for f in files:
                    if f.endswith(exts):
                        out.append(os.path.join(root, f))
    return out


def collect():
    """返回 (union_m1, union_m2, per_version_m1, per_version_m2)。

    per-version 视角为必需（260918-01 发现）：**union 口径会掩盖单版本缺口**——
    若 1.20.1 声明了 `blobProbe.chunkX` 而 1.21.6 未声明，union 口径下该项不在
    ORPHAN 集合中，1.21.6 的缺口被静默掩盖（#105 载体偏差家族）。
    """
    m1, m2 = {}, {}
    pv_m1, pv_m2 = {}, {}
    for gf in _walk(GRADLE_FILES, (".gradle",)):
        t = io.open(gf, encoding="utf-8", errors="replace").read()
        rel = os.path.relpath(gf, ROOT)
        # per-version key：从路径提取版本号（如 versions\1.20.1\java\build.gradle -> 1.20.1）
        m = re.search(r"versions[\\/]([^\\/]+)[\\/]", rel)
        ver = m.group(1) if m else "(shared)"
        pv_m1.setdefault(ver, set())
        for mm in RE_VMARG.finditer(t):
            m1.setdefault(mm.group(1), set()).add(rel)
            pv_m1[ver].add(mm.group(1))
    for sf in _walk(SRC_DIRS, (".java", ".rs")):
        t = io.open(sf, encoding="utf-8", errors="replace").read()
        rel = os.path.relpath(sf, ROOT)
        m = re.search(r"versions[\\/]([^\\/]+)[\\/]", rel)
        ver = m.group(1) if m else "(shared)"
        pv_m2.setdefault(ver, set())
        for rx in (RE_GETPROP, RE_GETINT, RE_GETBOOL):
            for mm in rx.finditer(t):
                m2.setdefault(mm.group(1), set()).add(rel)
                pv_m2[ver].add(mm.group(1))
    return m1, m2, pv_m1, pv_m2


def check_scope_and_naming():
    """#47 作用域检查 + #19 命名一致性检查（B6-1 四家族中另外两族的机械判据）。

    #47：`run.vmArg` 只在「接收 run 参数的闭包」或 loom `runs { }` 块内可见——
    写在闭包外（如 tasks.matching{} 配置期）会失败或静默不生效。判据：
    统计每个 gradle 文件中 `run.vmArg` 的**闭包内外**分布；闭包外的行数 > 0 即告警。
    #19：`-P` 属性名命名形态不统一（点分 'biome6.colDump' vs 驼峰 'blockProbeFull'），
    历史上导致「按族名猜属性名」的静默不生效。判据：报出同族（-D 名前缀）内的
    混合命名形态，供补映射时对齐。
    """
    scope_findings = []
    naming_findings = []
    for gf in _walk(GRADLE_FILES, (".gradle",)):
        rel = os.path.relpath(gf, ROOT)
        lines = io.open(gf, encoding="utf-8", errors="replace").read().split("\n")
        # ---- #47：闭包边界（以 def <name> = { ... } 形式识别接收 run 的闭包）----
        ranges = []
        for i, l in enumerate(lines):
            m = re.match(r"\s*def\s+(\w+)\s*=\s*\{\s*(\w+)\s*->", l)
            if m and m.group(2) == "run":
                depth = 0
                for j in range(i, len(lines)):
                    depth += lines[j].count("{") - lines[j].count("}")
                    if j > i and depth == 0:
                        ranges.append((i, j))
                        break
        for i, l in enumerate(lines):
            if "run.vmArg" not in l:
                continue
            if not any(a <= i <= b for a, b in ranges):
                scope_findings.append((rel, i + 1, l.strip()[:70]))
        # ---- #19：属性名形态（点分 vs 驼峰）----
        props = set(re.findall(r"findProperty\('([A-Za-z0-9_.]+)'\)", "\n".join(lines)))
        dotted = sorted(p for p in props if "." in p)
        camel = sorted(p for p in props if "." not in p and re.search(r"[A-Z]", p))
        if dotted and camel:
            naming_findings.append((rel, len(dotted), len(camel), dotted[:4], camel[:4]))
    return scope_findings, naming_findings


def main():
    m1, m2, pv_m1, pv_m2 = collect()
    d1, d2 = set(m1), set(m2)
    consistent = sorted(d1 & d2)
    dead = sorted(d1 - d2)
    orphan = sorted(d2 - d1)

    print("== 开关对账（B6-1）==")
    print("  声明面 M1（-P->-D 映射）: %d" % len(d1))
    print("  消费面 M2（sysprop 读取）: %d" % len(d2))
    print("  CONSISTENT: %d" % len(consistent))
    print("  DEAD（仅声明，无消费）: %d" % len(dead))
    print("  ORPHAN（仅消费，未声明）: %d" % len(orphan))
    print()
    print("  ⚠️ 口径提示：下面的「union 段」与「per-version 段」名字会重叠但含义不同——")
    print("     union ORPHAN = 两版**都没声明**；per-version 缺口 = **该版**没声明（可能另一版有）。")
    print("     读数字前先确认看的是哪一段（260918-01 实测：主会话曾误读一次）。")
    if dead:
        print("\n---- [union 段] DEAD（两版合并口径：声明但全载体零消费）----")
        for k in dead:
            print("   ", k)
    if orphan:
        print("\n---- [union 段] ORPHAN（两版合并口径：消费但两版都未声明）----")
        for k in orphan:
            print("   ", k)

    # ---- per-version 缺口（union 口径会掩盖单版本缺口，#105 载体偏差家族）----
    shared_cons = pv_m2.get("(shared)", set())
    per_version = {}
    print("\n---- [per-version 段] 单版本缺口（union 口径会掩盖的项）----")
    print("     含义：该版(+共享)有消费点，但该版 build.gradle 未声明 ⇒ 该版用户传 -P 静默无效。")
    for ver in sorted(v for v in pv_m1 if v != "(shared)"):
        consumed = pv_m2.get(ver, set()) | shared_cons
        declared = pv_m1.get(ver, set())
        gap = sorted(consumed - declared)
        per_version[ver] = gap
        union_orphan = sorted(set(gap) & set(orphan))
        other = sorted(set(gap) - set(orphan))
        print("  [%s] 声明 %d / 本版(+共享)消费 %d → 缺口 %d"
              % (ver, len(declared), len(consumed), len(gap)))
        if other:
            print("       ↑ 其中 %d 项**不在 union ORPHAN 中**（= 另一版已声明，掩盖发生在此）:"
                  % len(other))
            for k in other:
                print("           *", k)
        if union_orphan:
            print("       ↑ 其中 %d 项同时在 union ORPHAN 中（两版都缺）: %s"
                  % (len(union_orphan), ", ".join(union_orphan[:6])))

    # ---- #47 作用域 + #19 命名一致性（四家族中另两族的机械判据）----
    scope_findings, naming_findings = check_scope_and_naming()
    print("\n-- #47 作用域检查（run.vmArg 在接收 run 的闭包之外）--")
    if scope_findings:
        for rel, ln, txt in scope_findings:
            print("    [OUT-OF-SCOPE] %s:%d  %s" % (rel, ln, txt))
    else:
        print("    OK：全部 run.vmArg 均在有效闭包内")
    print("\n-- #19 命名一致性（同文件内点分/驼峰并存）--")
    if naming_findings:
        for rel, nd, nc, ds, cs in naming_findings:
            print("    %s：点分 %d（%s…） / 驼峰 %d（%s…）" % (rel, nd, ", ".join(ds), nc, ", ".join(cs)))
        print("    注：二义并存非缺陷，但补映射 MUST 与所在族对齐（#19 三犯家族）")
    else:
        print("    OK：无并存")

    result = {
        "coverage": {
            "declaration": GRADLE_FILES,
            "consumption": SRC_DIRS,
            "not_scanned": NOT_SCANNED,
        },
        "counts": {"M1": len(d1), "M2": len(d2), "consistent": len(consistent),
                   "dead": len(dead), "orphan": len(orphan)},
        "dead": dead, "orphan": orphan,
        "per_version_gaps": per_version,
        "out_of_scope": [{"file": r, "line": l, "text": t} for r, l, t in scope_findings],
    }
    if "--json" in sys.argv:
        out = sys.argv[sys.argv.index("--json") + 1]
        io.open(out, "w", encoding="utf-8").write(json.dumps(result, indent=1, ensure_ascii=False))
        print("\n[OK] ->", out)

    print("\n注：DEAD/ORPHAN 不等于是缺陷——多数为合法旁路（探针直接 -D 手传 / 其它 carrier 消费）。")
    print("    定性见 .investigations/b61-260917-06/t4-adjudication.md（族内对称性判据）。")
    print("    ⚠️ union 口径的 ORPHAN 会掩盖单版本缺口——请以 per-version 段为准。")
    # --strict：DEAD 非空 或 任一版本缺口 或 存在作用域外 run.vmArg 即非零退出
    if "--strict" in sys.argv and (dead or any(per_version.values()) or scope_findings):
        n = len(dead) + sum(len(g) for g in per_version.values()) + len(scope_findings)
        print("\n[FAIL] 存在 %d 项 DEAD/单版本缺口/作用域违规" % n)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
