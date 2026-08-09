# -*- coding: utf-8 -*-
import struct, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
p = r"E:\PYTHON\CoreSwap\versions\1.20.1\data\vanilla_3005152118058349760_4_-1320400_-198064.blocks"
f = open(p, "rb")
magic, seed = struct.unpack(">iq", f.read(12))
size, ox, oz, miny, h = struct.unpack(">iiiii", f.read(20))
print("magic=%#x seed=%d size=%d origin=(%d,%d) miny=%d h=%d" % (magic, seed, size, ox, oz, miny, h))
f.close()
