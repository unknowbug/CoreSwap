// macro_sampler_probe.rs — 验证 multi-channel 宏观采样器（DensityMacroSampler）正确性与性能。
// 结构：macrolize final_density → channels + combine；对 chunk 构建 cell corners slices，块级 trilerp + combine。
// 正确性：对比 原始 final_density.sample（应为同一插值语义）。
// 性能：对比逐点采样。
use std::sync::Arc;
use std::time::Instant;
use WorldgenRust::density::{DensityFunction, NoisePos, macrolize_channels};
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::json::parse;

// multi-channel 宏观采样器（对齐 SteelMC NoiseChunk 语义，标量版先不做 SIMD）
struct DensityMacroSampler {
    channels: Vec<Arc<DensityFunction>>,
    combine: DensityFunction,
    min_y: i32, height: i32,
    cell_w: i32, cell_h: i32,
    gx: usize, gy: usize, gz: usize,
}
impl DensityMacroSampler {
    fn new(channels: Vec<Arc<DensityFunction>>, combine: DensityFunction, min_y: i32, height: i32) -> Self {
        Self { channels, combine, min_y, height, cell_w: 4, cell_h: 8,
            gx: (16/4+1) as usize, gy: (height/8+1) as usize, gz: (16/4+1) as usize }
    }
    // 构建 chunk 的 cell corners 采样（slices[cx]），存 [cx][cz][cy][ch]
    fn build_slices(&self, cx: i32, cz: i32) -> Vec<f64> {
        let n = self.gx * self.gz * self.gy * self.channels.len(); // 简化为 3D corners * channels
        let mut slices = vec![0.0f64; self.gx * self.gy * self.gz * self.channels.len()];
        for ix in 0..self.gx {
            for iz in 0..self.gz {
                for iy in 0..self.gy {
                    let px = cx*16 + ix as i32 * self.cell_w;
                    let py = self.min_y + iy as i32 * self.cell_h;
                    let pz = cz*16 + iz as i32 * self.cell_w;
                    let pos = NoisePos { x: px, y: py, z: pz };
                    for ch in 0..self.channels.len() {
                        slices[((iy*self.gz + iz)*self.gx + ix)*self.channels.len() + ch]
                            = self.channels[ch].sample(&pos);
                    }
                }
            }
        }
        slices
    }
    // 块级 trilerp + combine
    fn sample(&self, slices: &[f64], x: i32, y: i32, z: i32) -> f64 {
        let gx = self.gx as i32; let gy = self.gy as i32; let gz = self.gz as i32;
        let chunk_x = x.div_euclid(16); let chunk_z = z.div_euclid(16);
        let gxx = x - chunk_x*16; let gzz = z - chunk_z*16; let gyy = y - self.min_y;
        let mut cx = gxx / self.cell_w; let mut cy = gyy / self.cell_h; let mut cz = gzz / self.cell_w;
        // clamp 边界
        cx = cx.clamp(0, gx-2); cy = cy.clamp(0, gy-2); cz = cz.clamp(0, gz-2);
        let fx = (gxx % self.cell_w) as f64 / self.cell_w as f64;
        let fy = (gyy % self.cell_h) as f64 / self.cell_h as f64;
        let fz = (gzz % self.cell_w) as f64 / self.cell_w as f64;
        let nch = self.channels.len();
        let at = |dx: i32, dy: i32, dz: i32, ch: usize| -> f64 {
            let cell_idx = ((cy+dy)*gz + (cz+dz))*gx + (cx+dx); // i32
            let idx = cell_idx as usize * nch + ch;
            slices[idx]
        };
        let mut interp = vec![0.0f64; nch];
        for ch in 0..nch {
            let d000=at(0,0,0,ch); let d100=at(1,0,0,ch); let d010=at(0,1,0,ch); let d110=at(1,1,0,ch);
            let d001=at(0,0,1,ch); let d101=at(1,0,1,ch); let d011=at(0,1,1,ch); let d111=at(1,1,1,ch);
            let d00=d000+(d100-d000)*fx; let d10=d010+(d110-d010)*fx;
            let d01=d001+(d101-d001)*fx; let d11=d011+(d111-d011)*fx;
            let d0=d00+(d10-d00)*fy; let d1=d01+(d11-d01)*fy;
            interp[ch] = d0 + (d1 - d0)*fz;
        }
        self.combine.sample_combine(&NoisePos { x, y, z }, &interp)
    }
}

fn main() {
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let seed: i64 = -8248318472910187742;
    let mut db = DensityBuilder::new(seed as u64, -64, 384);
    db.load_noise_params_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    let df_dir = format!("{}/data/minecraft/worldgen/density_function/overworld", wg_dir);
    let df_dir2 = df_dir.clone();
    db.set_df_ns("overworld");
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        std::fs::read_to_string(&format!("{}/{}.json", df_dir2, name)).unwrap()
    }));
    let settings = parse(&std::fs::read_to_string(format!("{}/data/minecraft/worldgen/noise_settings/overworld.json", wg_dir)).unwrap()).unwrap();
    let router = settings.get("noise_router").unwrap();
    let tree = db.build_node(router.get("final_density").unwrap()).ok().unwrap();
    let (channels, combine) = macrolize_channels(&tree);
    let sampler = DensityMacroSampler::new(channels, combine, -64, 384);

    let cx = -288; let cz = -256;
    // 预热（构建 slices）
    println!("构建 slices...");
    let t0 = Instant::now();
    let slices = sampler.build_slices(cx, cz);
    println!("slices 构建: {:.2}ms/chunk", t0.elapsed().as_secs_f64()*1e3);

    // 正确性：对比几个点 原始 final_density vs multi-channel
    let orig = &tree;
    let mut diff_total = 0.0f64; let mut n = 0;
    for y in [4i32, 64, 128, 200, 260, 300] {
        for z in [4i32, 8, 12] { for x in [4i32, 8, 12] {
            let wx = cx*16+x; let wz = cz*16+z;
            let a = orig.sample(&NoisePos{x:wx,y,z:wz});
            let b = sampler.sample(&slices, wx, y, wz);
            diff_total += (a-b).abs(); n += 1;
        }}
    }
    println!("multi-channel vs 原始 final_density：平均差异 {:.6} (n={})", diff_total/n as f64, n);

    // 性能：对 chunk 逐点采样（对比逐点原始）
    let t1 = Instant::now();
    for _r in 0..5 { for y in -64..320 { for z in 0..16 { for x in 0..16 { let _ = orig.sample(&NoisePos{x:cx*16+x,y,z:cz*16+z}); } } } }
    let dt_orig = t1.elapsed().as_secs_f64()/5.0*1e3;
    let t2 = Instant::now();
    for _r in 0..5 { for y in -64..320 { for z in 0..16 { for x in 0..16 { let _ = sampler.sample(&slices, cx*16+x, y, cz*16+z); } } } }
    let dt_mc = t2.elapsed().as_secs_f64()/5.0*1e3;
    println!("原始逐点采样: {:.2}ms/chunk; multi-channel(含slices复用): {:.2}ms/chunk", dt_orig, dt_mc);
    println!("(slices每chunk构建一次 + 块级trilerp); 若块级trilerp ~= 原始, 则savings = slices构建成本可摊薄)");
}
