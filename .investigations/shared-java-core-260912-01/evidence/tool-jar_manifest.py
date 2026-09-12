#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
V1 等价门工具：jar 条目级 sha256 清单（CoreSwap 共享 Java 适配核抽取 260912-01）

用法:
    python jar_manifest.py <jar> [<out.tsv>]

输出（out 省略时打到 stdout）:
    # jar <path>
    # jar_sha256 <sha>
    # entries <n>
    <entry_sha256>\t<entry_name>          (按 entry_name 升序)
    # classes <n>   (以 .class 结尾的条目数，便于与 KB #116 的「非 META-INF 条目数」对齐)

判据（计划 §5 V1）: 抽取前 pre jar 与抽取后 post jar 的**非 mixin 条目** sha 全等；
差异条目 MUST 逐条声明原因。
"""
import hashlib
import io
import os
import sys
import zipfile

sys.stdout.reconfigure(encoding="utf-8", errors="replace")


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    jar = sys.argv[1]
    out_lines = []
    with zipfile.ZipFile(jar) as zf:
        rows = []
        for info in zf.infolist():
            if info.is_dir():
                continue
            data = zf.read(info.filename)
            rows.append((info.filename, hashlib.sha256(data).hexdigest()))
        rows.sort(key=lambda r: r[0])
        out_lines.append("# jar " + os.path.abspath(jar))
        out_lines.append("# jar_sha256 " + sha256_file(jar))
        out_lines.append("# entries %d" % len(rows))
        for name, digest in rows:
            out_lines.append("%s\t%s" % (digest, name))
        out_lines.append("# classes %d" % sum(1 for n, _ in rows if n.endswith(".class")))
        out_lines.append("# non_meta %d" % sum(1 for n, _ in rows if not n.startswith("META-INF/")))
    text = "\n".join(out_lines) + "\n"
    if len(sys.argv) >= 3:
        with io.open(sys.argv[2], "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        print("[OK] wrote %s (%d entries)" % (sys.argv[2], len(out_lines) - 4))
    else:
        print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
