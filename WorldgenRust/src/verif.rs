// 验证：ref 数据加载 + 逐位对比（逆向对齐 Java 参照）
use std::fs;

/// 逐位对比两个 f64 切片，返回 (max abs diff, mismatch 数, 总数)
pub fn compare_f64(a: &[f64], b: &[f64]) -> (f64, usize, usize) {
    assert_eq!(a.len(), b.len(), "compare_f64 length mismatch");
    let mut maxd = 0.0f64;
    let mut mism = 0;
    for i in 0..a.len() {
        let d = (a[i] - b[i]).abs();
        if d > maxd { maxd = d; }
        if d > 1e-7 { mism += 1; }
    }
    (maxd, mism, a.len())
}

/// 从纯文本行读 ref 数据（每行最后一个 token 是 f64）。
/// 例：`vanilla_density_overworld_*_cns.txt` 格式 `y value`；`vanilla_*.txt` 分量 dump。
pub fn load_ref_lines(path: &str) -> Vec<f64> {
    let data = fs::read_to_string(path).unwrap_or_default();
    data.lines()
        .filter_map(|l| l.split_whitespace().last().and_then(|w| w.parse::<f64>().ok()))
        .collect()
}
