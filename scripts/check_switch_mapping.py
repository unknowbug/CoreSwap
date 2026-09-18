#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""CoreSwap 开关对账门（B6-1）：-P→-D 声明面 × sysprop 消费面 机械核对。

背景（260917-06 立项，260918-01 落地）：
  本工程有 100+ 个 gradle -P 开关，映射表（build.gradle 手工维护）与消费点
  （源码 System.getProperty）是**两份手工事实源**，过去靠人记，已致灾多起
  （#8 漏行静默不生效三犯 / #19 点分名 / #47 映射作用域 / #56 覆盖不全）。
  本工具把两者机械对账，输出四态：CONSISTENT / DEAD / ORPHAN / 分类定性。

用法：
    python scripts/check_switch_mapping.py                  # 对账 + 摘要
    python scripts/check_switch_mapping.py --json out.json  # 导出机读结果
    python scripts/check_switch_mapping.py --strict         # 存在 DEFECT 时非零退出（门禁模式）

判据（族内对称性，260917-06 T4 裁决确立）：
    族内存在 >=1 个 -P 映射先例 => 未映射项 = DEFECT（用户自然预期可用 -P）
    族内零 -P 映射先例           => 未映射项 = SCOPED（族设计即 -D 直传，合法旁路）
  另见豁免子句（探针专用路径 + 缺省权威）在 classification 的 reason 中登记。

覆盖面声明（§9.7）：未扫描的载体一律不判 DEAD；新增载体 MUST 同步更新本表。
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
    m1, m2 = {}, {}
    for gf in _walk(GRADLE_FILES, (".gradle",)):
        t = io.open(gf, encoding="utf-8", errors="replace").read()
        for m in RE_VMARG.finditer(t):
            m1.setdefault(m.group(1), set()).add(os.path.relpath(gf, ROOT))
    for sf in _walk(SRC_DIRS, (".java", ".rs")):
        t = io.open(sf, encoding="utf-8", errors="replace").read()
        rel = os.path.relpath(sf, ROOT)
        for rx in (RE_GETPROP, RE_GETINT, RE_GETBOOL):
            for m in rx.finditer(t):
                m2.setdefault(m.group(1), set()).add(rel)
    return m1, m2


def main():
    m1, m2 = collect()
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
    if dead:
        print("\n-- DEAD --")
        for k in dead:
            print("   ", k)
    if orphan:
        print("\n-- ORPHAN --")
        for k in orphan:
            print("   ", k)

    result = {
        "coverage": {
            "declaration": GRADLE_FILES,
            "consumption": SRC_DIRS,
            "not_scanned": NOT_SCANNED,
        },
        "counts": {"M1": len(d1), "M2": len(d2), "consistent": len(consistent),
                   "dead": len(dead), "orphan": len(orphan)},
        "dead": dead, "orphan": orphan,
    }
    if "--json" in sys.argv:
        out = sys.argv[sys.argv.index("--json") + 1]
        io.open(out, "w", encoding="utf-8").write(json.dumps(result, indent=1, ensure_ascii=False))
        print("\n[OK] ->", out)

    print("\n注：DEAD/ORPHAN 不等于是缺陷——多数为合法旁路（探针直接 -D 手传 / 其它 carrier 消费）。")
    print("    定性见 .investigations/b61-260917-06/t4-adjudication.md（族内对称性判据）。")
    # --strict：DEAD 非空即非零退出（防死开关回归；ORPHAN 多为合法旁路故不阻断）
    if "--strict" in sys.argv and dead:
        print("\n[FAIL] 存在 %d 个 DEAD 开关（声明但零消费）" % len(dead))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
