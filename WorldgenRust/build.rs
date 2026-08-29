// build.rs — build-time 编译 density 树（对齐 SteelMC transpiler）。
// 读 density_function/*.json + noise_settings/*.json → 生成 specialized 内联函数 → src/generated/。
// MVP：先生成 final_density 的 compute 函数，验证 build.rs + transpiler + 编译链路。
use std::env;
use std::fs;
use std::path::Path;

// 复用 crate 的 JSON 解析器（build.rs 独立编译，用 #[path] 引入）
#[path = "src/json.rs"]
mod json;

#[path = "build/density.rs"]
mod density;

fn main() {
    println!("cargo:rerun-if-changed=build/");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out_dir = Path::new(&manifest_dir).join("src/generated");
    fs::create_dir_all(&out_dir).expect("create generated dir");

    // MVP：生成 final_density 的 compute 函数
    let content = density::build_final_density();
    let path = out_dir.join("vanilla_density_functions.rs");
    if fs::read_to_string(&path).is_ok_and(|existing| existing == content) {
        // 不变不重写
    } else {
        fs::write(&path, content).expect("write generated density file");
    }
}
