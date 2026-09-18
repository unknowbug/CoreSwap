#!/usr/bin/env python3
"""CoreSwap C++ @anchor 注解扫描工具。

包装 anchorlaw-scanner 的 cpp.py（scan_cpp_file / summarize_cpp），提供：
- 目录递归扫描（*.h/*.cpp/*.hpp/*.cc）
- 每个 anchor 的 valid / source 校验
- 汇总报告（test/idk 数量、invalid 明细、skeleton 文件提示）

用法：
    set PYTHONPATH=E:/PYTHON/Anchorlaw/python/anchorlaw-scanner
    python scripts/scan_cpp_anchors.py [path]            # 扫描单文件或目录
    python scripts/scan_cpp_anchors.py --staleness 90   # 附带 idk 90 天升级提醒（按文件 mtime 近似）

协议：E:/PYTHON/CoreSwap/protocol/verification-protocol.md（§1 注解语法 / §2 staleness）
"""
import argparse
import os
import sys
import time
from datetime import datetime, timezone

if sys.stdout and hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

try:
    from anchorlaw_scanner.cpp import scan_cpp_file, summarize_cpp, is_cpp_file
except ImportError:
    print("错误：找不到 anchorlaw_scanner。先设置 PYTHONPATH 指向 Anchorlaw 源码：")
    print("  set PYTHONPATH=E:\\PYTHON\\Anchorlaw\\python\\anchorlaw-scanner")
    sys.exit(2)

ANCHOR_HINT = "// @anchor.test(\"...\", source=\"probe:...\") 或 // @anchor.idk(\"...\")"


def scan_path(path: str) -> list:
    """扫描文件或目录，返回 (path, anchors) 列表。"""
    results = []
    if os.path.isfile(path):
        if is_cpp_file(path):
            results.append((path, scan_cpp_file(path)))
        return results
    for root, _dirs, files in os.walk(path):
        for name in sorted(files):
            full = os.path.join(root, name)
            if is_cpp_file(full):
                results.append((full, scan_cpp_file(full)))
    return results


def report(results: list, staleness_days: int = 90, show_skeleton: bool = True) -> None:
    total_anchors = 0
    total_test = 0
    total_idk = 0
    total_invalid = 0
    skeleton_files = []
    stale_idks = []
    now = time.time()

    print("=" * 72)
    print("CoreSwap C++ @anchor 扫描报告")
    print(f"时间: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}")
    print("=" * 72)

    for path, anchors in results:
        rel = os.path.relpath(path)
        if not anchors:
            if show_skeleton:
                skeleton_files.append(rel)
            continue
        total_anchors += len(anchors)
        print(f"\n[{rel}]  {len(anchors)} anchors")
        for a in anchors:
            if a.kind == "test":
                total_test += 1
            elif a.kind == "idk":
                total_idk += 1
                # staleness 近似：文件 mtime 早于 staleness_days 的 idk 提示
                age = now - os.path.getmtime(path)
                if age > staleness_days * 86400:
                    stale_idks.append((rel, a.line_number, a.description, int(age // 86400)))
            mark = "OK " if a.valid else "INVALID"
            src = a.source or "(无 source)"
            print(f"  [{a.kind:4s}] {mark} L{a.line_number}: {a.description}")
            if a.kind == "test" and a.source:
                print(f"         source: {a.source}")
            for issue in a.issues:
                print(f"         ⚠ {issue}")
        if not a.valid:
            total_invalid += 1

    print("\n" + "=" * 72)
    print(f"汇总: {total_anchors} anchors | test={total_test} idk={total_idk} invalid={total_invalid}")
    if skeleton_files:
        print(f"\n⚠ 无锚点文件（skeleton，违反第一律，P0/P1 函数应标注）: {len(skeleton_files)}")
        for f in skeleton_files[:20]:
            print(f"   - {f}")
    if stale_idks:
        print(f"\n⚠ idk staleness（超过 {staleness_days} 天未解决）: {len(stale_idks)}")
        for rel, line, desc, days in stale_idks[:10]:
            print(f"   - {rel}:L{line} ({days}天) {desc}")
    if total_invalid:
        print(f"\n❌ invalid anchors: {total_invalid}（test 缺 source 等）")
        sys.exit(1)
    print("\n✅ 所有 anchor 有效")


def main() -> None:
    ap = argparse.ArgumentParser(description="CoreSwap C++ @anchor 扫描")
    ap.add_argument("path", nargs="?", default="versions/1.20.1/cpp/worldgen/src",
                    help="扫描路径（默认 versions/1.20.1/cpp/worldgen/src）")
    ap.add_argument("--staleness", type=int, default=90, help="idk staleness 天数（默认 90）")
    ap.add_argument("--no-skeleton", action="store_true", help="不显示无锚点文件")
    args = ap.parse_args()

    if not os.path.exists(args.path):
        print(f"路径不存在: {args.path}")
        sys.exit(2)

    results = scan_path(args.path)
    if not results:
        print("未找到 C++ 文件")
        sys.exit(0)
    report(results, args.staleness, show_skeleton=not args.no_skeleton)


if __name__ == "__main__":
    main()
