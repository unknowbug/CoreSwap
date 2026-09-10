import sys, os, struct, zlib, gzip, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

tagcount = collections.Counter()

class R:
    def __init__(self, b): self.b=b; self.i=0
    def u1(self): v=self.b[self.i]; self.i+=1; return v
    def u2(self): v=struct.unpack_from('>H',self.b,self.i)[0]; self.i+=2; return v
    def i4(self): v=struct.unpack_from('>i',self.b,self.i)[0]; self.i+=4; return v
    def i8(self): v=struct.unpack_from('>q',self.b,self.i)[0]; self.i+=8; return v
    def f4(self): v=struct.unpack_from('>f',self.b,self.i)[0]; self.i+=4; return v
    def f8(self): v=struct.unpack_from('>d',self.b,self.i)[0]; self.i+=8; return v
    def n(self,n): v=self.b[self.i:self.i+n]; self.i+=n; return v

def payload(r,t):
    tagcount[t]+=1
    if t==1: return r.u1()
    if t==2: return r.u2()
    if t==3: return r.i4()
    if t==4: return r.i8()
    if t==5: return r.f4()
    if t==6: return r.f8()
    if t==7:
        # ⚠️ 本普查脚本**只按 u1（1 字节）**读长度 = 与「旧工具」同口径，**故意如此**：
        #    目的是暴露**失步签名**（普查里出现非 NBT 合法 tag 号 15/16/…/255）。
        #    ⇒ 普查得到的 tag 7 计数（9046）**不能当精确出现次数**（含失步后的垃圾字节误判）。
        #    规范读法对照见 tag7_impact.py（t==7 → i4：全量 ok=7749 / bad=0）。
        n1 = r.u1()
        return ('BYTE_ARRAY_len_u1', n1)
    if t==8:
        n=r.u2(); return r.n(n)
    if t==9:
        et=r.u1(); n=r.i4(); return [payload(r,et) for _ in range(n)]
    if t==10:
        d={}
        while True:
            tt=r.u1()
            if tt==0: return d
            nm=r.n(r.u2()); d[nm]=payload(r,tt)
    if t==11:
        n=r.i4(); return [r.i4() for _ in range(n)]
    if t==12:
        n=r.i4(); return [r.i8() for _ in range(n)]
    raise ValueError('tag %d' % t)

def scan(region_dir, limit_files=None):
    files = sorted(f for f in os.listdir(region_dir) if f.endswith('.mca'))
    if limit_files: files = files[:limit_files]
    ok=0; bad=0
    for fn in files:
        raw=open(os.path.join(region_dir,fn),'rb').read()
        if len(raw)<8192: continue
        for i in range(1024):
            off=(raw[i*4]<<16)|(raw[i*4+1]<<8)|raw[i*4+2]
            if off==0: continue
            st=off*4096
            ln=struct.unpack_from('>I',raw,st)[0]; comp=raw[st+4]
            body=raw[st+5:st+4+ln]
            try:
                if comp==2: body=zlib.decompress(body)
                elif comp==1: body=gzip.decompress(body)
            except Exception: continue
            try:
                r=R(body); t=r.u1(); r.n(r.u2()); payload(r,t); ok+=1
            except Exception: bad+=1
    return ok,bad

root=r"E:\PYTHON\CoreSwap\.investigations"
print("== 1.20.1 (region-os-r1) 全量 ==")
print(scan(os.path.join(root,"perf-reg-260910-06","cmd-output","region-os-r1")))
print("== 1.21.6 (region-na-r1, 260910-05) 全量 ==")
print(scan(os.path.join(root,"perf-closeout-260910-05","cmd-output","region-na-r1")))
print("== tag 出现次数（两个载体合计） ==")
for t in sorted(tagcount):
    print(f"  tag {t:2d}: {tagcount[t]}")
