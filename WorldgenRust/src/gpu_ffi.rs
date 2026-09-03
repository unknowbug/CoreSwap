// gpu_ffi.rs — lossless-accel 路线② P2（260903-05）：GPU 密度源（WG_GPU_DENSITY 门控，默认关）。
// 通道：LoadLibrary(gpu_ffi.dll) → C++ GpuDensityEngine（Vulkan compute，shader 内含 interp_N，
// 每点输出 = 生产级 per-block finalDensity，f32——与 C++ fillOneChunkCore GPU 分支同语义，
// e2e 3.128e-07 已验证，见 worldgen_api.cpp:841 与 .investigations/lossless-accel/route2-260903-05.md）。
// 语义红线（lossless）：不做「角点 + Rust 侧插值合并」——combine 外层非线性，插值合并 ≠ Java 语义；
// 本实现 = 逐块批量（对齐 C++，4096/批），慢于 CPU 路线（~0.49s/chunk vs ~10ms），
// 交付目标是「语义无损 + 可切换通路」，性能靠后续 dispatch 资源复用/批量合并（judge 260903-05 低垂果实）。
// 并发（judge 260903-05 已核）：C++ fill 全程持 fillMtx + fence 同步阻塞 = 完全串行，
// 多线程共享 handle 无 GPU 并行收益 → 生产接线按 GPU 提交单线程化设计，Rust 侧 Mutex 仅保正确性。
// handle 级缓存：create ~75s（Vulkan init + pipeline 编译），进程级按 seed 缓存（NOT 每 chunk / 每 handle 付）。
#[cfg(windows)]
use std::ffi::c_void;

#[cfg(windows)]
const GPU_BATCH: usize = 4096; // 对齐 C++ fillOneChunkCore GPU_BATCH（buffer = n*splitTotal*4B，4096 点 ≈ 142MB VRAM 上限约束）

#[cfg(windows)]
mod ffi {
    use std::ffi::c_void;
    pub type Handle = *mut c_void;
    #[allow(non_snake_case)]
    pub struct Ffi {
        pub create: unsafe extern "system" fn(u64, *const u8) -> Handle,
        pub create_ex: unsafe extern "system" fn(u64, *const u8, i32) -> Handle,
        pub fill: unsafe extern "system" fn(Handle, *const i32, i32, *mut f32),
        pub destroy: unsafe extern "system" fn(Handle),
        pub last_error: unsafe extern "system" fn() -> *const u8,
    }
    pub fn load() -> Option<Ffi> {
        let dll = b"gpu_ffi.dll\0";
        let h = unsafe { windows_sys_load(dll.as_ptr()) }?;
        unsafe fn proc_(h: isize, name: &[u8]) -> isize {
            unsafe { windows_sys_proc(h, name.as_ptr()) }
        }
        let create = unsafe { proc_(h, b"gpu_ffi_create\0") };
        let create_ex = unsafe { proc_(h, b"gpu_ffi_create_ex\0") };
        let fill = unsafe { proc_(h, b"gpu_ffi_fill\0") };
        let destroy = unsafe { proc_(h, b"gpu_ffi_destroy\0") };
        let last_error = unsafe { proc_(h, b"gpu_ffi_last_error\0") };
        if create == 0 || fill == 0 || destroy == 0 || last_error == 0 || create_ex == 0 {
            eprintln!("[WG_GPU] gpu_ffi.dll missing exports (create_ex={} — 需 build.ps1 -Ffi 重出 dll)", create_ex != 0);
            return None;
        }
        Some(Ffi {
            create: unsafe { std::mem::transmute(create) },
            create_ex: unsafe { std::mem::transmute(create_ex) },
            fill: unsafe { std::mem::transmute(fill) },
            destroy: unsafe { std::mem::transmute(destroy) },
            last_error: unsafe { std::mem::transmute(last_error) },
        })
    }
    unsafe extern "system" {
        fn LoadLibraryA(name: *const u8) -> isize;
        fn GetProcAddress(h: isize, name: *const u8) -> isize;
    }
    unsafe fn windows_sys_load(name: *const u8) -> Option<isize> {
        let h = LoadLibraryA(name);
        if h == 0 { None } else { Some(h) }
    }
    unsafe fn windows_sys_proc(h: isize, name: *const u8) -> isize { GetProcAddress(h, name) }
}

// 进程级 handle 缓存（seed → handle）。create ~75s 不可重复付；
// seed 变化时销毁旧 handle 重建（当前单世界生产 = 单 seed，缓存命中恒真）。
// Handle 非线程安全（C++ fill 内部 fillMtx 串行化 GPU 访问，见文件头并发结论）——
// 包装新类型实现 Send/Sync，正确性由 get_handle 的 Mutex + C++ fillMtx 双层保证。
#[cfg(windows)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpuKind { Final, Channels }
#[cfg(windows)]
#[derive(Clone, Copy)]
struct GpuHandle(ffi::Handle);
#[cfg(windows)]
unsafe impl Send for GpuHandle {}
#[cfg(windows)]
static GPU_HANDLE: std::sync::Mutex<Option<(i64, GpuKind, GpuHandle)>> = std::sync::Mutex::new(None);

/// spv 路径：优先 WG_GPU_SPV / WG_GPU_SPV_CHANNELS env，否则 wgDir 相对约定（对齐 C++ worldgen_api.cpp）。
#[cfg(windows)]
fn spv_path(wg_dir: &str, file: &str) -> String {
    let env_key = if file == "final_density.spv" { "WG_GPU_SPV" } else { "WG_GPU_SPV_CHANNELS" };
    if let Ok(p) = std::env::var(env_key) { return p; }
    let rel = format!("{}/../../cpp/worldgen/gpu-assets/{}", wg_dir, file);
    if std::path::Path::new(&rel).exists() { return rel; }
    format!("{}/../../gpu-assets/{}", wg_dir, file)
}

/// 获取（或创建）seed+kind 对应的 GPU handle。失败返回 None 并打印 last_error（调用方 graceful fallback）。
/// Final 与 Channels 为独立 engine 实例（各自 create ~75s；judge C2「同 engine 双 pipeline」优化后置）。
#[cfg(windows)]
fn get_handle(kind: GpuKind, seed: i64, wg_dir: &str) -> Option<ffi::Handle> {
    let mut g = GPU_HANDLE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((s, k, gh)) = g.as_ref() {
        if *s == seed && *k == kind { return Some(gh.0); }
        // seed/kind 变化：销毁旧 handle
        if let Some(f) = ffi::load() { unsafe { (f.destroy)(gh.0); } }
        *g = None;
    }
    let f = ffi::load()?;
    let (spv, ops) = match kind {
        GpuKind::Final => (spv_path(wg_dir, "final_density.spv"), 1),
        // X2 v3：5 个 per-channel spv（';' 分隔）→ C++ 引擎多 pipeline，planar 输出 out[k*n+i]
        GpuKind::Channels => {
            let paths: Vec<String> = (0..NCH).map(|k| spv_path(wg_dir, &format!("final_density_ch{}.spv", k))).collect();
            (paths.join(";"), NCH)
        }
    };
    let cpath = format!("{}\0", spv);
    let h = if ops == 1 { unsafe { (f.create)(seed as u64, cpath.as_ptr()) } }
                   else { unsafe { (f.create_ex)(seed as u64, cpath.as_ptr(), ops) } };
    if h.is_null() {
        let err = unsafe { (f.last_error)() };
        let msg = if err.is_null() { String::from("unknown") } else {
            unsafe { std::ffi::CStr::from_ptr(err as *const std::os::raw::c_char).to_string_lossy().into_owned() }
        };
        eprintln!("[WG_GPU] create failed ({:?}): {} — CPU fallback", kind, msg);
        return None;
    }
    eprintln!("[WG_GPU] engine ready ({:?}, seed={}, spv={}, outPerSample={})", kind, seed, spv, ops);
    *g = Some((seed, kind, GpuHandle(h)));
    Some(h)
}

/// GPU 密度源：per-chunk 批量（WG_GPU_DENSITY 门控，默认关；零退化铁律）。
/// 实现 DensitySource chunk 路径：sample_chunk 一次批量算整 chunk 逐块 finalDensity（f32→f64），
/// sample_interp = 索引直读（index: lx + lz*16 + ly*256，与 ChunkData 同布局）。
pub struct GpuDensity {
    #[cfg(windows)]
    seed: i64,
    #[cfg(windows)]
    wg_dir: String,
    #[cfg(windows)]
    min_y: i32,
    #[cfg(not(windows))]
    _priv: (),
}

impl GpuDensity {
    /// 构造即验证通道可用（LoadLibrary + create 一次付 ~75s）；
    /// 失败返回 None（worldgen_handle 回退 transpiler/macro，graceful fallback）。
    pub fn new(seed: i64, wg_dir: &str, min_y: i32) -> Option<Self> {
        #[cfg(windows)]
        {
            get_handle(GpuKind::Final, seed, wg_dir)?;
            Some(GpuDensity { seed, wg_dir: wg_dir.to_string(), min_y })
        }
        #[cfg(not(windows))]
        {
            let _ = (seed, wg_dir, min_y);
            eprintln!("[WG_GPU_DENSITY] unsupported platform — CPU fallback");
            None
        }
    }
}

/// 整 chunk 逐块 finalDensity 批量（对齐 C++ fillOneChunkCore GPU 分支：4096/批，异常 chunk 级回退）。
/// 返回 None = GPU fill 失败（调用方应回退 CPU 路径重算本 chunk）。
#[cfg(windows)]
pub fn fill_chunk_gpu(dense: &GpuDensity, cx: i32, cz: i32, min_y: i32, noise_height: i32) -> Option<Vec<f64>> {
    let total = (noise_height as usize) * 256;
    let h = get_handle(GpuKind::Final, dense.seed, &dense.wg_dir)?;
    let f = ffi::load()?;
    let mut out = vec![0.0f64; total];
    let mut coords = vec![0i32; GPU_BATCH * 3];
    let mut gout = vec![0f32; GPU_BATCH];
    let mut base = 0usize;
    while base < total {
        let cnt = GPU_BATCH.min(total - base);
        for k in 0..cnt {
            let pi = base + k;
            let by = pi / 256;
            let rem = pi % 256;
            let bz = rem / 16;
            let bx = rem % 16;
            coords[k * 3 + 0] = cx * 16 + bx as i32;
            coords[k * 3 + 1] = min_y + by as i32;
            coords[k * 3 + 2] = cz * 16 + bz as i32;
        }
        let ok = unsafe {
            (f.fill)(h, coords.as_ptr(), cnt as i32, gout.as_mut_ptr());
            let e = (f.last_error)();
            e.is_null() || *e == 0 // last_error 线程局部，fill 成功时为空串
        };
        if !ok {
            eprintln!("[WG_GPU_DENSITY] fill failed at base={} — chunk fallback", base);
            return None;
        }
        for k in 0..cnt {
            out[base + k] = gout[k] as f64;
        }
        base += cnt;
    }
    Some(out)
}

// ChunkDensity 布局：slices = [lx + lz*16 + ly*256]（ly 相对 min_y，噪声高度域）。
impl crate::terrain::ChunkDensitySampler for GpuDensity {
    fn sample_interp(&self, slices: &[f64], pos: &crate::density::NoisePos) -> f64 {
        let lx = pos.x.rem_euclid(16) as usize;
        let lz = pos.z.rem_euclid(16) as usize;
        let ly = (pos.y - self.min_y) as usize;
        let idx = lx + lz * 16 + ly * 256;
        slices.get(idx).copied().unwrap_or(0.0)
    }
}
impl crate::terrain::DensitySource<GpuDensity> for GpuDensity {
    fn sample(&self, pos: &crate::density::NoisePos) -> f64 {
        // 逐点回退路径（sample_chunk 恒 Some，正常不走到；诊断用单点 fill）
        #[cfg(windows)]
        {
            if let (Some(h), Some(f)) = (get_handle(GpuKind::Final, self.seed, &self.wg_dir), ffi::load()) {
                let c = [pos.x, pos.y, pos.z];
                let mut o = [0f32; 1];
                unsafe { (f.fill)(h, c.as_ptr(), 1, o.as_mut_ptr()); }
                return o[0] as f64;
            }
        }
        0.0
    }
    fn sample_chunk(&self, cx: i32, cz: i32, min_y: i32, height: i32) -> Option<crate::terrain::ChunkDensity<'_, GpuDensity>> {
        #[cfg(windows)]
        {
            let slices = fill_chunk_gpu(self, cx, cz, min_y, height)?;
            Some(crate::terrain::ChunkDensity { sampler: self, slices })
        }
        #[cfg(not(windows))]
        {
            let _ = (cx, cz, min_y, height);
            None
        }
    }
}

// ---- X2（260903-05）：GpuChannelDensity —— 5 channels @ cell corners（WG_GPU_CHANNELS 门控，默认关）----
// 语义（judge 260903-05 B 有条件批准，逐通道对拍 + 计数断言封堵）：
//   interp_k 在查询点为 cell min-corner（gx%4==0 ∧ gy%8==0 ∧ gz%4==0）时 fx=fy=fz=0 →
//   返回 delegate 在该点精确值 = Interpolated 节点 channel 值（= Java inner 角点精确采样）。
//   角点 channels（GPU，f32）→ Rust trilerp（f64）+ compute_final_density combine（外层逐块）
//   = 与 TranspilerDensity 完全同构（后者已 diff0），仅角点采样源换 GPU。
// ⚠️ 通道序：shader interp 序 = Python 遍历序（channels_map.json），Rust macrolize 序独立——
//   两套实现无构造保证，对拍探针（bin-diag/gpu_channel_probe）逐通道核验后方可生产启用。
#[cfg(windows)]
pub const NCH: i32 = 5; // final_density 5 channels（1 BlendDensity + 4 noodle）；channels_map.json 断言同值

pub struct GpuChannelDensity {
    #[cfg(windows)]
    seed: i64,
    #[cfg(windows)]
    wg_dir: String,
    #[cfg(windows)]
    min_y: i32,
    #[cfg(windows)]
    noise_height: i32,
    // CPU fallback（GPU fill 失败时 chunk 级/逐点回退；与 GPU 路径同语义——TranspilerDensity 已 diff0，
    // 角点 channels + trilerp + combine 逐位同构，见本文件 X2 头注）
    #[cfg(windows)]
    fallback: crate::terrain::TranspilerDensity,
    #[cfg(not(windows))]
    _priv: (),
}

impl GpuChannelDensity {
    /// 构造即 create（~75s 一次付，独立于 Final engine 实例）。失败返回 None（graceful fallback）。
    pub fn new(seed: i64, wg_dir: &str, min_y: i32, noise_height: i32, fallback: crate::terrain::TranspilerDensity) -> Option<Self> {
        #[cfg(windows)]
        {
            get_handle(GpuKind::Channels, seed, wg_dir)?;
            Some(GpuChannelDensity { seed, wg_dir: wg_dir.to_string(), min_y, noise_height, fallback })
        }
        #[cfg(not(windows))]
        {
            let _ = (seed, wg_dir, min_y, noise_height, fallback);
            None
        }
    }

    /// 探针/诊断：CPU 侧同布局 slices（fallback transpiler，已 diff0）
    pub fn cpu_slices(&self, cx: i32, cz: i32) -> Vec<f64> { self.fallback.build_slices_for(cx, cz) }
    /// 探针/诊断：CPU 侧逐点采样（fallback；DensitySource::sample 走 thread_local slice 缓存路径）
    pub fn cpu_sample(&self, pos: &crate::density::NoisePos) -> f64 {
        <crate::terrain::TranspilerDensity as crate::terrain::DensitySource<crate::terrain::TranspilerDensity>>::sample(&self.fallback, pos)
    }

    /// 整 chunk cell corners（5×49×5 = 1225 点）× 5 channels 一次批量（< 4096 单批，split 上传 ≈42MB）。
    /// slices 布局与 TranspilerDensity::build_slices 逐位同构：((iy*gz+iz)*gx+ix)*5+ch。
    #[cfg(windows)]
    fn build_slices_gpu(&self, cx: i32, cz: i32) -> Option<Vec<f64>> {
        let gx = 5usize; let gz = 5usize; let gy = (self.noise_height / 8 + 1) as usize;
        let n = gx * gy * gz; // 1225（overworld）
        let h = get_handle(GpuKind::Channels, self.seed, &self.wg_dir)?;
        let f = ffi::load()?;
        let mut coords = vec![0i32; n * 3];
        let mut out = vec![0f32; n * NCH as usize];
        let mut i = 0usize;
        for ix in 0..gx { for iz in 0..gz { for iy in 0..gy {
            coords[i * 3 + 0] = cx * 16 + ix as i32 * 4;
            coords[i * 3 + 1] = self.min_y + iy as i32 * 8;
            coords[i * 3 + 2] = cz * 16 + iz as i32 * 4;
            i += 1;
        }}}
        let ok = unsafe {
            (f.fill)(h, coords.as_ptr(), n as i32, out.as_mut_ptr());
            let e = (f.last_error)();
            e.is_null() || *e == 0
        };
        if !ok { eprintln!("[WG_GPU_CHANNELS] corner fill failed — chunk fallback"); return None; }
        // GPU fill planar 输出 out[k*n + i]（k=channel, i=corner 索引）→ 转 slices
        // 布局 ((iy*gz+iz)*gx+ix)*5+ch（corner 索引序 = ix 外/iz 中/iy 内，与 fill 坐标序一致）
        let mut slices = vec![0.0f64; n * NCH as usize];
        let mut i = 0usize;
        for ix in 0..gx { for iz in 0..gz { for iy in 0..gy {
            let dst = ((iy * gz + iz) * gx + ix) * NCH as usize;
            for ch in 0..NCH as usize {
                slices[dst + ch] = out[ch * n + i] as f64;
            }
            i += 1;
        }}}
        Some(slices)
    }
}

impl crate::terrain::ChunkDensitySampler for GpuChannelDensity {
    fn sample_interp(&self, slices: &[f64], pos: &crate::density::NoisePos) -> f64 {
        // 与 TranspilerDensity::sample_interp_impl 同构：trilerp 5 channels + compute_final_density
        let gx = 5i32; let gz = 5i32; let gy = (self.noise_height / 8 + 1) as i32;
        let chunk_x = pos.x.div_euclid(16); let chunk_z = pos.z.div_euclid(16);
        let gxx = pos.x - chunk_x * 16; let gzz = pos.z - chunk_z * 16; let gyy = pos.y - self.min_y;
        let mut cx = gxx / 4; let mut cy = gyy / 8; let mut cz = gzz / 4;
        cx = cx.clamp(0, gx - 2); cy = cy.clamp(0, gy - 2); cz = cz.clamp(0, gz - 2);
        let fx = (gxx % 4) as f64 / 4.0; let fy = (gyy % 8) as f64 / 8.0; let fz = (gzz % 4) as f64 / 4.0;
        let at = |dx: i32, dy: i32, dz: i32, ch: usize| -> f64 {
            let cell_idx = ((cy + dy) * gz + (cz + dz)) * gx + (cx + dx);
            slices[cell_idx as usize * NCH as usize + ch]
        };
        let mut interp = [0.0f64; 8];
        for ch in 0..NCH as usize {
            let d000 = at(0, 0, 0, ch); let d100 = at(1, 0, 0, ch);
            let d010 = at(0, 1, 0, ch); let d110 = at(1, 1, 0, ch);
            let d001 = at(0, 0, 1, ch); let d101 = at(1, 0, 1, ch);
            let d011 = at(0, 1, 1, ch); let d111 = at(1, 1, 1, ch);
            let d00 = d000 + (d100 - d000) * fx; let d10 = d010 + (d110 - d010) * fx;
            let d01 = d001 + (d101 - d001) * fx; let d11 = d011 + (d111 - d011) * fx;
            let d0 = d00 + (d10 - d00) * fy; let d1 = d01 + (d11 - d01) * fy;
            interp[ch] = d0 + (d1 - d0) * fz;
        }
        crate::generated_density::compute_final_density(&self.fallback.noises(), &interp[..NCH as usize], pos.x as f64, pos.y as f64, pos.z as f64)
    }
}
impl crate::terrain::DensitySource<GpuChannelDensity> for GpuChannelDensity {
    fn sample(&self, pos: &crate::density::NoisePos) -> f64 {
        // 逐点回退（GPU fill 失败时 fill_chunk 走此路径）= CPU transpiler 同语义
        self.fallback.sample(pos)
    }
    fn sample_chunk(&self, cx: i32, cz: i32, min_y: i32, height: i32) -> Option<crate::terrain::ChunkDensity<'_, GpuChannelDensity>> {
        #[cfg(windows)]
        {
            match self.build_slices_gpu(cx, cz) {
                Some(slices) => Some(crate::terrain::ChunkDensity { sampler: self, slices }),
                None => {
                    // GPU chunk 失败 → CPU fallback slices（同布局同语义，chunk 级优雅回退）
                    let slices = self.fallback.build_slices_for(cx, cz);
                    Some(crate::terrain::ChunkDensity { sampler: self, slices })
                }
            }
        }
        #[cfg(not(windows))]
        {
            let _ = (cx, cz, min_y, height);
            None
        }
    }
}
