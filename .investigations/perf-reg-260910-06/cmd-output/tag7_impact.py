import sys, os, struct, zlib, gzip, collections
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
# 对照：同一 region 目录，用 u1(旧) / i4(新) 两种 tag7 长度读法，比较解析成功数与 chunk 键集
def make(read_i4):
    class R:
        def __init__(s,b): s.b=b; s.i=0
        def u1(s): v=s.b[s.i]; s.i+=1; return v
        def u2(s): v=struct.unpack_from('>H',s.b,s.i)[0]; s.i+=2; return v
        def i4(s): v=struct.unpack_from('>i',s.b,s.i)[0]; s.i+=4; return v
        def i8(s): v=struct.unpack_from('>q',s.b,s.i)[0]; s.i+=8; return v
        def f4(s): v=struct.unpack_from('>f',s.b,s.i)[0]; s.i+=4; return v
        def f8(s): v=struct.unpack_from('>d',s.b,s.i)[0]; s.i+=8; return v
        def n(s,n): v=s.b[s.i:s.i+n]; s.i+=n; return v
    def payload(r,t):
        if t==1: return r.u1()
        if t==2: return r.u2()
        if t==3: return r.i4()
        if t==4: return r.i8()
        if t==5: return r.f4()
        if t==6: return r.f8()
        if t==7:
            n = r.i4() if read_i4 else r.u1()
            return r.n(n)
        if t==8: n=r.u2(); return r.n(n)
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
        raise ValueError('tag %d'%t)
    return R, payload
def scan(region_dir, read_i4):
    R, payload = make(read_i4)
    keys=set(); ok=0; bad=0; badstatus=0
    for fn in sorted(f for f in os.listdir(region_dir) if f.endswith('.mca')):
        m=fn[2:-4].split('.'); rx,rz=int(m[0]),int(m[1])
        raw=open(os.path.join(region_dir,fn),'rb').read()
        if len(raw)<8192: continue
        for i in range(1024):
            off=(raw[i*4]<<16)|(raw[i*4+1]<<8)|raw[i*4+2]
            if off==0: continue
            st=off*4096; ln=struct.unpack_from('>I',raw,st)[0]; comp=raw[st+4]
            body=raw[st+5:st+4+ln]
            try:
                if comp==2: body=zlib.decompress(body)
                elif comp==1: body=gzip.decompress(body)
            except Exception: continue
            try:
                r=R(body); t=r.u1(); r.n(r.u2()); d=payload(r,t)
                d = d.get('') if isinstance(d,dict) and '' in d else d
                ok+=1
                stt = d.get('Status') if isinstance(d,dict) else None
                if not isinstance(stt,str): badstatus+=1
                keys.add((rx*32+(i&31), rz*32+(i>>5)))
            except Exception: bad+=1
    return ok,bad,badstatus,keys
base=r"E:\PYTHON\CoreSwap\.investigations\perf-reg-260910-06\cmd-output"
for tag in ["region-os-r1"]:
    d=os.path.join(base,tag)
    o1=scan(d,False); o2=scan(d,True)
    print(f"{tag}: 旧(u1) ok={o1[0]} bad={o1[1]} badStatus={o1[2]} keys={len(o1[3])}")
    print(f"{tag}: 新(i4) ok={o2[0]} bad={o2[1]} badStatus={o2[2]} keys={len(o2[3])}")
    print("  keys 差:", len(o2[3]-o1[3]), "only_old=", len(o1[3]-o2[3]))
    print("  旧有 keys 中 Status 非字符串(=疑似失步) =", o1[2], " / 新 =", o2[2])
