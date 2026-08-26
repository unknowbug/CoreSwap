# rust-density-builder 对齐验证记录（冻结快照）

> 2026-08-24 | 主会话 | 目的：把 Rust-vs-C++ buildNode 对齐证据**冻结**到 .investigations/，使 diff=0 可复现、可回溯（响应 judge P2-②）。

## 冻结文件（SHA256）
| 文件 | SHA256 |
|---|---|
| rust_out.txt（Rust overworld_probe 输出） | 30D8EAB97A227CC7AD896A85104CF44B9A4210D899B40B4456247496EBA6F7AF |
| cpp_out.txt（C++ rust_ref_check 输出） | FC9BFD7C129C8F5F9E2FFFC92C1F826ACA463F93EFAE4CD82ACAD8F7D28CE056 |
| rust_ref_check.cpp（C++ 参照源） | 006D4AF0ECB90FDB07713AEA8C5B97BDAB78A5F02F3A23AFC3158ED91C51801F |
| overworld_probe.rs（Rust 探针源） | F194FAFF7748266872E3C98DEB9DA41CC9ED3694CD3795181CF632E37AED01EF |

## 冻结文件 v1.1（追加 full finalDensity 对齐）
| 文件 | SHA256 |
|---|---|
| rust_fd_out.txt（Rust finaldensity_probe 输出） | 4229EBD2D523D50F8A78C7072B6F274B0A78EB65400A0C3969A0B3388A4521E9 |
| cpp_fd_out.txt（C++ rust_ref_check final_density 输出） | 2FB002EA014A6AFE9CFB30281A4ACC2D0B6E15B13300429B55DD838697B2C125 |
| finaldensity_probe.rs（Rust finalDensity 探针源） | BCE718B5350192836353549CD70C423292BB3F9A8983D86A5F3E2ADBAA2E45E2 |

## 复现命令
```powershell
# 1) C++ 参照（cl 直链，MSVC，见 rust_ref_check.cpp 顶部注释；需同一 buildNoiseParams 全表 + md5.cpp）
# 2) Rust 探针
$env:CARGO_PATH = "$env:USERPROFILE\.cargo\bin\cargo.exe"
& "$env:CARGO_PATH" run --bin overworld_probe --quiet > rust_out.txt
& "versions/1.20.1/cpp/build-msvc/bin/rust_ref_check.exe" > cpp_out.txt
# 3) 规范化差分（提取所有 =<num> 列）；期望：仅差 Rust 探针末尾 "sloped_cheese min=... max=..." summary 行
```

## 覆盖规模（响应 judge P1-①）
- 16 个 overworld 密度函数（10 顶层 + 6 caves/*）× 10 个采样点 = 160 个采样值 + 各函数 min/max（16 对）。
- 采样点：{(0,0,0),(4,64,4),(8,128,8),(40,192,40),(100,-64,-40),(-64,64,-64),(128,288,128),(200,0,200),(16,-112,16),(72,320,72)}（覆盖跨 min_y..max_y 与跨 xz，exercise interpolated grid/old_blended_noise 高频）。
- 已知近退化点：factor（continents<-0.19 左端平段=3.95）、jaggedness（=0.0）、offset（部分点同值）——**未触 spline 拐点区**（见 judge P1-① 提示；下一步应抽自变量范围更宽的网格点）。

## 对齐基准（响应 judge P1-②）
- **对齐对象 = C++ `density_builder.h`（buildNode）**，非 Java vanilla 逐块。两套为自洽复现（同 JSON 语义 + 同 noise_params 硬编码表），共同误解会导致 diff=0 但偏离 vanilla。
- **Rust-vs-vanilla（block_probe/ref density）逐位对齐未做**——属待办（见 NEXT_SESSION「下一批」2）。

## 验证分层（响应 judge P3-②）
- 分层：**Full（逐位）**——Rust 输出 vs C++ 输出逐位一致（非反射/非静态）。
- 执行者：主会话（cl / cargo 直跑）。分层有效域 = C++ buildNode；对 vanilla 属 Partial/Degraded（未测）。

## judge 处置（响应 judge P1-①/②/③, P2, P3）
- judge：**有条件通过**，推荐维持 candidate；confirmed 留给用户。
- 已处理：P1-①（覆盖扩到 160 采样 + 标注近退化点）、P1-②（对齐基准显式 = C++ buildNode）、P3-①（产物已改 16 函数）、P3-②（分层声明）。
- 未处理（P2，交用户/下一批）：P2-① WorldgenRust 未入 git（无 HEAD 基线，审整文件终态）；P2-③ noise_params 硬编码表 vs minecraft:noise_params.json（两端共同继承，建议下一步逐键对参数文件或改读文件）。
- judge 意见：`E:\PYTHON\CoreSwap\.investigations\rust-density-builder\review-001.md`（核心——维持 candidate，confirmed 升格的必要条件列出）。
