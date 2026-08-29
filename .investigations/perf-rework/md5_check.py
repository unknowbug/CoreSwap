import hashlib
d = hashlib.md5(b'octave_0').digest()
lo = int.from_bytes(d[0:8], 'big')
hi = int.from_bytes(d[8:16], 'big')
print(f'lo=0x{lo:016x} hi=0x{hi:016x}')
