# coord_precision_probe.py —— 验证坐标 float 化 vs 整数/小数拆分 的 Perlin 噪声误差
# 关键疑点：maintainPrecision 折叠后坐标 ~2^24，float ulp=2，整体 float 化会丢小数部分 → 噪声误差 O(1)?
import struct
import math

GRADIENTS = [
    (1,1,0),(-1,1,0),(1,-1,0),(-1,-1,0),
    (1,0,1),(-1,0,1),(1,0,-1),(-1,0,-1),
    (0,1,1),(0,-1,1),(0,1,-1),(0,-1,-1),
    (1,1,0),(0,-1,1),(-1,1,0),(0,-1,-1),
]

def f32(x):
    return struct.unpack('f', struct.pack('f', x))[0]

class Perlin:
    def __init__(self):
        self.perm = list(range(256))  # identity
        self.ox = self.oy = self.oz = 0.0
    def map(self, i):
        return self.perm[i & 0xFF]
    def grad(self, h, x, y, z):
        g = GRADIENTS[h & 15]
        return g[0]*x + g[1]*y + g[2]*z
    def section(self, sx, sy, sz, lx, ly, lz, fadeY):
        i=self.map(sx); j=self.map(sx+1); k=self.map(i+sy); l=self.map(i+sy+1)
        m=self.map(j+sy); n=self.map(j+sy+1)
        d=self.grad(self.map(k+sz),lx,ly,lz); e=self.grad(self.map(m+sz),lx-1,ly,lz)
        f=self.grad(self.map(l+sz),lx,ly-1,lz); g=self.grad(self.map(n+sz),lx-1,ly-1,lz)
        h=self.grad(self.map(k+sz+1),lx,ly,lz-1); o=self.grad(self.map(m+sz+1),lx-1,ly,lz-1)
        p=self.grad(self.map(l+sz+1),lx,ly-1,lz-1); q=self.grad(self.map(n+sz+1),lx-1,ly-1,lz-1)
        def fade(v): return v*v*v*(v*(v*6-15)+10)
        def lerp(dd,s,ee): return s+dd*(ee-s)
        r=fade(lx); s=fade(fadeY); t=fade(lz)
        x0=lerp(r,d,e); x1=lerp(r,f,g); x2=lerp(r,h,o); x3=lerp(r,p,q)
        y0=lerp(s,x0,x1); y1=lerp(s,x2,x3)
        return lerp(t,y0,y1)
    def sample_double(self, x, y, z):
        # 基线：全程 double（vanilla 语义）
        d=x+self.ox; e=y+self.oy; f=z+self.oz
        i=math.floor(d); j=math.floor(e); k=math.floor(f)
        g=d-i; h=e-j; l=f-k
        return self.section(i,j,k,g,h,l,h)
    def sample_split(self, x, y, z):
        # 方案 B：整数(int32 精确) + 小数(float) 拆分，section 内 double
        d=x+self.ox; e=y+self.oy; f=z+self.oz
        i=math.floor(d); j=math.floor(e); k=math.floor(f)
        g=f32(d-i); h=f32(e-j); l=f32(f-k)  # 小数部分 float 化
        return self.section(i,j,k,g,h,l,h)
    def sample_f32coord(self, x, y, z):
        # 方案 A：坐标整体 float 化，然后 double 采样
        return self.sample_double(f32(x), f32(y), f32(z))

def maintainPrecision(v):
    return v - math.floor(v / 3.3554432E7 + 0.5) * 3.3554432E7

if __name__ == '__main__':
    p = Perlin()
    # 坐标：模拟远坐标（3000万块 × scale 171）折叠前后
    SCALE = 171.103
    tests = [
        ("近坐标(720块)", 720 * SCALE),
        ("中坐标(1万块)", 10000 * SCALE),
        ("远坐标(3000万块)", 30000000 * SCALE),
        ("折叠后(~2^24)", 16777216.5),
    ]
    print("=== 噪声误差对比（Perlin，identity perm, origin=0）===")
    print(f"{'坐标场景':<20}{'坐标值':>18}{'整体float化':>14}{'拆分(小数float)':>16}")
    for name, coord in tests:
        x = coord; y = coord * 0.7; z = coord * 1.3  # 三个不同倍数
        base = p.sample_double(x, y, z)
        # 方案 A：整体 float 化
        fa = p.sample_f32coord(x, y, z)
        errA = abs(fa - base)
        # 方案 B：折叠 + 拆分
        fx = maintainPrecision(x); fy = maintainPrecision(y); fz = maintainPrecision(z)
        fb = p.sample_split(fx, fy, fz)
        errB = abs(fb - base)  # 注意 base 未折叠，折叠本身是语义一部分；这里只看拆分引入的误差
        # 折叠后 base
        base_folded = p.sample_double(fx, fy, fz)
        errB2 = abs(fb - base_folded)  # 拆分相对折叠后 base 的误差
        print(f"{name:<20}{coord:>18.1f}{errA:>14.3e}{errB2:>16.3e}")

    print("\n=== 关键问题：折叠后坐标的 float 表示 ===")
    folded = maintainPrecision(30000000 * SCALE)
    print(f"远坐标折叠后 = {folded:.6f}")
    print(f"float(折叠后) = {f32(folded):.6f}, 误差 = {abs(f32(folded) - folded):.6f}")
    print(f"折叠后小数部分 = {folded - math.floor(folded):.6f}（float 化后是否保留）")
