#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Extract a single jar entry to a file (260912-01 V1b classification helper).

usage: python extract_entry.py <jar> <entry> <outpath>
"""
import io
import sys
import zipfile


def main(argv):
    jar, entry, out = argv
    with zipfile.ZipFile(jar) as z:
        data = z.read(entry)
    with io.open(out, "wb") as f:
        f.write(data)
    print("[OK] %s -> %s (%d bytes)" % (entry, out, len(data)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
