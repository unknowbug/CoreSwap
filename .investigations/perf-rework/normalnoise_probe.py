# normalnoise_probe.py —— 验证 NormalNoise（DoublePerlinNoiseSampler）的 double 拆分 + float 采样误差
# 模拟 DFC shader 的采样逻辑（double 坐标拆分 + float grad/fade/lerp）vs 纯 double
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
    def section(self, sx, sy, sz, lx, ly, lz):
        i=self.map(sx); j=self.map(sx+1); k=self.map(i+sy); l=self.map(i+sy+1)
        m=self.map(j+sy); n=self.map(j+sy+1)
        d=self.grad(self.map(k+sz),lx,ly,lz); e=self.grad(self.map(m+sz),lx-1,ly,lz)
        f=self.grad(self.map(l+sz),lx,ly-1,lz); g=self.grad(self.map(n+sz),lx-1,ly-1,lz)
        h=self.grad(self.map(k+sz+1),lx,ly,lz-1); o=self.grad(self.map(m+sz+1),lx-1,ly,lz-1)
        p=self.grad(self.map(l+sz+1),lx,ly-1,lz-1); q=self.grad(self.map(n+sz+1),lx-1,ly-1,lz-1)
        fade=lambda v: v*v*v*(v*(v*6-15)+10); lerp=lambda dd,s,ee: s+dd*(ee-s)
        r=fade(lx); s=fade(ly); t=fade(lz)
        x0=lerp(r,d,e); x1=lerp(r,f,g); x2=lerp(r,h,o); x3=lerp(r,p,q)
        y0=lerp(s,x0,x1); y1=lerp(s,x2,x3)
        return lerp(t,y0,y1)
    def sample_double(self, x, y, z):
        # 纯 double（含 origin + floor），3 参数 sample
        d=x+self.ox; e=y+self.oy; f=z+self.oz
        i=math.floor(d); j=math.floor(e); k=math.floor(f)
        return self.section(i,j,k,d-i,e-j,f-k)
    def sample_split_f32(self, x, y, z):
        # 拆分：int32 整数 + float 小数，section 内 float
        d=x+self.ox; e=y+self.oy; f=z+self.oz
        i=math.floor(d); j=math.floor(e); k=math.floor(f)
        return self.section_f32(i,j,k,f32(d-i),f32(e-j),f32(f-k))
    def section_f32(self, sx, sy, sz, lx, ly, lz):
        # float 采样
        i=self.map(sx); j=self.map(sx+1); k=self.map(i+sy); l=self.map(i+sy+1)
        m=self.map(j+sy); n=self.map(j+sy+1)
        def gf(h,x,y,z):
            g=GRADIENTS[h&15]; return f32(f32(g[0]*x)+f32(g[1]*y)+f32(g[2]*z))
        d=gf(self.map(k+sz),lx,ly,lz); e=gf(self.map(m+sz),lx-1,ly,lz)
        f=gf(self.map(l+sz),lx,ly-1,lz); g=gf(self.map(n+sz),lx-1,ly-1,lz)
        h=gf(self.map(k+sz+1),lx,ly,lz-1); o=gf(self.map(m+sz+1),lx-1,ly,lz-1)
        p=gf(self.map(l+sz+1),lx,ly-1,lz-1); q=gf(self.map(n+sz+1),lx-1,ly-1,lz-1)
        fade=lambda v: f32(f32(f32(v*v)*v) * f32(f32(v*f32(v*6-15)) + 10))
        lerp=lambda dd,s,ee: f32(s+f32(dd*f32(ee-s)))
        r=fade(lx); s=fade(ly); t=fade(lz)
        x0=lerp(r,d,e); x1=lerp(r,f,g); x2=lerp(r,h,o); x3=lerp(r,p,q)
        y0=lerp(s,x0,x1); y1=lerp(s,x2,x3)
        return lerp(t,y0,y1)

# NormalNoise（DoublePerlinNoiseSampler）采样
def octave_sample_double(pns, x, y, z, lacunarity, persistence, amps):
    d = 0.0; e = lacunarity; f = persistence
    for i, pn in enumerate(pns):
        g = pn.sample_double(maintainPrecision(x*e), maintainPrecision(y*e), maintainPrecision(z*e))
        d += amps[i] * g * f
        e *= 2.0; f /= 2.0
    return d
def octave_sample_f32(pns, x, y, z, lacunarity, persistence, amps):
    d = 0.0; e = lacunarity; f = persistence
    for i, pn in enumerate(pns):
        g = pn.sample_split_f32(maintainPrecision(x*e), maintainPrecision(y*e), maintainPrecision(z*e))
        d += amps[i] * g * f
        e *= 2.0; f /= 2.0
    return d

if __name__ == '__main__':
    # continentalness 参数：firstOctave=-9, amplitudes=[1,1,2,2,2,1,1,1,1]
    amps = [1.0,1.0,2.0,2.0,2.0,1.0,1.0,1.0,1.0]
    firstOctave = -9
    n = len(amps)
    lacunarity = 2.0 ** (-firstOctave)   # 512
    persistence = (2.0**(n-1)) / (2.0**n - 1.0)  # 256/511
    nonz = [i for i,a in enumerate(amps) if a != 0.0]
    j, k = min(nonz), max(nonz)
    create_amp = 0.1 * (1.0 + 1.0/(k-j+1))
    amplitude = 0.16666666666666666 / create_amp
    DOMAIN = 1.0181268882175227
    # first + second sampler（各 9 个 Perlin，伪随机 seed）
    first = [Perlin(1000+i) for i in range(n)]
    second = [Perlin(2000+i) for i in range(n)]

    print(f"continentalness: firstOctave={firstOctave} n={n} lacunarity={lacunarity:.6f} persistence={persistence:.6f} amplitude={amplitude:.9f}")
    print("=== NormalNoise double拆分+float vs 纯double（远坐标）===")
    maxd = 0.0; sumd = 0.0; cnt = 0
    for t in range(500):
        x = (30000000 + (t % 10) * 0.13) * 0.25   # pos.x * xz_scale
        y = (64 + (t % 7) * 0.37) * 0.0           # y_scale=0
        z = (30000000 + (t % 13) * 0.11) * 0.25
        # double 全链路
        d1 = octave_sample_double(first, x, y, z, lacunarity, persistence, amps)
        d2 = octave_sample_double(second, x*DOMAIN, y*DOMAIN, z*DOMAIN, lacunarity, persistence, amps)
        base = (d1 + d2) * amplitude
        # double 拆分 + float
        s1 = octave_sample_f32(first, x, y, z, lacunarity, persistence, amps)
        s2 = octave_sample_f32(second, x*DOMAIN, y*DOMAIN, z*DOMAIN, lacunarity, persistence, amps)
        sp = (s1 + s2) * amplitude
        diff = abs(sp - base)
        if diff > maxd: maxd = diff
        sumd += diff; cnt += 1
    print(f"N={cnt} maxDiff={maxd:.3e} avgDiff={sumd/cnt:.3e}")
