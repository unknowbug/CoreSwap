# probe-delivery — 残留 1830 两侧判别探针代码草稿（worker 交付，260904，未编译验证）

> 状态：**draft（未编译验证）**。Rust 侧 = patch 片段（本文档 §2，主会话应用 + `build.ps1` 编译）；Java 侧 = 新文件已落盘 + 注册已改（§3）。
> 依据：`.investigations/residual-1830/scout-aquifer-map.md` §2 D1-D5 / §3 探针设计；判别目标 = **单向漂移 vs 边界震荡**（§2 关键结构性观察）。
> 纪律：subagent 沙箱无 shell，本交付只产出代码/patch；编译、gradle、gradle 参数→JVM 映射由主会话处理。

## 0. 判别原理（一行）

同一点在两侧各打一行 `AQDUMP ...`（字段名/顺序完全一致），逐字段 diff：
- **o/p/q/r/s/t（整数，纯确定性）任一不同** → D4（blob 邻域/split 链差）实锤；
- **est 不同** → D2；**erosion/depth/gate 不同** → D3；
- **floodedRaw/flooded/spreadRaw 系统性单向偏**（均值同号、量级稳定）→ 单向漂移（D1 主判别）；
- **floodedRaw/flooded/e 在 0/阈值附近 ±0.0x 级随机符号差** → 边界震荡（D1/D2 输入微差在边界双向翻转）；
- **cdE/e/cdG/g/cdH/h + density** → D5（density 输入差）旁证（density 本身在 dump 的 density 字段）。

## 1. 行格式（两侧唯一契约，30 字段 + decision）

```
AQDUMP x=<i> y=<i> z=<i> density=<f6> floodedRaw=<f6|na> flooded=<f6|na> spreadRaw=<f6|na> spreadRnd=<i|na> spreadBase=<i|na> erosion=<f6|na> depth=<f6|na> gate=<0/1|na> barrier=<f6|na> bl=<0/1|na> f=<f6|na> est=<i|MAX|early(k>o)|na> fl2=(y,block) fl3=(y,block) fl4=(y,block) opq=(o,p,q) r=(bx,by,bz|y,block) s=(...) t=(...) d=<f6> fq=<f6> gpq=<f6> cdE=<f6|na> e=<f6|na> cdG=<f6|na> g=<f6|na> cdH=<f6|na> h=<f6|na> decision=<...>
```

| 字段 | 语义 | 捕获点 |
|---|---|---|
| x/y/z/density | apply 输入 | 入口 |
| floodedRaw / flooded | fluidLevelFloodedness 采样 **clamp 前原值 / clamp±1 后**（D1 判别核心） | get_fluid_block_y 首链（first-wins） |
| spreadRaw / spreadRnd / spreadBase | fluidLevelSpread 采样原值 / roundDownToMultiple(·,3) / l*40+20 | get_noise_based_fluid_level |
| erosion / depth / gate | method_43718 门输入与命中布尔（depth 短路未采样=na） | get_fluid_block_y |
| barrier | barrierNoise 首次采样值（|q|≤2 才采） | calculate_density |
| bl / f / est | 13 邻域 bl 标记 / 淹没权重 clampedMap / est（**首链**；MAX=扫不到；early(k>o)=中心列早退；na=fl2 缓存命中链未跑） | get_fluid_level/get_fluid_block_y |
| fl2/fl3/fl4 | getWaterLevel 结果 (y,block)（block: 0=air 1=water 2=lava） | apply |
| opq | 最近/次近/第三 blob 距离平方（MAX=空位） | apply 三元组维护后 |
| r/s/t | blob pack 坐标解包 (bx,by,bz) + 对应 FluidLevel (y,block)（**D4 判别核心**） | apply |
| d / fq / gpq | maxDistance(o,p) / maxDistance(o,q) / maxDistance(p,q) | apply |
| cdE/e, cdG/g, cdH/h | calculate_density 结果与 d(·f·)加权积（未计算分支=na） | apply |
| decision | 见 §1.1 | 出口 |

### 1.1 decision 映射（Rust 串 ↔ Java vanilla 真实返回）

| Rust decision | Java decision | 语义 |
|---|---|---|
| `Rock:density>0` | `null` | density>0 早退（两侧都归 stone） |
| `BLOCK:2` | `BLOCK:2` | 默认 lava（block_y<-54） |
| `BLOCK:<bs>` (d<=0) | `BLOCK:0/1/2` | 距离场内直接 bs |
| `BLOCK:1` | `BLOCK:1` | water-over-lava 特判 |
| `null:density+e>0` / `null:density+g>0` / `null:density+h>0` | `null` | 三次合并检查早退（null=保持 stone） |
| `BLOCK:<bs>` (final) | `BLOCK:0/1/2` | 正常终局 |

Java decision 取 **RETURN 捕获的 vanilla 真实返回值**（非复算），Rust 取自身实际 decision——decision 本身的差异即最终判定差异。

## 2. Rust patch（`WorldgenRust/src/aquifer.rs`，主会话应用）

### Patch R1 — 顶部插入 AQDUMP 模块（anchor：`aquifer_surf_count_reset` 函数体结束之后、`#[derive(Clone, Copy)] pub struct FluidLevel` 之前）

old:
```rust
pub fn aquifer_surf_count_reset() -> [usize; 2] {
    SURF_COUNT.with(|c| { let mut c = c.borrow_mut(); let r = *c; *c = [0, 0]; r })
}

#[derive(Clone, Copy)]
pub struct FluidLevel {
```

new:
```rust
pub fn aquifer_surf_count_reset() -> [usize; 2] {
    SURF_COUNT.with(|c| { let mut c = c.borrow_mut(); let r = *c; *c = [0, 0]; r })
}

// ===== WG_AQDUMP（残留 1830 判别探针，260904 worker 草稿，未编译验证）：点文件驱动 aquifer 全链路 dump =====
// 门控：env WG_AQDUMP=<点文件路径>（行格式 `x y z`，# 注释）。进程级读一次（OnceLock，
// 仿 surface_rules.rs soul_dump_points 点文件模式）；apply 入口一次命中判断；门控关时
// points=None → hit 恒 false、note_* 首个原子 load 即返回——热路径仅多 1 次原子 load/点
//（与 WG_AQUIFER* 计数器同级，对齐诊断门控铁律）。
// 输出：stdout 单行，字段名/顺序与 Java AquiferDumpProbeMixin 逐字段一致（.investigations/residual-1830/probe-delivery.md §1 契约）。
// seed 头：aqdump_seed(seed)（worldgen_handle 每 chunk 调，OnceLock 只打一次）。
static AQDUMP_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn aqdump_points() -> Option<&'static std::collections::HashSet<(i32, i32, i32)>> {
    static DUMP: std::sync::OnceLock<Option<std::collections::HashSet<(i32, i32, i32)>>> =
        std::sync::OnceLock::new();
    DUMP.get_or_init(|| {
        let path = std::env::var("WG_AQDUMP").ok()?;
        let txt = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("[AQDUMP] cannot read {}: {}", path, e));
        let mut set = std::collections::HashSet::new();
        for line in txt.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let mut it = line.split_whitespace();
            if let (Some(a), Some(b), Some(c)) = (it.next(), it.next(), it.next()) {
                if let (Ok(x), Ok(y), Ok(z)) = (a.parse(), b.parse(), c.parse()) { set.insert((x, y, z)); }
            }
        }
        AQDUMP_ON.store(true, std::sync::atomic::Ordering::Relaxed);
        eprintln!("[AQDUMP] enabled: {} points from {}", set.len(), path);
        Some(set)
    })
    .as_ref()
}

/// seed 头（供三查）：worldgen_handle fill_chunk 入口每 chunk 调一次；仅首次实际打印。
pub fn aqdump_seed(seed: i64) {
    if !AQDUMP_ON.load(std::sync::atomic::Ordering::Relaxed) { return; }
    static ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| println!("AQDUMP seed={}", seed));
}

fn aqdump_hit(x: i32, y: i32, z: i32) -> bool {
    if !AQDUMP_ON.load(std::sync::atomic::Ordering::Relaxed) { return false; }
    aqdump_points().map_or(false, |s| s.contains(&(x, y, z)))
}

// 判别收集器：apply 入口建（点命中时）、出口统一打印——覆盖全部早退分支，避免逐分支 printf。
// 字段 None → 打 na；i32::MAX 哨兵 → 打 MAX（不误当坐标）。
struct AqDump {
    x: i32, y: i32, z: i32, density: f64,
    flooded_raw: Option<f64>, flooded: Option<f64>,
    spread_raw: Option<f64>, spread_rnd: Option<i32>, spread_base: Option<i32>,
    erosion: Option<f64>, depth: Option<f64>, gate: Option<bool>,
    barrier: Option<f64>,
    bl: Option<u8>, fw: Option<f64>, est: Option<String>,
    fl2: Option<(i32, i32)>, fl3: Option<(i32, i32)>, fl4: Option<(i32, i32)>,
    o: Option<i32>, p: Option<i32>, q: Option<i32>,
    r: Option<String>, s: Option<String>, t: Option<String>,
    d: Option<f64>, fq: Option<f64>, gpq: Option<f64>,
    cd_e: Option<f64>, e: Option<f64>, cd_g: Option<f64>, g: Option<f64>,
    cd_h: Option<f64>, h: Option<f64>,
    decision: String,
}
impl AqDump {
    fn new(x: i32, y: i32, z: i32, density: f64) -> AqDump {
        AqDump { x, y, z, density, flooded_raw: None, flooded: None, spread_raw: None, spread_rnd: None,
            spread_base: None, erosion: None, depth: None, gate: None, barrier: None, bl: None, fw: None,
            est: None, fl2: None, fl3: None, fl4: None, o: None, p: None, q: None, r: None, s: None, t: None,
            d: None, fq: None, gpq: None, cd_e: None, e: None, cd_g: None, g: None, cd_h: None, h: None,
            decision: "na".to_string() }
    }
}

thread_local! {
    static AQDUMP: std::cell::RefCell<Option<Box<AqDump>>> = std::cell::RefCell::new(None);
    static AQDUMP_SUPPRESS: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

fn aqd() -> bool {
    if !AQDUMP_ON.load(std::sync::atomic::Ordering::Relaxed) { return false; }
    // water-over-lava 检查直调 get_fluid_level 的链路值不捕获（非 fl2 首链）
    if AQDUMP_SUPPRESS.with(|s| s.get()) { return false; }
    true
}

fn note_decision(s: &str) {
    if !aqd() { return; }
    AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { d.decision = s.to_string(); } });
}
fn note_triplet(o: i32, p: i32, q: i32) {
    if !aqd() { return; }
    AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { d.o = Some(o); d.p = Some(p); d.q = Some(q); } });
}
fn note_blob(slot: u8, pos: i64, fl: &FluidLevel) {
    if !aqd() { return; }
    let txt = format!("({},{},{}|{},{})", Aquifer::unpack_x(pos), Aquifer::unpack_y(pos), Aquifer::unpack_z(pos), fl.y, fl.block);
    AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() {
        match slot { 1 => { d.fl2 = Some((fl.y, fl.block)); d.r = Some(txt); }
                     2 => { d.fl3 = Some((fl.y, fl.block)); d.s = Some(txt); }
                     _ => { d.fl4 = Some((fl.y, fl.block)); d.t = Some(txt); } }
    }});
}
fn note_d(v: f64) { if !aqd() { return; } AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { d.d = Some(v); } }); }
fn note_fq(v: f64) { if !aqd() { return; } AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { d.fq = Some(v); } }); }
fn note_gpq(v: f64) { if !aqd() { return; } AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { d.gpq = Some(v); } }); }
fn note_cde(cd: f64, e: f64) { if !aqd() { return; } AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { d.cd_e = Some(cd); d.e = Some(e); } }); }
fn note_cdg(cd: f64, g: f64) { if !aqd() { return; } AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { d.cd_g = Some(cd); d.g = Some(g); } }); }
fn note_cdh(cd: f64, h: f64) { if !aqd() { return; } AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { d.cd_h = Some(cd); d.h = Some(h); } }); }
fn note_flooded(raw: f64, clamped: f64) {
    if !aqd() { return; }
    AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { if d.flooded_raw.is_none() { d.flooded_raw = Some(raw); d.flooded = Some(clamped); } } });
}
fn note_spread(raw: f64, rnd: i32, base: i32) {
    if !aqd() { return; }
    AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { if d.spread_raw.is_none() { d.spread_raw = Some(raw); d.spread_rnd = Some(rnd); d.spread_base = Some(base); } } });
}
fn note_gate(er: f64, dp: Option<f64>, gate: bool) {
    if !aqd() { return; }
    AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { if d.gate.is_none() { d.erosion = Some(er); d.depth = dp; d.gate = Some(gate); } } });
}
fn note_chain(est: i32, bl: bool, fw: Option<f64>) {
    if !aqd() { return; }
    AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() {
        if d.est.is_none() {
            d.est = Some(if est == i32::MAX { "MAX".to_string() } else { est.to_string() });
            d.bl = Some(if bl { 1 } else { 0 });
            d.fw = fw;
        }
    }});
}
fn note_est_early() {
    if !aqd() { return; }
    AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { if d.est.is_none() { d.est = Some("early(k>o)".to_string()); } } });
}
fn note_barrier(v: f64) {
    if !aqd() { return; }
    AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().as_mut() { if d.barrier.is_none() { d.barrier = Some(v); } } });
}

fn ff(v: Option<f64>) -> String { match v { Some(v) => format!("{:.6}", v), None => "na".to_string() } }
fn fi(v: Option<i32>) -> String { match v { Some(i32::MAX) => "MAX".to_string(), Some(v) => v.to_string(), None => "na".to_string() } }
fn ffs(v: &Option<String>) -> String { v.clone().unwrap_or_else(|| "na".to_string()) }

impl AqDump {
    fn print(&self) {
        let fl = |v: &Option<(i32, i32)>| match v { Some((y, b)) => format!("({},{})", y, b), None => "na".to_string() };
        let b01 = |v: Option<bool>| match v { Some(true) => "1".to_string(), Some(false) => "0".to_string(), None => "na".to_string() };
        println!("AQDUMP x={} y={} z={} density={:.6} floodedRaw={} flooded={} spreadRaw={} spreadRnd={} spreadBase={} erosion={} depth={} gate={} barrier={} bl={} f={} est={} fl2={} fl3={} fl4={} opq=({},{},{}) r={} s={} t={} d={} fq={} gpq={} cdE={} e={} cdG={} g={} cdH={} h={} decision={}",
            self.x, self.y, self.z, self.density,
            ff(self.flooded_raw), ff(self.flooded), ff(self.spread_raw), fi(self.spread_rnd), fi(self.spread_base),
            ff(self.erosion), ff(self.depth), b01(self.gate), ff(self.barrier),
            match self.bl { Some(v) => v.to_string(), None => "na".to_string() },
            ff(self.fw), ffs(&self.est),
            fl(&self.fl2), fl(&self.fl3), fl(&self.fl4),
            fi(self.o), fi(self.p), fi(self.q),
            ffs(&self.r), ffs(&self.s), ffs(&self.t),
            ff(self.d), ff(self.fq), ff(self.gpq),
            ff(self.cd_e), ff(self.e), ff(self.cd_g), ff(self.g), ff(self.cd_h), ff(self.h),
            self.decision);
    }
}

#[derive(Clone, Copy)]
pub struct FluidLevel {
```

### Patch R2 — apply → 入口/出口包装 + apply_inner（整段替换，old = aquifer.rs L294-341 原文）

old:（现 `pub fn apply(&mut self, ...) -> i32 { ... }` 全函数，L294-341，含 `if density > 0.0 { return -1; }` 起至 `bs\n    }` 止——以仓库当前 HEAD 为准逐字取）

new:
```rust
    pub fn apply(&mut self, block_x: i32, block_y: i32, block_z: i32, density: f64) -> i32 {
        // WG_AQDUMP：入口建收集器、出口统一打印（carver apply(pos,0.0) 同路复用，deepslate→air 顺带覆盖）
        let hit = aqdump_hit(block_x, block_y, block_z);
        if hit { AQDUMP.with(|c| *c.borrow_mut() = Some(Box::new(AqDump::new(block_x, block_y, block_z, density)))); }
        let r = self.apply_inner(block_x, block_y, block_z, density);
        if hit { AQDUMP.with(|c| { if let Some(d) = c.borrow_mut().take() { d.print(); } }); }
        r
    }

    fn apply_inner(&mut self, block_x: i32, block_y: i32, block_z: i32, density: f64) -> i32 {
        if density > 0.0 { note_decision("Rock:density>0"); return -1; }
        let mut fluid_block;
        let mut fluid_y;
        if block_y < -54 { fluid_block = LAVA; fluid_y = -54; } else { fluid_block = WATER; fluid_y = 63; }
        if fluid_block == LAVA { note_decision("BLOCK:2"); return fluid_block; }

        let l = floor_div(block_x - 5, 16);
        let m = floor_div(block_y + 1, 12);
        let n = floor_div(block_z - 5, 16);
        let mut o = i32::MAX; let mut p = i32::MAX; let mut q = i32::MAX;
        let mut r: i64 = 0; let mut s: i64 = 0; let mut t: i64 = 0;
        for u in 0..=1 { for v in -1..=1 { for w in 0..=1 {
            let x = l + u; let y = m + v; let z = n + w;
            let ab = self.get_block_pos(x, y, z);
            let ad = Self::unpack_x(ab) - block_x;
            let ae = Self::unpack_y(ab) - block_y;
            let af = Self::unpack_z(ab) - block_z;
            let ag = ad*ad + ae*ae + af*af;
            if o >= ag { t = s; s = r; r = ab; q = p; p = o; o = ag; }
            else if p >= ag { t = s; s = ab; q = p; p = ag; }
            else if q >= ag { t = ab; q = ag; }
        }}}
        note_triplet(o, p, q);

        let fl2 = self.get_water_level_at(r);
        note_blob(1, r, &fl2);
        let d = Self::max_distance(o, p);
        note_d(d);
        let bs = fl2.get_block_state(block_y);
        if d <= 0.0 { note_decision(&format!("BLOCK:{}", bs)); return bs; }
        if bs == WATER {
            // water-over-lava 直调 get_fluid_level：suppress（该链非 fl2 首链，不捕获）
            AQDUMP_SUPPRESS.with(|s| s.set(true));
            let lava_check = self.get_fluid_level(block_x, block_y - 1, block_z);
            AQDUMP_SUPPRESS.with(|s| s.set(false));
            if lava_check.get_block_state(block_y - 1) == LAVA { note_decision("BLOCK:1"); return bs; }
        }

        let fl3 = self.get_water_level_at(s);
        note_blob(2, s, &fl3);
        let mut md = MutableDouble::new();
        let cd_e = self.calculate_density(block_x, block_y, block_z, &mut md, fl2, fl3);
        let e = d * cd_e;
        note_cde(cd_e, e);
        if density + e > 0.0 { note_decision("null:density+e>0"); return -1; }

        let fl4 = self.get_water_level_at(t);
        note_blob(3, t, &fl4);
        let f = Self::max_distance(o, q);
        note_fq(f);
        if f > 0.0 {
            let cd_g = self.calculate_density(block_x, block_y, block_z, &mut md, fl2, fl4);
            let g = d * f * cd_g;
            note_cdg(cd_g, g);
            if density + g > 0.0 { note_decision("null:density+g>0"); return -1; }
        }
        let g2 = Self::max_distance(p, q);
        note_gpq(g2);
        if g2 > 0.0 {
            let cd_h = self.calculate_density(block_x, block_y, block_z, &mut md, fl3, fl4);
            let h = d * g2 * cd_h;
            note_cdh(cd_h, h);
            if density + h > 0.0 { note_decision("null:density+h>0"); return -1; }
        }
        note_decision(&format!("BLOCK:{}", bs));
        bs
    }
```

（语义注：water-over-lava 从 `a && b(...)` 改为 `if a { ...b... }`——调用时机/短路语义不变。）

### Patch R3 — get_fluid_level 中心列早退补记（L428-429）

old:
```rust
            let bl2 = off[0] == 0 && off[1] == 0;
            if bl2 && k > o { return default_fl; }
```
new:
```rust
            let bl2 = off[0] == 0 && off[1] == 0;
            if bl2 && k > o { note_est_early(); return default_fl; }
```

### Patch R4 — get_fluid_block_y：门展开 + flooded/est/bl 捕获（整段替换 L444-460）

old:
```rust
    fn get_fluid_block_y(&self, block_x: i32, block_y: i32, block_z: i32, default_fl: &FluidLevel, surface_height_estimate: i32, bl: bool) -> i32 {
        let pos = NoisePos { x: block_x, y: block_y, z: block_z };
        let (mut d, mut e): (f64, f64);
        if self.erosion.sample(&pos) < -0.225f32 as f64 && self.depth.sample(&pos) > 0.9f32 as f64 {
            d = -1.0; e = -1.0;
        } else {
            let ii = surface_height_estimate + 8 - block_y;
            let f = if bl { lerp_clamp2(ii as f64, 0.0, 64.0, 1.0, 0.0) } else { 0.0 };
            let g = clamp(self.fluid_floodedness.sample(&pos), -1.0, 1.0);
            let h = map2(f, 1.0, 0.0, -0.3, 0.8);
            let kk = map2(f, 1.0, 0.0, -0.8, 0.4);
            d = g - kk; e = g - h;
        }
        if e > 0.0 { default_fl.y }
        else if d > 0.0 { self.get_noise_based_fluid_level(block_x, block_y, block_z, surface_height_estimate) }
        else { -32512 }
    }
```
new:
```rust
    fn get_fluid_block_y(&self, block_x: i32, block_y: i32, block_z: i32, default_fl: &FluidLevel, surface_height_estimate: i32, bl: bool) -> i32 {
        let pos = NoisePos { x: block_x, y: block_y, z: block_z };
        let (mut d, mut e): (f64, f64);
        // WG_AQDUMP：method_43718 门展开（保 Java && 短路语义：erosion 不满足时 depth 不采样 → depth=na）
        let er = self.erosion.sample(&pos);
        let mut dp = None;
        let gate = if er < -0.225f32 as f64 {
            let dv = self.depth.sample(&pos);
            dp = Some(dv);
            dv > 0.9f32 as f64
        } else { false };
        note_gate(er, dp, gate);
        if gate {
            d = -1.0; e = -1.0;
            note_chain(surface_height_estimate, bl, None); // 门命中：flooded/f 未采样 → na
        } else {
            let ii = surface_height_estimate + 8 - block_y;
            let f = if bl { lerp_clamp2(ii as f64, 0.0, 64.0, 1.0, 0.0) } else { 0.0 };
            let g_raw = self.fluid_floodedness.sample(&pos);
            let g = clamp(g_raw, -1.0, 1.0);
            note_flooded(g_raw, g); // clamp 前原值 + clamp 后（判别边界震荡）
            note_chain(surface_height_estimate, bl, Some(f)); // first-wins = fl2 首链
            let h = map2(f, 1.0, 0.0, -0.3, 0.8);
            let kk = map2(f, 1.0, 0.0, -0.8, 0.4);
            d = g - kk; e = g - h;
        }
        if e > 0.0 { default_fl.y }
        else if d > 0.0 { self.get_noise_based_fluid_level(block_x, block_y, block_z, surface_height_estimate) }
        else { -32512 }
    }
```

### Patch R5 — get_noise_based_fluid_level：spread 捕获（L468-469）

old:
```rust
        let d = self.fluid_spread.sample(&pos) * 10.0;
        let p = round_down_to_multiple(d, 3);
```
new:
```rust
        let raw = self.fluid_spread.sample(&pos);
        let d = raw * 10.0;
        let p = round_down_to_multiple(d, 3);
        note_spread(raw, p, n);
```
（`n = l * 40 + 20` 已在该函数上方定义。）

### Patch R6 — calculate_density：barrier 首采捕获（L395）

old:
```rust
                    let pos = NoisePos { x: block_x, y: block_y, z: block_z }; let tv = self.barrier.sample(&pos); md.v = tv; md.has = true; tv
```
new:
```rust
                    let pos = NoisePos { x: block_x, y: block_y, z: block_z }; let tv = self.barrier.sample(&pos); note_barrier(tv); md.v = tv; md.has = true; tv
```

### Patch R7 — `WorldgenRust/src/worldgen_handle.rs`：seed 头挂钩（fill_chunk 入口，`let mut aq = crate::aquifer::Aquifer::new(` 之前）

old:
```rust
        let mut aq = crate::aquifer::Aquifer::new(
```
new:
```rust
        // WG_AQDUMP：seed 头（OnceLock 只打一次；门控关时一次原子 load）
        crate::aquifer::aqdump_seed(self.seed);
        let mut aq = crate::aquifer::Aquifer::new(
```

## 3. Java 侧（已落盘）

| 文件 | 状态 |
|---|---|
| `runtime/1.20.1/java/src/main/java/wg/bench/AquiferDumpProbe.java` | 新增（driver：读点文件 → 算 chunk region → 预生成 FULL → 打 `AQDUMP seed=` 头 → 停服；仿 EstDumpProbe） |
| `runtime/1.20.1/java/src/main/java/wg/bench/mixin/AquiferDumpProbeMixin.java` | 新增（`@Mixin(AquiferSampler.Impl.class)`，apply HEAD 复算 + RETURN 捕获 vanilla decision；@Shadow 直读 vanilla 私有字段，无 CellCache 反射） |
| `runtime/1.20.1/java/src/main/resources/coreswap.mixins.json` | 已改（mixins 数组加 `"AquiferDumpProbeMixin"`） |
| `runtime/1.20.1/java/src/main/java/wg/bench/BenchMod.java` | 已改（anyProbe 列表加 `aqdump.probe`；dispatch else-if 加 `AquiferDumpProbe.run`） |

选型说明：**Mixin（@Shadow）优于纯反射**——@Shadow 把 Impl 私有 sampler 字段/waterLevels/blockPositions 直接并入 mixin 类，编译期即可见，避开 -288「CellCache 反射不可信」整类问题；仅 `FluidLevel.y`（包私有 final）一处小反射（有 getBlockState(Integer.MIN_VALUE) 技巧取 state 免反射）。

复算纪律（mixin javadoc 已写）：blockPositions/waterLevels **只读**（miss 本地重放 split/链，不写回——vanilla 随后自填同值）；est 走公开方法 `chunkNoiseSampler.estimateSurfaceHeight`（精确值缓存）；decision 以 vanilla 真实返回为准；d/e/f/bl 公式静态复刻（已证零偏离）。

## 4. 参数映射（主会话执行用）

| 侧 | 触发 | 参数 |
|---|---|---|
| Rust | env | `WG_AQDUMP=<点文件绝对路径>`（block_probe/dll 进程 env；建议同设 `WG_EST_L2=0` 消除 L2 缓存变量，或两侧都默认开——est 是精确值缓存，理论上无差） |
| Java | systemProperty | `-Daqdump.probe=1`（触发）`-Daqdump.points=<点文件>`（必需）`-Daqdump.out=<输出文件>`（可选，缺省 stdout）`-Vanilla`（建议，确保纯 vanilla） |
| gradle 映射 | 不改 build.gradle | 主会话按现有 EstDumpProbe 的 -P→-D 通道映射同样接 `aqdump.probe/aqdump.points/aqdump.out` 三个 prop 名即可 |

点文件格式（两侧同）：
```
# x y z
200 16 238
204 20 240
208 24 236
```
建议点集：residual-1830 域内 diff cell 中心各取 ~20-40 点（water→air / air→water / stone→water / water→stone 四族都覆盖）+ 少量一致点作对照。

## 5. 主会话执行手册

1. 应用 §2 patch R1-R7 → `pwsh versions/1.20.1/cpp/build.ps1`（Rust 侧 `cargo build --release` 随工程流程）。
2. Java 侧：`runtime/1.20.1/java` 下 gradle 构建（4 个改动文件都在，无需改 build.gradle）。
3. **seed 三查先行**：两侧输出头 `AQDUMP seed=` 必须都等于 `8576294172403134396`，不一致即废。
4. 运行：
   - Rust：block_probe/dll 生成流程 + `WG_AQDUMP=.tmp/aqdump/points.txt`，stdout 重定向 `.tmp/aqdump/rust.txt`（每命中点一行；carver 路径点若在点集内同样出行——decision=BLOCK:2 即 lava 默认）。
   - Java：`-Daqdump.probe=1 -Daqdump.points=.tmp/aqdump/points.txt -Daqdump.out=.tmp/aqdump/java.txt -Vanilla`，预生成后自动停服。
5. 判读（worker/主会话解读）：
   - 先比整数列 `opq/r/s/t`（+fl2/fl3/fl4）：**任一不同 → D4 实锤**（纯确定性，无浮点噪声，最廉价判据）；
   - 再比 `est/gate/erosion/depth`：不同 → D2/D3；
   - 主判据 `floodedRaw/flooded`：对每点算 `Δ=rust-java`——**符号一致、量级稳定 → 单向漂移**；**±0.0x 随机符号、集中在 ±1 clamp 与 e=0 边界附近 → 边界震荡**（结合 e/cdE 列：Δe 与 decision 翻转点重合即边界震荡实锤）；
   - `density/d/e` 差 → D5 旁证（Rust 宏观 density vs Java 网格）。
   - ⚠️ 浮点文本比较建议**解析成 f64 后比数值**（容差建议 1e-6 报告、>1e-4 判差异），不要做字符串全等——Rust `{:.6}` 与 Java `%.6f` 舍入模式（half-even vs half-up）在第 6 位可能差 1 ulp 文本。
   - Java 头两行 `[AQDUMP] points=...` 与 Rust stderr `[AQDUMP] enabled: ...` 是控制行，diff 前过滤。

## 6. 静态自检清单（未编译验证声明附件）

① **类型宽度**：Rust `i64` pack/unpack ↔ Java `long` BlockPos.asLong/unpackLong（位布局一致：x26/y12/z26）；`o/p/q` 两侧 i32/int（`i32::MAX`/`Integer.MAX_VALUE` 哨兵→打 `MAX`）；est `i32::MAX`→`MAX`；`d/e/e` 全 f64/double。Java `j / 2.0`（int→double）↔ Rust `j as f64 / 2.0` 一致；`-0.225F`/`0.9F` 两侧都走 float 提升（Rust `-0.225f32 as f64`）。
② **早退路径全覆盖**：Rust 7 个出口（density>0 / lava 默认 / d<=0 / waterOverLava / e>0 / g>0 / h>0 / final）全经统一出口 `d.print()`；Java HEAD+RETURN 两注入覆盖 vanilla 全部 return（含 density>0 HEAD 早退）。同点多次 apply（NOISE 宏观 + carver）会各打一行——按 density 字段区分（carver 恒 0.0）。
③ **门控关零开销**：Rust 每 apply 入口 1 次 `AQDUMP_ON` 原子 load（aqdump_hit 内首个 if）+ 每 note 1 次原子 load（无点集哈希）；env/文件只在 OnceLock 首次读。Java `AQ_ON` 为 compile-time-constant 风格 static final boolean，两注入首行返回。
④ **哨兵**：`i32::MAX`/`Integer.MAX_VALUE`→`MAX`；`Long.MAX_VALUE` 哨兵只在 blockPositions 读取侧判（不打印原值）；`NaN`→`na`；r/s/t 三元未占用位（Java/ vanilla 同）为 0L → 打 `(0,0,0|...)`（vanilla 语义同，两侧一致可比）。
⑤ **字段名对齐**：30 字段 + decision 两侧逐字符一致（§1 契约 = Rust print 格式串 = Java wgBuildLine append 序列）；输出均为 stdout 单行（Java 可选落文件，行内容相同）。
⑥ 其他：Java `wgFluidY` 反射失败返回 `Integer.MAX_VALUE` → 打 `MAX` 可见（不静默）；Java 线程安全（POINTS 只读 + 文件 synchronized + ThreadLocal stash）；Rust suppress 保证 water-over-lava 链不污染首链捕获，与 Java「cap 只传 fl2 链」等价。

**已知小差异（预期无影响）**：Rust `{:.6}` half-even vs Java `%.6f` half-up（§5 判读第 5 条规避）；Java 复算链的 barrier 值来自复算中首个 `|q|≤2` 采样（与 vanilla MutableDouble 缓存同值）。

## 7. 验证分层声明

本交付 = **Degraded（静态审查）**：未编译、未运行。上机后需核对：Rust patch 应用点行号漂移（以 old 片段逐字匹配为准）；Mixin `@Shadow` 对 Impl 私有字段的可见性（accessWidener 无需——@Shadow 即可）；`DimensionType.field_35479` 中间名在本 mappings 集下存在（yarn sources L415 原文引用，应可用；编译报错则替换字面量 -32512——注意保留与 source 同语义）。
