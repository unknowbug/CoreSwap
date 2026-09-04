// beard_weight_probe.rs — 输出 Beardifier 权重表（calculate_structure_weight）几个值，供 powf 逐位对拍。
// 权重表 = pow(2.718281828459045, -squaredMagnitude(x, y+0.5, z)/16.0)，float 截断。
// 输出几个点，与 Java Math.pow / C++ std::pow 参照对比（验证 Rust powf 逐位一致）。
use WorldgenRust::beardifier::Beardifier;

fn main() {
    // 通过 Beardifier 的权重表间接验证：构造一个 BEARD_BOX piece，sample 输出依赖权重表。
    // 但更直接：手动复刻 calculate_structure_weight 的几个点。
    // 权重表索引：table[i*576 + j*24 + k] = calculate_structure_weight(j-12, k-12, i-12)
    // 即 table 在 (i,j,k) 存 calculate_structure_weight(j-12, k-12, i-12)。
    // 输出几个 (x,y,z) 的 calculate_structure_weight 值（pow 语义）。
    let points = [
        (0, 0, 0),      // 中心：pow(E, -(0 + 0.25 + 0)/16) = pow(E, -0.015625)
        (1, 0, 0),      // pow(E, -(1 + 0.25 + 0)/16)
        (0, 1, 0),      // pow(E, -(0 + 2.25 + 0)/16)
        (3, 2, 1),      // pow(E, -(9 + 6.25 + 1)/16)
        (5, 5, 5),      // pow(E, -(25 + 30.25 + 25)/16)
        (11, 11, 11),   // 边缘
    ];
    println!("Beardifier calculate_structure_weight (pow(E, -d/16), float-truncated):");
    for &(x, y, z) in &points {
        // 复刻 C++ calculateStructureWeight：pow(2.718281828459045, -d/16.0)，float 截断
        let dx = x as f64; let dy = y as f64 + 0.5; let dz = z as f64;
        let d = dx*dx + dy*dy + dz*dz;
        let v = (2.718281828459045_f64).powf(-d / 16.0) as f32;
        println!("  ({},{},{}) d={:.4} weight={:.9} (f32)", x, y, z, d, v);
    }
    println!("beard_weight_probe done (compare weights to Java Math.pow / C++ std::pow)");
}
