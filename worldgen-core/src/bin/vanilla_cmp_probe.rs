// vanilla_cmp_probe.rs — Rust finalDensity vs vanilla Minecraft density 直接对照（cherry 种子，0,0 区域 4x4 chunk）。
// 读 vanilla_<seed>_4.density（WorldGenBench 二进制：WGB1 + seed + size + xzInt + yInt + 每 chunk 块）
// 对每个采样点算 Rust finalDensity，与 vanilla 逐点对比。这是【直接 vs 原版】验证，非 C++ 转递。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;

fn build_tree(seed: u64) -> Arc<DensityFunction> {
    let mut db = DensityBuilder::new(seed, -64, 384);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let df_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    db.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        let p = PathBuf::from(format!("{}\\{}.json", df_dir, name));
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("\n[LOADFAIL] {}", p.display()))
    }));
    let settings = parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let fd = settings.get("noise_router").and_then(|r| r.get("final_density")).unwrap();
    Arc::new(db.build_node(fd).unwrap())
}

fn be32(b: &[u8], i: &mut usize) -> i32 { let v = i32::from_be_bytes(b[*i..*i+4].try_into().unwrap()); *i += 4; v }
fn be64(b: &[u8], i: &mut usize) -> i64 { let v = i64::from_be_bytes(b[*i..*i+8].try_into().unwrap()); *i += 8; v }

fn main() {
    let seed: i64 = -2032795982907864146;
    let path = "E:\\PYTHON\\CoreSwap\\.investigations\\rust-density-builder\\vanilla_-2032795982907864146_4.density";
    let b = fs::read(path).unwrap();
    let mut i = 0usize;
    let magic = be32(&b, &mut i);
    let vseed = be64(&b, &mut i);
    let size = be32(&b, &mut i);
    let xz = be32(&b, &mut i);
    let y = be32(&b, &mut i);
    println!("magic=0x{:X} seed={} size={} xzInt={} yInt={}", magic, vseed, size, xz, y);
    assert_eq!(magic, 0x57474231, "WGB1 magic");
    assert_eq!(vseed, seed, "seed mismatch");

    let tree = build_tree(seed as u64);
    let mut total = 0u64; let mut matched = 0u64; let mut max_diff = 0.0f64; let mut worst = (0i32,0i32,0i32,0.0f64,0.0f64);
    for _c in 0..(size as usize)*(size as usize) {
        let wx = be32(&b, &mut i); let wz = be32(&b, &mut i);
        let sx = be32(&b, &mut i); let sy = be32(&b, &mut i); let sz = be32(&b, &mut i);
        let min_y = be32(&b, &mut i); let height = be32(&b, &mut i);
        for yidx in 0..sy {
            for zidx in 0..sz {
                for xidx in 0..sx {
                    let rv = f64::from_bits(be64(&b, &mut i) as u64);
                    let x = wx*16 + xidx*xz;
                    let yy = min_y + yidx*y;
                    let zz = wz*16 + zidx*xz;
                    let got = tree.sample(&NoisePos{x, y: yy, z: zz});
                    total += 1;
                    let d = (got - rv).abs();
                    if d < 1e-9 { matched += 1; }
                    if d > max_diff { max_diff = d; worst = (x, yy, zz, rv, got); }
                }
            }
        }
    }
    println!("Rust finalDensity vs vanilla ({} pts): matched(<1e-9)={}/{}  maxDiff={:.3e}", total, matched, total, max_diff);
    println!("worst @({},{},{}) vanilla={} rust={}", worst.0, worst.1, worst.2, worst.3, worst.4);
}
