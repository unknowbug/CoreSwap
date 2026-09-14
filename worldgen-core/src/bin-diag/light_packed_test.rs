// 探针轮 260914-04：light_decode_packed 最小合成帧单测（定位 packed rc=-2 失败点）
// 场景：216 节 = 214 空节 + 1 singular（bits=0,psz=1）+ 1 bits4 节（psz=2，8 long 位流）
use WorldgenRust::light::{light_decode_packed, BLOCKS9_LEN};

fn main() {
    let mut meta = vec![0i32; 432];
    let mut pal: Vec<i32> = Vec::new();
    let mut sto: Vec<i64> = Vec::new();

    // 节 0（c=0,s=0）：singular，pal 编码 = 5（rawId5|lum0）
    meta[0] = 0;
    meta[1] = 1;
    pal.push(5);

    // 节 1（c=0,s=1）：bits=4, psz=2，全 216 节里它独占位流——4096 元素 / epl16 = 256 long，
    // 每元素值交替 0/1：long j 内 16 个 nibble = 0x1111...（低 16 nibble 全 1）
    meta[2] = 4;
    meta[3] = 2;
    pal.push(7);
    pal.push(9);
    for _ in 0..256 {
        sto.push(0x1111111111111111u64 as i64); // 每元素=1 → pal[1]=9
    }

    let mut out = vec![0i32; BLOCKS9_LEN];
    match light_decode_packed(&meta, &pal, &sto, &mut out) {
        Ok(()) => {
            // 校验：节 0 全 5；节 1 全 9；节 2（空）全 0
            let s0_ok = out[0..4096].iter().all(|&v| v == 5);
            let s1_ok = out[4096..8192].iter().all(|&v| v == 9);
            let s2_ok = out[8192..12288].iter().all(|&v| v == 0);
            println!("[PACKED-TEST] Ok singular={} bits4={} empty={}", s0_ok, s1_ok, s2_ok);
            assert!(s0_ok && s1_ok && s2_ok);
        }
        Err(e) => println!("[PACKED-TEST] Err {:?}", e),
    }

    // 负对照：索引越界（psz=1 但位流有值 1）
    let mut meta2 = vec![0i32; 432];
    meta2[0] = 4;
    meta2[1] = 1;
    let pal2 = vec![7i32];
    let sto2 = vec![0x1111111111111111u64 as i64; 256];
    let mut out2 = vec![0i32; BLOCKS9_LEN];
    match light_decode_packed(&meta2, &pal2, &sto2, &mut out2) {
        Ok(()) => println!("[PACKED-TEST] NEG-CONTROL FAIL (should Err)"),
        Err(e) => println!("[PACKED-TEST] neg-control Err {:?} (expected PackedDecode)", e),
    }
    println!("[PACKED-TEST] done");
}
