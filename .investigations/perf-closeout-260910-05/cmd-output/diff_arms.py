import sys, os, struct, zlib, gzip, json
from collections import Counter
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

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
    if t==1: return r.u1()
    if t==2: return r.u2()
    if t==3: return r.i4()
    if t==4: return r.i8()
    if t==5: return r.f4()
    if t==6: return r.f8()
    if t==7: return r.n(r.u1())
    if t==8: return r.n(r.u2()).decode('utf-8','replace')
    if t==9:
        it=r.u1(); ln=r.i4(); return [payload(r,it) for _ in range(ln)]
    if t==10:
        d={}
        while True:
            tt=r.u1()
            if tt==0: break
            nm=r.n(r.u2()).decode('utf-8','replace')
            d[nm]=payload(r,tt)
        return d
    if t==11: ln=r.i4(); return [r.i4() for _ in range(ln)]
    if t==12: ln=r.i4(); return [r.i8() for _ in range(ln)]
    raise ValueError(f'tag {t}')

def read_nbt(data):
    r=R(data); t=r.u1(); r.n(r.u2())
    return payload(r,t)

def iter_chunks(path):
    raw=open(path,'rb').read()
    if len(raw)<8192: return
    for i in range(1024):
        off=(raw[i*4]<<16)|(raw[i*4+1]<<8)|raw[i*4+2]
        if off==0: continue
        start=off*4096
        ln=struct.unpack_from('>I',raw,start)[0]
        comp=raw[start+4]
        body=raw[start+5:start+4+ln]
        try:
            if comp==2: body=zlib.decompress(body)
            elif comp==1: body=gzip.decompress(body)
        except Exception:
            continue
        try: yield read_nbt(body)
        except Exception: continue

def load_world(root):
    # returns {(cx,cz): chunkNBT}
    w={}
    rdir=os.path.join(root,'region')
    if not os.path.isdir(rdir): rdir=root
    for fn in os.listdir(rdir):
        if not fn.endswith('.mca'): continue
        m=fn[2:-4].split('.')
        rx,rz=int(m[0]),int(m[1])
        for nbt in iter_chunks(os.path.join(rdir,fn)):
            data=nbt.get('') if isinstance(nbt,dict) and '' in nbt else nbt
            if not isinstance(data,dict): continue
            pos=data.get('xPos'),data.get('zPos')
            if pos[0] is None: continue
            w[(pos[0],pos[1])]=data
    return w

def block_at(chunk,x,y,z):
    # 1.21.6 chunk NBT: sections[{Y,block_states:{palette, data}}]（无 xPos 已处理）
    cy=y>>4
    for sec in chunk.get('sections',[]):
        if sec.get('Y')==cy:
            bs=sec.get('block_states')
            if bs is None: return 'minecraft:air'
            pal=bs['palette']
            data=bs.get('data')
            ly,lz,lx=y&15,z&15,x&15
            if data is None:
                e=pal[0]; return e.get('Name','?') if isinstance(e,dict) else str(e)
            idx=lx + lz*16 + ly*256
            # bit packing: indices little-endian across longs (MC 1.16+)
            bits=max(4,(len(pal)-1).bit_length())
            per=64//bits
            li=idx//per; off=idx%per
            v=(data[li]>> (off*bits)) & ((1<<bits)-1)
            # longs are signed 64: python int fine
            e=pal[v]
            return e.get('Name','?') if isinstance(e,dict) else str(e)
    return 'minecraft:air'

van=load_world(sys.argv[1])
cs =load_world(sys.argv[2])
print(f"vanilla chunks={len(van)} coreswap chunks={len(cs)}")
common=set(van)&set(cs)
print(f"common={len(common)}")

# 1) 定点：15 个邻居更新坐标
pts=[(-522,63,139),(-805,57,138),(-500,6,240),(-874,48,-588),(-296,48,-18),
     (-443,60,311),(-556,60,316),(-71,56,-517),(-1531,61,239),(-471,45,-215),
     (-592,42,216),(-527,52,78),(-424,62,74),(-196,47,295)]
print('===定点对拍（邻居更新坐标 ±4 柱采样）===')
for (x,y,z) in pts:
    cx,cz=x>>4,z>>4
    if (cx,cz) not in common:
        print(f'({x},{y},{z}) chunk({cx},{cz}) NOT in both arms'); continue
    # 采样该坐标上下 6 格 + 同柱
    rows=[]
    for yy in range(y-2,y+3):
        bv=block_at(van[(cx,cz)],x&15,yy,z&15)
        bc=block_at(cs[(cx,cz)],x&15,yy,z&15)
        rows.append(f'  y={yy}: V={bv.split(":")[-1]:<14} C={bc.split(":")[-1]:<14} {"DIFF" if bv!=bc else ""}')
    print(f'pos({x},{y},{z}):')
    print('\n'.join(rows))

# 2) 普查：差分统计（块对 + y 分布）——section 级快路径
print('===全域差分普查===')

def pal_names(pal):
    return [e.get('Name','?') if isinstance(e,dict) else str(e) for e in pal]

def decode(bs):
    pal=pal_names(bs['palette']); data=bs.get('data')
    if data is None: return [pal[0]]*4096
    bits=max(4,(len(pal)-1).bit_length()); per=64//bits
    out=[]
    for li in range(4096):
        v=(data[li//per]>>((li%per)*bits))&((1<<bits)-1)
        out.append(pal[v] if v<len(pal) else '?')
    return out

pairs=Counter(); ydist=Counter(); ndiff=0; nblock=0; sec_same=0; sec_diff=0
for key in sorted(common):
    cx,cz=key
    vsecs={s.get('Y'):s for s in van[key].get('sections',[]) if s.get('Y') is not None}
    csecs={s.get('Y'):s for s in cs[key].get('sections',[]) if s.get('Y') is not None}
    for cy in set(vsecs)|set(csecs):
        vs,csn=vsecs.get(cy),csecs.get(cy)
        vb=vs.get('block_states') if vs else None
        cb=csn.get('block_states') if csn else None
        vdec=None; cdec=None
        if vb is None and cb is None: continue
        if vb is None: vdec=['minecraft:air']*4096
        else: vdec=decode(vb)
        if cb is None: cdec=['minecraft:air']*4096
        else: cdec=decode(cb)
        if vdec==cdec: sec_same+=1; nblock+=4096; continue
        sec_diff+=1
        for i in range(4096):
            nblock+=1
            if vdec[i]!=cdec[i]:
                ndiff+=1; pairs[(vdec[i],cdec[i])]+=1
                ydist[cy*16]+=1
print(f'blocks={nblock} diff={ndiff} ({100*ndiff/max(1,nblock):.4f}%) sections same/diff={sec_same}/{sec_diff}')
print('top pairs:')
for (a,b),c in pairs.most_common(20): print(f'  {a} -> {b}: {c}')
print('y-section dist:', sorted(ydist.items()))
