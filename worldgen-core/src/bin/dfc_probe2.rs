// dfc_probe2.rs — dump interp[1] 闭包节点表 + 逐步求值（定位常数输出根因）
use WorldgenRust::dfc_backend::DfcBackend;

const SEED: i64 = -8248318472910187742;

fn main() {
    let be = DfcBackend::new(SEED as u64);
    be.dump_closures();
    let _ = be;
}
