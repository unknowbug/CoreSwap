# interpolated_probe.py —— InterpolatedNoiseSampler（old_blended_noise）完整采样 + 拆分 + smear 的 FP32 误差
# 验证：完整链路（16+16+8 octave + y 轴 smear + 插值）拆分后 float 采样 vs double 全链路
import struct
import math

def f32(x):
    return struct.unpack('f', struct.pack('f', x))[0]

GRADIENTS = [
    (1,1,0),(-1,1,0),(1,-1,0),(-1,-1,0),
    (1,0,1),(-1,0,1),(1,0,-1),(-1,0,-1),
    (0,1,1),(0,-1,1),(0,1,-1),(0,-1,-1),
    (1,1,0),(0,-1,1),(-1,1,0),(0,-1,-1),
]

def maintainPrecision(v):
    return v - math.floor(v / 3.3554432E7 + 0.5) * 3.3554432E7

class Perlin:
    def __init__(self, seed):
        self.perm = list(range(256))
        # 简单伪随机 shuffle（确定性，用于模拟非 identity perm）
        state = seed & 0xffffffff
        for i in range(256):
            state = (state * 1103515245 + 12345) & 0xffffffff
            j = state % 256
            self.perm[i], self.perm[j] = self.perm[j], self.perm[i]
        self.ox = 0.0; self.oy = 0.0; self.oz = 0.0
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
    def sample5_double(self, x, y, z, yScale, yMax):
        """5 参数 sample（含 y 轴 smear），全程 double"""
        d=x+self.ox; e=y+self.oy; f=z+self.oz
        i=math.floor(d); j=math.floor(e); k=math.floor(f)
        g=d-i; h=e-j; l=f-k
        if yScale != 0.0:
            m = yMax if (yMax >= 0.0 and yMax < h) else h
            n = math.floor(m / yScale + 1.0E-7) * yScale
        else:
            n = 0.0
        return self.section(i, j, k, g, h - n, l, h)
    def sample5_split(self, x, y, z, yScale, yMax):
        """5 参数 sample，拆分：整数 int32 + 小数 float，section 内 double（先测小数 float 化误差）"""
        d=x+self.ox; e=y+self.oy; f=z+self.oz
        i=math.floor(d); j=math.floor(e); k=math.floor(f)
        g=f32(d-i); h=f32(e-j); l=f32(f-k)
        if yScale != 0.0:
            m = yMax if (yMax >= 0.0 and yMax < h) else h
            n = math.floor(m / yScale + 1.0E-7) * yScale
        else:
            n = 0.0
        return self.section(i, j, k, g, f32(h - n), l, f32(h))

class Interpolated:
    """InterpolatedNoiseSampler（16+16+8 octave）"""
    def __init__(self):
        # lower/upper: 16 octave (firstOctave=-15)，interpolation: 8 octave (firstOctave=-7)
        self.lower = [Perlin(1000 + r) for r in range(16)]
        self.upper = [Perlin(2000 + r) for r in range(16)]
        self.interp = [Perlin(3000 + p) for p in range(8)]
        self.scaledXzScale = 684.412 * 0.25   # 171.103
        self.scaledYScale = 684.412 * 0.125   # 85.5515
        self.xzFactor = 80.0
        self.yFactor = 160.0
        self.smear = 8.0
    def sample_double(self, x, y, z):
        d = x * self.scaledXzScale
        e = y * self.scaledYScale
        f = z * self.scaledXzScale
        g = d / self.xzFactor
        h = e / self.yFactor
        i = f / self.xzFactor
        j = self.scaledYScale * self.smear
        k = j / self.yFactor
        n = 0.0
        o = 1.0
        for p in range(8):
            pn = self.interp[p]
            if pn is not None:
                n += pn.sample5_double(maintainPrecision(g*o), maintainPrecision(h*o), maintainPrecision(i*o), k*o, h*o) / o
            o /= 2.0
        q = (n / 10.0 + 1.0) / 2.0
        bl = q >= 1.0
        bl2 = q <= 0.0
        l = 0.0; m = 0.0
        o = 1.0
        for r in range(16):
            s = maintainPrecision(d*o); t = maintainPrecision(e*o); u = maintainPrecision(f*o)
            v = j*o
            if not bl:
                pn = self.lower[r]
                if pn is not None:
                    l += pn.sample5_double(s, t, u, v, e*o) / o
            if not bl2:
                pn = self.upper[r]
                if pn is not None:
                    m += pn.sample5_double(s, t, u, v, e*o) / o
            o /= 2.0
        qq = max(0.0, min(1.0, q))
        return (l / 512.0 + qq * (m / 512.0 - l / 512.0)) / 128.0
    def sample_split(self, x, y, z):
        """拆分版：坐标拆分 + 小数 float 化，section 内 double"""
        d = x * self.scaledXzScale
        e = y * self.scaledYScale
        f = z * self.scaledXzScale
        g = d / self.xzFactor
        h = e / self.yFactor
        i = f / self.xzFactor
        j = self.scaledYScale * self.smear
        k = j / self.yFactor
        n = 0.0
        o = 1.0
        for p in range(8):
            pn = self.interp[p]
            if pn is not None:
                n += pn.sample5_split(maintainPrecision(g*o), maintainPrecision(h*o), maintainPrecision(i*o), k*o, h*o) / o
            o /= 2.0
        q = (n / 10.0 + 1.0) / 2.0
        bl = q >= 1.0
        bl2 = q <= 0.0
        l = 0.0; m = 0.0
        o = 1.0
        for r in range(16):
            s = maintainPrecision(d*o); t = maintainPrecision(e*o); u = maintainPrecision(f*o)
            v = j*o
            if not bl:
                pn = self.lower[r]
                if pn is not None:
                    l += pn.sample5_split(s, t, u, v, e*o) / o
            if not bl2:
                pn = self.upper[r]
                if pn is not None:
                    m += pn.sample5_split(s, t, u, v, e*o) / o
            o /= 2.0
        qq = max(0.0, min(1.0, q))
        return (l / 512.0 + qq * (m / 512.0 - l / 512.0)) / 128.0

if __name__ == '__main__':
    ip = Interpolated()
    # 远坐标（3000 万块）+ 小数，y 用带小数的值（触发 smear）
    print("=== InterpolatedNoiseSampler 拆分 float vs double（远坐标 + y 小数）===")
    maxd = 0.0; sumd = 0.0; cnt = 0; maxat = None
    for t in range(200):
        x = (30000000 + (t % 10) * 0.13) * 1.0
        y = (64 + (t % 7) * 0.37) * 1.0
        z = (30000000 + (t % 13) * 0.11) * 1.0
        base = ip.sample_double(x, y, z)
        sp = ip.sample_split(x, y, z)
        diff = abs(sp - base)
        if diff > maxd: maxd = diff; maxat = (x, y, z, sp, base)
        sumd += diff; cnt += 1
    print(f"N={cnt} maxDiff={maxd:.3e} avgDiff={sumd/cnt:.3e}")
    if maxat:
        print(f"  maxDiff @ {maxat[:3]}: split={maxat[3]:.9f} double={maxat[4]:.9f}")
