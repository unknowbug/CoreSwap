// multiworld_nether_blocks.rs — Phase A：Rust nether 块级验证（fill_chunk_blocks vs vanilla nether 参照）。
// 参照：E:\PYTHON\MC\data\vanilla_-8248318472910187742_4_0_0_nether.blocks（WGB2 大端，size 4, origin 0,0, min_y 0, height 256）。
// 输出：match% / nonAir match% / 每 32 层带 match% / 首 10 个 mismatch。
use WorldgenRust::worldgen_handle::WorldgenHandle;
use WorldgenRust::density::NoisePos as NP2;
use WorldgenRust::biome::BiomeClassifier;
use WorldgenRust::density_builder::DensityBuilder as DB2;
use WorldgenRust::json::parse as parse2;

fn be16(b: &[u8], i: &mut usize) -> u16 { let v = u16::from_be_bytes(b[*i..*i+2].try_into().unwrap()); *i += 2; v }
fn be32(b: &[u8], i: &mut usize) -> i32 { let v = i32::from_be_bytes(b[*i..*i+4].try_into().unwrap()); *i += 4; v }
fn be64(b: &[u8], i: &mut usize) -> i64 { let v = i64::from_be_bytes(b[*i..*i+8].try_into().unwrap()); *i += 8; v }

fn main() {
    let seed: i64 = -8248318472910187742;
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = match WorldgenHandle::create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256) {
        Some(h) => h,
        None => { println!("[FAIL] nether create_for_dim failed"); return; }
    };
    println!("handle: min_y={} height={}", h.min_y, h.height);

    let path = "E:\\PYTHON\\MC\\data\\vanilla_-8248318472910187742_4_0_0_nether.blocks";
    let bd = std::fs::read(path).unwrap();
    let mut i = 0usize;
    let magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let size = be32(&bd, &mut i);
    let origin_x = be32(&bd, &mut i); let origin_z = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("ref: magic=0x{:X} seed={} size={} origin=({},{}) min_y={} height={}", magic, vseed, size, origin_x, origin_z, min_y, height);
    let bpc = (16 * 16 * height) as usize;
    if h.min_y != min_y || h.height != height { println!("[WARN] handle 维度与参照不一致"); }

    // WG_BIOMEDUMP：nether biome 分类诊断（mismatch 位置判定名）
let dump_biome = std::env::var("WG_BIOMEDUMP").is_ok();
let (bc, trees) = if dump_biome {
    let mut db2 = DB2::new(seed as u64, min_y, 128);
    db2.load_noise_params_file(&format!("{}/../noise_params.json", wg_dir)).unwrap();
    db2.set_df_ns("nether");
    let df2 = format!("{}/data/minecraft/worldgen/density_function/nether", wg_dir);
    let df2c = df2.clone();
    db2.set_external_loader(Box::new(move |_f: &str, name: &str| -> String {
        std::fs::read_to_string(&format!("{}/{}.json", df2c, name)).unwrap()
    }));
    if settings2_ok_legacy(&wg_dir) { db2.set_legacy_random(); }
    let settings2 = parse2(&std::fs::read_to_string(format!("{}/data/minecraft/worldgen/noise_settings/nether.json", wg_dir)).unwrap()).unwrap();
    let router2 = settings2.get("noise_router").unwrap();
    let mut trees = std::collections::HashMap::new();
    for key in ["temperature","vegetation","continents","erosion","depth","ridges"] {
        trees.insert(key.to_string(), std::sync::Arc::new(db2.build_node(router2.get(key).unwrap()).ok().unwrap()));
    }
    let bc = BiomeClassifier::load(&format!("{}/../biome_params_nether.json", wg_dir));
    (Some(bc), Some(trees))
} else { (None, None) };

let mut total = 0u64; let mut match_t = 0u64; let mut tnair = 0u64; let mut mnair = 0u64;
    // 分层统计：32 层一带
    let bands = (height / 32) as usize;
    let mut band_total = vec![0u64; bands]; let mut band_match = vec![0u64; bands];
    let mut mismatches: Vec<(i32, i32, i32, i32, i32)> = Vec::new();
    for _c in 0..(size * size) {
        let cx = be32(&bd, &mut i); let cz = be32(&bd, &mut i);
        let mut vanilla = vec![0i32; bpc];
        for k in 0..bpc { vanilla[k] = be16(&bd, &mut i) as i32; }
        for _bi in 0..256 { let bl = be16(&bd, &mut i) as usize; if bl > 0 { i += bl; } }

        let blocks = h.fill_chunk_blocks(cx, cz);
        if blocks.len() != bpc { println!("[FAIL] chunk({},{}) blocks.len={} != bpc={}", cx, cz, blocks.len(), bpc); return; }
        for k in 0..bpc {
            let got = blocks[k]; let want = vanilla[k];
            total += 1;
            if want != 0 { tnair += 1; }
            let y = min_y + (k / 256) as i32;
            let band = ((y - min_y) / 32) as usize;
            if got == want {
                match_t += 1; if want != 0 { mnair += 1; }
                band_match[band] += 1;
            } else if mismatches.len() < 10 {
                let yy = (k / 256) as i32;
                let rem = (k % 256) as i32;
                let zz = rem / 16;
                let xx = rem % 16;
                mismatches.push((cx * 16 + xx, min_y + yy, cz * 16 + zz, got, want));
                if dump_biome {
                    if let (Some(bc), Some(trees)) = (&bc, &trees) {
                        let pos = NP2 { x: cx * 16 + xx, y: min_y + yy, z: cz * 16 + zz };
                        let g = |k2: &str| trees.get(k2).unwrap().as_ref();
                        let name = bc.biome_of(g("temperature"), g("vegetation"), g("continents"), g("erosion"), g("depth"), g("ridges"), &pos);
                        println!("[BIOME] ({},{},{}) -> {} (want id{})", pos.x, pos.y, pos.z, name, want);
                    }
                }
            }
            band_total[band] += 1;
        }
    }
    println!("TOTAL: match={}/{} ({:.4}%)  nonAir match={}/{} ({:.4}%)",
        match_t, total, 100.0 * match_t as f64 / total as f64,
        mnair, tnair, 100.0 * mnair as f64 / tnair.max(1) as f64);
    for b in 0..bands {
        let y0 = min_y + (b * 32) as i32;
        println!("  y={}..{}: {}/{} ({:.2}%)", y0, y0 + 31, band_match[b], band_total[b],
            100.0 * band_match[b] as f64 / band_total[b].max(1) as f64);
    }
    println!("first mismatches (x,y,z, got, want): {:?}", mismatches);

    // 混淆对直方图：每带 Top got→want 配对（定性判断错位类型：阈值翻转 / id 错位 / 缺方块类）
    if std::env::var("WG_CONFUSION").is_ok() {
        let name_of = |id: i32| -> String {
            match id { 0 => "air".into(), 31 => "bedrock".into(), 33 => "lava".into(), 256 => "netherrack".into(), 257 => "soul_sand".into(), other => format!("id{}", other) }
        };
        let mut confusion: std::collections::HashMap<(i32, i32, i32), u64> = std::collections::HashMap::new();
        let mut i2 = 0usize;
        let _magic2 = be32(&bd, &mut i2); let _seed2 = be64(&bd, &mut i2); let size2 = be32(&bd, &mut i2);
        let _ox = be32(&bd, &mut i2); let _oz = be32(&bd, &mut i2); let _my = be32(&bd, &mut i2); let _hh = be32(&bd, &mut i2);
        let bpc2 = (16 * 16 * height) as usize;
        for _c2 in 0..(size2 * size2) {
            let cx2 = be32(&bd, &mut i2); let cz2 = be32(&bd, &mut i2);
            let mut vanilla2 = vec![0i32; bpc2];
            for k in 0..bpc2 { vanilla2[k] = be16(&bd, &mut i2) as i32; }
            for _bi in 0..256 { let bl = be16(&bd, &mut i2) as usize; if bl > 0 { i2 += bl; } }
            let blocks2 = h.fill_chunk_blocks(cx2, cz2);
            for k in 0..bpc2 {
                let g = blocks2[k]; let w = vanilla2[k];
                if g != w {
                    let y = min_y + (k / 256) as i32;
                    let band = ((y - min_y) / 32) as usize;
                    *confusion.entry((band as i32, g, w)).or_insert(0) += 1;
                }
            }
        }
        let mut pairs: Vec<((i32, i32, i32), u64)> = confusion.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        println!("[混淆对] band(y0) got->want count Top12:");
        for (i2, ((band, g, w), cnt)) in pairs.iter().enumerate().take(12) {
            println!("  y{}..: {} -> {} : {}", min_y + band * 32, name_of(*g), name_of(*w), cnt);
        }
    }
}




fn settings2_ok_legacy(wg_dir: &str) -> bool {
    let p = format!("{}/data/minecraft/worldgen/noise_settings/nether.json", wg_dir);
    let s = parse2(&std::fs::read_to_string(&p).unwrap()).unwrap();
    s.get("legacy_random_source").and_then(|v| v.as_bool()).unwrap_or(false)
}
