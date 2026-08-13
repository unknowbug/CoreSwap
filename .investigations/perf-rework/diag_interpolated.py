# diag_interpolated.py —— 诊断 InterpolatedNoiseSampler 拆分误差来源（/o 放大 vs smear）
import struct
import math

def f32(x): return struct.unpack('f', struct.pack('f', x))[0]
GRADIENTS = [(1,1,0),(-1,1,0),(1,-1,0),(-1,-1,0),(1,0,1),(-1,0,1),(1,0,-1),(-1,0,-1),
             (0,1,1),(0,-1,1),(0,1,-1),(0,-1,-1),(1,1,0),(0,-1,1),(-1,1,0),(0,-1,-1)]
def maintainPrecision(v): return v - math.floor(v / 3.3554432E7 + 0.5) * 3.3554432E7

class Perlin:
    def __init__(self, seed):
        self.perm = list(range(256))
        state = seed & 0xffffffff
        for i in range(256):
            state = (state * 1103515245 + 12345) & 0xffffffff
            j = state % 256
            self.perm[i], self.perm[j] = self.perm[j], self.perm[i]
        self.ox = self.oy = self.oz = 0.0
    def map(self, i): return self.perm[i & 0xFF]
    def grad(self, h, x, y, z):
        g = GRADIENTS[h & 15]; return g[0]*x+g[1]*y+g[2]*z
    def section(self, sx, sy, sz, lx, ly, lz, fadeY):
        i=self.map(sx); j=self.map(sx+1); k=self.map(i+sy); l=self.map(i+sy+1)
        m=self.map(j+sy); n=self.map(j+sy+1)
        d=self.grad(self.map(k+sz),lx,ly,lz); e=self.grad(self.map(m+sz),lx-1,ly,lz)
        f=self.grad(self.map(l+sz),lx,ly-1,lz); g=self.grad(self.map(n+sz),lx-1,ly-1,lz)
        h=self.grad(self.map(k+sz+1),lx,ly,lz-1); o=self.grad(self.map(m+sz+1),lx-1,ly,lz-1)
        p=self.grad(self.map(l+sz+1),lx,ly-1,lz-1); q=self.grad(self.map(n+sz+1),lx-1,ly-1,lz-1)
        fade=lambda v: v*v*v*(v*(v*6-15)+10); lerp=lambda dd,s,ee: s+dd*(ee-s)
        r=fade(lx); s=fade(fadeY); t=fade(lz)
        x0=lerp(r,d,e); x1=lerp(r,f,g); x2=lerp(r,h,o); x3=lerp(r,p,q)
        y0=lerp(s,x0,x1); y1=lerp(s,x2,x3)
        return lerp(t,y0,y1)
    def sample5(self, x, y, z, yScale, yMax, frac_bits):
        """frac_bits: 小数精度位数（None=double 精确, 23=float, 17=低精度）"""
        d=x+self.ox; e=y+self.oy; f=z+self.oz
        i=math.floor(d); j=math.floor(e); k=math.floor(f)
        g=d-i; h=e-j; l=f-k
        if frac_bits is not None:
            g=round(g*(2**frac_bits))/(2**frac_bits)  # 模拟限位小数
            h=round(h*(2**frac_bits))/(2**frac_bits)
            l=round(l*(2**frac_bits))/(2**frac_bits)
        if yScale != 0.0:
            m = yMax if (yMax >= 0.0 and yMax < h) else h
            n = math.floor(m / yScale + 1.0E-7) * yScale
        else:
            n = 0.0
        return self.section(i, j, k, g, h - n, l, h)

if __name__ == '__main__':
    pn = Perlin(12345)
    # 单 octave，远坐标，y 带小数（触发 smear）
    x = maintainPrecision(30000001.17 * 171.103 * 0.5)
    y = maintainPrecision(64.0 * 85.5515 * 0.5)
    z = maintainPrecision(30000001.1 * 171.103 * 0.5)
    yScale = 684.412 * 0.5
    yMax = 64.0 * 85.5515 * 0.5
    print(f"坐标 x={x:.6f} y={y:.6f} z={z:.6f} yScale={yScale:.6f} yMax={yMax:.6f}")
    base = pn.sample5(x, y, z, yScale, yMax, None)
    print(f"double 精确: {base:.12f}")
    for bits in [52, 40, 32, 24, 23, 22, 20, 18, 16]:
        v = pn.sample5(x, y, z, yScale, yMax, bits)
        print(f"  小数限 {bits} 位: {v:.12f}  误差={abs(v-base):.3e}")
    print("\n=== 关键：误差被 /o 放大（r 越大 o=2^-r 越小，误差放大 2^r）===")
    for r in [0, 5, 10, 15]:
        o = 2.0 ** (-r)
        xs = maintainPrecision(30000001.17 * 171.103 * o)
        ys = maintainPrecision(64.0 * 85.5515 * o)
        zs = maintainPrecision(30000001.1 * 171.103 * o)
        b = pn.sample5(xs, ys, zs, 684.412*o, 64.0*85.5515*o, None)
        f23 = pn.sample5(xs, ys, zs, 684.412*o, 64.0*85.5515*o, 23)
        err = abs(f23 - b) * 2.0**r  # 除以 o 后的贡献误差
        print(f"  r={r:2d} o=2^-{r}: 单octave误差={abs(f23-b):.3e}, 除以o后贡献误差={err:.3e}")
