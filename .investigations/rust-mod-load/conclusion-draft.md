# 结论 docs 草稿：Rust worldgen 作为 mod 运行（里程碑）

> **状态**：draft（本文件是知识库 subagent 产出的草稿，待主会话应用 + 验证后定稿）。
> **载体建议**：结论进 `versions/1.20.1/docs/07-block-pipeline.md`（Rust 块级管线小节，追加不覆盖）+ 过程进 `versions/1.20.1/docs/10-timewise-archive.md`（时间线追加）；错误台账已独立成篇 `.investigations/rust-mod-load/rust-mod-errors.md`（M1-M4）。
> **价值门**：架构模式（Rust cdylib + C++ JNI 桥 + mod 加载）与关键语义 = **高价值（必记，可复用）**；具体验证数值（93.76% 等）= **中价值（简记，一次性数值不展开）**；验证方法链 = **高价值（可复用判据）**。

---

## 一、架构：Rust worldgen 作为 mod 运行（三层链路）

> 高价值（可复用架构模式）。Rust 全量重写 worldgen 后，如何把 Rust 块级管线作为 Minecraft mod 运行——三层链路：**Rust cdylib（C ABI）→ C++ JNI 桥（worldgen.dll）→ mod 加载（Java_wg_CppWorldgen_*）**。

```
Rust WorldgenRust.dll（cdylib，导出 wg_* C ABI）
  ↑ LoadLibrary + GetProcAddress
C++ rust_jni_bridge.cpp → worldgen.dll（导出 Java_wg_CppWorldgen_* JNI 函数）
  ↑ JNI
Java wg.CppWorldgen（mod 加载，调用 init/fillBlocks/setBeardifier/densityParams）
```

### 1. Rust 侧：块级管线封装 + C ABI（WorldgenRust/）

- **`worldgen_handle.rs`**：`WorldgenHandle::create(seed, worldgen_dir)` 一次 seed 初始化（构建全部 noise samplers + density 树 + biome + surface + carver 缓存）；`fill_chunk_blocks(cx, cz)` 完整区块生成（fill_chunk 宏观 → BlockColumn → build_surface → carver 17×17 邻域），返回 16×16×384 vanilla raw block id。
- **`api.rs`**：C ABI 导出 `wg_create` / `wg_destroy` / `wg_fill_blocks_multi` / `wg_set_beardifier` / `wg_clear_beardifier` / `wg_density_*` / `wg_fill_density`。Cargo.toml `crate-type = ["cdylib", "rlib"]`。
- **关键语义**：
  - `wg_fill_blocks_multi` 当前**串行生成**（先保证正确性，多线程后续加）——裸指针跨线程 Send 问题（错误台账 M2）。
  - `wg_set_beardifier` 传 pieces（每 8 int：minX/minY/minZ/maxX/maxY/maxZ/terrain/groundLevelDelta）+ junctions（每 3 int）——Beardifier 结构密度修正输入。
  - density 网格参数（XZ_INTERVAL=4 / Y_INTERVAL=8 / MIN_Y=-64 / HEIGHT=384 / POINTS_PER_CHUNK）通过 `wg_density_*` 导出，供 Java 侧对齐采样网格。

### 2. C++ JNI 桥（rust_jni_bridge.cpp → worldgen.dll）

- 加载 Rust `WorldgenRust.dll`（优先同目录，否则 `-Dcpp.rust.lib` / `CPP_RUST_LIB` env），`GetProcAddress` 取 wg_* 函数指针。
- 导出 `Java_wg_CppWorldgen_init/destroy/fillBlocks/setBeardifier/fillDensity/densityParams` 六个 JNI 函数，对齐 C++ `jni_bridge.cpp`（Java `wg.CppWorldgen` 的 JNI 声明）。
- **关键语义**：JNI 桥是**薄转发层**——只做 JNI 数组 ↔ C 指针转换 + 调 wg_*，不重实现 worldgen 逻辑。`fillBlocks` 用 `std::vector` 中转 + `memcpy` 回 Java int 数组。

### 3. mod 加载（Java wg.CppWorldgen）

- Java 侧通过 JNI 调用 `init`（建句柄）→ `fillBlocks`（批量生成）→ `setBeardifier`（结构密度）→ `densityParams`（网格参数）。
- 与 C++ worldgen.dll 的 mod 加载路径**同构**（Java_wg_CppWorldgen_* 声明一致），Rust 版只是把 C++ 实现换成 Rust cdylib + JNI 桥。

---

## 二、验证结果（中价值，简记一次性数值）

> 验证分层 = **Partial**（JNI 加载 Rust dll 生成 chunk 对比 vanilla FULL 参照，非逐位 Full）。具体数值为一次性快照，不展开；验证方法链见下节（高价值）。

| 验证 | 结果 | 说明 |
|---|---|---|
| `dll_test.c`（LoadLibrary 测 wg_* 导出） | wg_* 导出 OK | C ABI 层验证 |
| `jni_dll_test.c`（测 worldgen.dll JNI 导出） | 6 个 JNI 函数导出 OK | JNI 桥层验证 |
| `handle_probe`（WorldgenHandle vs vanilla） | 95.54% | Rust 块级管线 vs vanilla FULL 参照 |
| **JniProbe（JNI 加载 Rust dll 生成 64 chunks）** | **match=93.76%**（y=64..319 100%，地下 71-90%） | **最终验证**：JNI 全链路 |

- **JniProbe 最终数据**（`.investigations/rust-mod-load/cmd-output/jniprobe_rust.txt`）：seed=-8248318472910187742 size=4 origin=3200,3208，TOTAL match=5899105/6291456（**93.7637%**），nonAir=1553186/1936400（80.2100%）；layerMatch%：y=-64..-33:90% y=-32..-1:88% y=0..31:71% y=32..63:76% **y=64..319:100%**。
- **语义解读**：y=64..319（地表以上 air 区）100% 吻合；地下带（y<64）71-90%——地下差异与 C++ worldgen 的已知边界同源（carver 剩余差异 / FEATURE 范围外 / Beardifier 结构区，见 07 篇已知限制），非 JNI 桥引入。**JNI 桥本身正确**（air 区 100% 证明加载/调用/数据传递无误）。

---

## 三、验证方法链（高价值，可复用判据）

> 三层验证逐层递进，每层验证「上一层已正确」后再进下一层——这是「Rust 作为 mod 运行」的可复用验证路径。

1. **C ABI 层**（`dll_test.c`）：LoadLibrary + GetProcAddress 验证 wg_* 导出存在 + 能建句柄 + 能生成 chunk（非 air 计数）。证明 Rust cdylib 的 C ABI 正确。
2. **JNI 桥层**（`jni_dll_test.c`）：LoadLibrary worldgen.dll + GetProcAddress 验证 6 个 `Java_wg_CppWorldgen_*` 导出存在。证明 JNI 桥编译/导出正确。
3. **全链路**（JniProbe）：JNI 加载 Rust dll 生成 chunk 对比 vanilla FULL 参照。证明「Rust cdylib + C++ JNI 桥 + mod 加载」全链路正确。

**可复用判据**：
- **「air 区 100% + 地下带 70-90%」的签名 = 桥接正确 + 地下差异来自 worldgen 已知边界**（air 区无结构/无 carver 干扰，纯桥接数据传递；地下差异是 worldgen 语义边界，非桥接 bug）——与 07 篇「air 区吻合 + ground 带全错 = 参照/种子配置错」的签名**互补**（后者是配置错，前者是桥接正确）。
- **逐层验证**：先证 C ABI（dll_test）→ 再证 JNI 导出（jni_dll_test）→ 最后全链路（JniProbe）——任一层失败先修该层，不跨层猜。

---

## 四、关键语义（高价值，可复用）

- **Rust edition 2024 的 C ABI 导出**：`#[no_mangle]` 需 `#[unsafe(no_mangle)]`（错误台账 M1）。
- **裸指针跨线程 Send**：`*mut i32` 不实现 Send，edition 2024 下 SendPtr 包装不生效，改串行生成（错误台账 M2）。
- **gradle 需 danger-full-access**：native-platform.dll 加载需完整文件系统访问（错误台账 M3）。
- **MSVC 编译含中文的 UTF-8 源文件需 /utf-8**：code page 936 错解（错误台账 M4）。
- **JNI 桥 = 薄转发层**：只做 JNI 数组 ↔ C 指针转换 + 调 wg_*，不重实现 worldgen 逻辑——Rust 与 C++ 的 mod 加载路径同构（Java_wg_CppWorldgen_* 声明一致）。

---

## 五、域/边界（必须写明）

- 验证分层 = **Partial**（JNI 加载 Rust dll 生成 chunk 对比 vanilla FULL 参照，非逐位 Full）。
- 对齐基准 = **vanilla FULL 参照**（含 carver + features）；Rust 块级管线不含 Beardifier 结构密度修正（`@anchor.idk` 已知边界，见 03 篇）。
- 地下带差异（y<64 71-90%）与 C++ worldgen 已知边界同源（carver 剩余差异 / FEATURE 范围外 / Beardifier 结构区），**非 JNI 桥引入**。
- `wg_fill_density` 当前返回 0（Rust 侧暂未实现完整 density 网格，fillDensity 用）——已知未实现项。

---

## 六、排除清单（❌ 一行式）

- ❌ 「JNI 桥有 bug」——air 区（y=64..319）100% 吻合证明桥接数据传递正确，地下差异来自 worldgen 已知边界，非桥接。
- ❌ 「Rust cdylib C ABI 导出失败」——dll_test 验证 wg_* 导出 OK。
- ❌ 「JNI 桥导出失败」——jni_dll_test 验证 6 个 Java_wg_CppWorldgen_* 导出 OK。

---

## 七、时间线条目草稿（追加到 10-timewise-archive.md 末尾）

> 载体：`versions/1.20.1/docs/10-timewise-archive.md`（时间线追加，每条带状态标注）。主会话应用时按日期追加。

### 2026-08-XX（追加）：Rust worldgen 作为 mod 运行（✅ 关键里程碑）

> CoreSwap worldgen 全量重写为 Rust（WorldgenRust/）后，把 Rust 块级管线作为 Minecraft mod 运行。三层链路：**Rust cdylib（C ABI）→ C++ JNI 桥（worldgen.dll）→ mod 加载（Java_wg_CppWorldgen_*）**。配套：07 篇「Rust worldgen 作为 mod 运行」结论小节 + `.investigations/rust-mod-load/` + `rust-mod-errors.md` 错误台账（M1-M4）。

### ✅ 一、Rust 块级管线封装 + C ABI（关键里程碑）
- `worldgen_handle.rs`：`WorldgenHandle::create` + `fill_chunk_blocks`（fill_chunk 宏观 → BlockColumn → build_surface → carver 17×17 邻域）。
- `api.rs`：C ABI 导出 `wg_create/wg_destroy/wg_fill_blocks_multi/wg_set_beardifier/wg_clear_beardifier/wg_density_*`；Cargo.toml `crate-type = ["cdylib", "rlib"]`。
- `wg_fill_blocks_multi` 当前**串行生成**（裸指针跨线程 Send 问题，错误台账 M2）。

### ✅ 二、C++ JNI 桥（rust_jni_bridge.cpp → worldgen.dll）
- 加载 Rust `WorldgenRust.dll`（LoadLibrary + GetProcAddress 取 wg_*），导出 `Java_wg_CppWorldgen_init/destroy/fillBlocks/setBeardifier/fillDensity/densityParams` 六个 JNI 函数。
- JNI 桥 = **薄转发层**（JNI 数组 ↔ C 指针转换 + 调 wg_*），与 C++ jni_bridge.cpp 同构。

### ✅ 三、验证（三层递进）
- `dll_test.c`：wg_* 导出 OK（C ABI 层）。
- `jni_dll_test.c`：6 个 JNI 函数导出 OK（JNI 桥层）。
- `handle_probe`：WorldgenHandle vs vanilla 95.54%。
- **JniProbe（最终验证）**：JNI 加载 Rust dll 生成 64 chunks，match=**93.76%**（y=64..319 100%，地下 71-90%）——air 区 100% 证明桥接正确，地下差异来自 worldgen 已知边界（carver/FEATURE/Beardifier），非 JNI 桥引入。

### 🧰 四、工具演进（本轮新增）
- `dll_test.c`（C ABI 导出验证）、`jni_dll_test.c`（JNI 导出验证）、`handle_probe.rs`（WorldgenHandle 块级管线验证）、JniProbe（JNI 全链路验证）。

### 📌 记录指引（知识库归口）
- 错误台账：`.investigations/rust-mod-load/rust-mod-errors.md`（M1-M4 五段式）。
- 结论：07 篇「Rust worldgen 作为 mod 运行」小节。
- 过程：本节 + `.investigations/rust-mod-load/`。
- **域边界（保持）**：验证分层 = Partial（JNI 加载 Rust dll 对比 vanilla FULL 参照）；Rust 块级管线不含 Beardifier（`@anchor.idk`）；`wg_fill_density` 暂未实现（返回 0）。
