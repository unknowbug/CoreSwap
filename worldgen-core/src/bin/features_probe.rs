// features_probe.rs — 验证 WorldgenHandle 的 FEATURES 阶段（矿石/装饰层）。
// 用 WorldgenHandle::create + fill_chunk_blocks（含 apply_features）生成 chunk，对比 vanilla FULL 参照。
// 验证：FEATURES 阶段开启后 match 率应提升（矿石/disk/spring 等装饰层）。
use WorldgenRust::worldgen_handle::WorldgenHandle;

fn be16(b: &[u8], i: &mut usize) -> u16 { let v = u16::from_be_bytes(b[*i..*i+2].try_into().unwrap()); *i += 2; v }
fn be32(b: &[u8], i: &mut usize) -> i32 { let v = i32::from_be_bytes(b[*i..*i+4].try_into().unwrap()); *i += 4; v }
fn be64(b: &[u8], i: &mut usize) -> i64 { let v = i64::from_be_bytes(b[*i..*i+8].try_into().unwrap()); *i += 8; v }

fn main() {
    // 260905-05 参数化（原为硬编码 -8248 seed / MC 侧参照）：args[1]=seed args[2]=参照路径
    let args: Vec<String> = std::env::args().collect();
    let seed: i64 = if args.len() > 1 { args[1].parse().expect("seed i64") } else { -8248318472910187742 };
    let default_ref = if seed == -8248318472910187742 {
        "E:\\python\\MC\\data\\vanilla_-8248318472910187742_4_-288_-256_FULL.bak.blocks".to_string()
    } else {
        format!("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\recheck\\vanilla_{seed}_6_720_-432.blocks")
    };
    let ref_path = if args.len() > 2 { args[2].clone() } else { default_ref };
    let wg_dir = "E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen";
    let h = WorldgenHandle::create(seed, wg_dir).expect("create handle");
    println!("handle created: min_y={} height={}", h.min_y, h.height);

    // 读 vanilla FULL 参照（含 carver+features）
    let path = ref_path;
    let bd = std::fs::read(path).unwrap();
    let mut i = 0usize;
    let magic = be32(&bd, &mut i); let vseed = be64(&bd, &mut i); let size = be32(&bd, &mut i);
    let origin_x = be32(&bd, &mut i); let origin_z = be32(&bd, &mut i); let min_y = be32(&bd, &mut i); let height = be32(&bd, &mut i);
    println!("magic=0x{:X} seed={} size={} origin=({},{}) minY={} height={}", magic, vseed, size, origin_x, origin_z, min_y, height);
    let bpc = 16*16*height as usize;

    let mut total = 0u64; let mut match_t = 0u64; let mut tnair = 0u64; let mut mnair = 0u64;
    let mut feature_placed = 0u64; let mut feature_match = 0u64; // FEATURES 放置的方块是否匹配 vanilla
    let trace_id: i32 = if args.len() > 3 { args[3].parse().expect("trace id") } else { -1 };
    for _c in 0..(size*size) {
        let cx = be32(&bd, &mut i); let cz = be32(&bd, &mut i);
        let mut vanilla = vec![0i32; bpc];
        for k in 0..bpc { vanilla[k] = be16(&bd, &mut i) as i32; }
        for _bi in 0..256 { let bl = be16(&bd, &mut i) as usize; if bl>0 { i += bl; } }

        // WorldgenHandle 块级管线（含 FEATURES）
        let blocks = h.fill_chunk_blocks(cx, cz);
        let mut vt = 0i64; let mut gt = 0i64; let mut cd = 0u64;
        for k in 0..bpc {
            let got = blocks[k];
            if trace_id >= 0 { if vanilla[k] == trace_id { vt += 1; } if got == trace_id { gt += 1; } if vanilla[k] != got { cd += 1; } }
            total += 1;
            if vanilla[k] != 0 { tnair += 1; }
            if got == vanilla[k] { match_t += 1; if vanilla[k] != 0 { mnair += 1; } }
        }
        if trace_id >= 0 { println!("chunk({cx},{cz}): diff={cd} trace_id={trace_id} vanilla={vt} rust={gt}"); }
    }
    println!("WorldgenHandle(+features) vs vanilla FULL: match={}/{} ({:.2}%)  nonAir={}/{} ({:.2}%)", match_t, total, 100.0*match_t as f64/total as f64, mnair, tnair, if tnair>0 {100.0*mnair as f64/tnair as f64} else {0.0});
}
