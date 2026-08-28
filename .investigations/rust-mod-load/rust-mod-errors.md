# Rust worldgen 作为 mod 运行：错误与根因清单（重点记录）

> 载体：`.investigations/rust-mod-load/rust-mod-errors.md`（错误台账，独立成篇）。
> 本文件按「现象 → 根因 → 定位 → 修复 → 教训」五段式记录「Rust worldgen 作为 mod 运行」里程碑（Rust cdylib + C++ JNI 桥 + mod 加载）过程中的错误。结论性对齐数据见对应主题篇/时间线；本文件只记「错在哪、为什么错、怎么发现、下次怎么避」。
> 背景：WorldgenRust/ 已完成块级管线封装（`worldgen_handle.rs` WorldgenHandle::create + fill_chunk_blocks）→ C ABI（`api.rs` wg_* 导出，Cargo.toml 加 cdylib）→ JNI 层（`rust_jni_bridge.cpp` 加载 WorldgenRust.dll 导出 Java_wg_CppWorldgen_*）→ 验证（dll_test / jni_dll_test / handle_probe / JniProbe）。本 session 排查踩坑 4 个（M1-M4）。
> 编号：本课题用 **M 系列**（mod-load），与 `.investigations/rust-density-builder/rust-errors.md` 的 R 系列（density_builder 对齐）区分，避免跨课题编号混淆。

---

## M1. Rust edition 2024 的 `#[no_mangle]` 需 `#[unsafe(no_mangle)]`——裸导出编译失败

### 现象
- `api.rs` 用 `#[no_mangle] pub extern "C" fn wg_create(...)` 声明 C ABI 导出，`cargo build` 报错：
  `error: `#[no_mangle]` is not allowed on `extern` functions in edition 2024`（或 `use of `#[no_mangle]` requires `unsafe` in edition 2024`）。
- 编译直接失败，无法产出 cdylib。

### 根因（机制）
- Rust **edition 2024** 把 `#[no_mangle]`（以及 `#[export_name]`、`#[link_section]` 等）从「普通属性」升级为 **`unsafe` 属性**——因为导出符号名/链接段会绕过 Rust 的符号命名与安全检查，属于 unsafe 操作，语言层面强制要求显式 `unsafe` 标注。
- edition 2021 及更早版本 `#[no_mangle]` 是合法普通属性；升级到 edition 2024 后语法收紧，旧写法直接编译失败。
- 本项目 `Cargo.toml` 是 `edition = "2024"`，故触发。

### 定位（诊断方法）
- `cargo build` 报错信息直接点名 `#[no_mangle]` 与 edition 2024 的冲突——**编译器错误即定位**，无需额外工具。
- 对照 Rust edition 2024 迁移文档确认 `#[no_mangle]` 属 unsafe 属性清单。

### 修复
- `api.rs` 全部导出函数把 `#[no_mangle]` 改为 **`#[unsafe(no_mangle)]`**（`#[unsafe(no_mangle)] pub extern "C" fn ...`）。
- 修复后 `cargo build` 通过，cdylib 正常产出。

### 教训（可复用判错经验）
- **Rust edition 2024 下写 C ABI 导出（`#[no_mangle]`/`#[export_name]`）必须用 `#[unsafe(no_mangle)]`**——这是 edition 2024 的语法收紧，不是可选项。
- **先确认 Cargo.toml 的 edition**：edition 2021 用 `#[no_mangle]`，edition 2024 用 `#[unsafe(no_mangle)]`；跨 edition 复制代码时最容易踩。
- 编译器报错信息通常直接点名语法冲突，**先读报错原文再查文档**，不要凭旧版经验改。

---

## M2. 裸指针 `*mut i32` 不能跨线程 Send——SendPtr 包装在 edition 2024 下不生效，改串行生成

### 现象
- `api.rs` 的 `wg_fill_blocks_multi` 想用多线程并行生成 chunk（`threads` 参数），把 `*mut i32` 输出指针放进线程闭包，`cargo build` 报错：
  `error[E0277]: `*mut i32` cannot be sent between threads safely`（`*mut i32` 不实现 `Send`）。
- 尝试用 `SendPtr` 包装（`struct SendPtr(*mut i32); unsafe impl Send for SendPtr {}`）解决，但**在 edition 2024 下该包装不生效**（或仍报错/语义不符）。

### 根因（机制）
- Rust 的裸指针 `*mut T` **默认不实现 `Send`/`Sync`**（语言层面认为裸指针可能指向非线程安全数据，跨线程传递是 unsafe 语义）。
- 多线程并行生成需要把输出指针 move 进线程闭包 → 编译器拒绝。
- **edition 2024 对 `unsafe impl` 的语义收紧**：`unsafe impl Send` 需要显式 `unsafe` 块/标注，且编译器对「unsafe 属性/impl 的合法性」检查更严——`SendPtr` 包装在 edition 2024 下要么编译失败、要么被判定为不满足 Send 语义（裸指针指向的 `Vec<i32>` 数据本身是 Send 的，但裸指针类型不自动推导）。
- 本项目 `Cargo.toml` 是 `edition = "2024"`，故 SendPtr 包装方案失效。

### 定位（诊断方法）
- `cargo build` 报 `E0277` 直接点名 `*mut i32` 不 Send——**编译器错误即定位**。
- 尝试 SendPtr 包装后仍失败 → 确认是 edition 2024 的 unsafe impl 语义收紧，而非单纯缺包装。

### 修复
- **放弃多线程并行，改为串行生成**：`wg_fill_blocks_multi` 用 `for i in 0..count { ... }` 逐个 chunk 串行调用 `fill_chunk_blocks`（`api.rs` L54-59 注释「串行生成（先保证正确性，多线程后续加）」）。
- 串行下无跨线程 move 裸指针问题，编译通过。
- 后续若要并行，需用安全抽象（如把输出指针转成 `Vec<i32>` 的 `&mut` 切片 + 线程池 + 安全所有权转移），而非裸 `SendPtr`。

### 教训（可复用判错经验）
- **裸指针 `*mut T` 不实现 `Send`/`Sync`，跨线程 move 必须用安全抽象（`Vec`/`Arc<Mutex>`/切片所有权），不要裸 `SendPtr` 包装**——尤其 edition 2024 下 `unsafe impl Send` 语义收紧，包装方案更不可靠。
- **「先保证正确性，多线程后续加」是 C ABI 桥接的稳妥路径**：串行先跑通，再考虑并行（并行需安全所有权转移，不是裸指针）。
- 编译器 `E0277` 报错直接点名类型不满足 trait——**先读报错原文，判断是「缺包装」还是「语义本身不允许」**，再决定改法。

---

## M3. gradle native-platform.dll 加载失败——需 danger-full-access 权限

### 现象
- 运行 gradle（`gradle runServer` / JniProbe 相关任务）时，gradle 启动阶段报错：
  `Could not initialize class org.gradle.internal.nativeplatform.filesystem.FileSystemServices` 或 `native-platform.dll` 加载失败（`UnsatisfiedLinkError` / `Could not load native-platform.dll`）。
- gradle 无法启动，JniProbe 无法运行。

### 根因（机制）
- gradle 的 native-platform 库（`native-platform.dll`）在启动时被加载，用于文件系统/进程操作。该 dll 需要**从临时目录解压并加载**，且需要**写临时目录 + 加载 native 库**的权限。
- 在 DSH 沙箱（受限文件权限）下，gradle 无法写临时目录/加载 native dll → 初始化失败。
- 这是**环境权限问题**，不是代码/配置问题——gradle 本身需要完整文件系统访问。

### 定位（诊断方法）
- gradle 报错信息直接点名 `native-platform.dll` 加载失败 + `FileSystemServices` 初始化失败——**报错即定位**。
- 确认是沙箱权限限制（写临时目录/加载 native 库被拦），而非 gradle 配置错误。

### 修复
- 运行 gradle 的命令需要 **`danger-full-access` 权限**（DSH 沙箱放宽到完整文件系统访问），让 gradle 能写临时目录 + 加载 native dll。
- 修复后 gradle 正常启动，JniProbe 可运行。

### 教训（可复用判错经验）
- **gradle 启动需要完整文件系统访问（写临时目录 + 加载 native-platform.dll）**——在受限沙箱下运行 gradle 会因 native 库加载失败而初始化失败，需放宽权限（danger-full-access）。
- **「native 库加载失败」类报错先查环境权限（沙箱/临时目录/写权限），再查配置**——gradle 的 native-platform 是启动必需，不是可选组件。
- 与 AGENTS.md §八.3（pip install 被拦用 PYTHONPATH）同类：**沙箱拦 native 库/安装类操作，需放宽权限或用环境变量规避**。

---

## M4. C 编译器中文注释 code page 936 报错——需 /utf-8 编译选项

### 现象
- 用 MSVC `cl.exe` 编译 `dll_test.c` / `jni_dll_test.c`（含中文注释）时，编译器报错：
  `warning C4819: The file contains a character that cannot be represented in the current code page (936)` 或 `error C2001: newline in constant` / 中文注释被误解析。
- 编译失败或产生警告，无法产出测试 exe。

### 根因（机制）
- MSVC 默认按**系统 code page（中文系统 = 936/GBK）**解析源文件。源文件是 **UTF-8 编码**（含中文注释），编译器按 GBK 读 UTF-8 字节 → 中文字节被错解 → 注释内容被误判为代码/字符串 → 报错。
- 这是**源文件编码（UTF-8）与编译器默认 code page（936/GBK）不匹配**的经典问题。
- 与 AGENTS.md §八.2（PowerShell 中文输出崩溃）同源：**UTF-8 内容在 GBK 环境下的编码错位**。

### 定位（诊断方法）
- 编译器报 `C4819`（字符无法用当前 code page 表示）直接点名编码问题——**报错即定位**。
- 确认源文件是 UTF-8（含中文注释）+ 系统 code page 是 936 → 编码不匹配。

### 修复
- 编译命令加 **`/utf-8`** 选项（`cl /utf-8 dll_test.c ...`），强制 MSVC 按 UTF-8 解析源文件。
- 修复后编译通过，无 C4819 警告。

### 教训（可复用判错经验）
- **MSVC 编译含中文（非 ASCII）的 UTF-8 源文件必须加 `/utf-8`**——否则按系统 code page（936/GBK）错解 UTF-8 字节，中文注释被误判为代码。
- 与 AGENTS.md §八.2 同源：**UTF-8 内容在 GBK 环境下的编码错位**——编译（`/utf-8`）、PowerShell 输出（`PYTHONIOENCODING`）、文件读写（DSH edit/write 干净字节）三处都要防。
- **`C4819` 警告是编码不匹配的明确信号**——看到即加 `/utf-8`，不要忽略。

---

## 附：错误 → 根因 速查表（一页索引）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| `cargo build` 报 `#[no_mangle]` 不允许 / 需 unsafe（M1） | Rust **edition 2024** 把 `#[no_mangle]`/`#[export_name]` 升级为 **unsafe 属性**，必须写 `#[unsafe(no_mangle)]`；edition 2021 及更早是普通属性 | **先确认 Cargo.toml edition**：2021 用 `#[no_mangle]`，2024 用 `#[unsafe(no_mangle)]`；编译器报错直接点名语法冲突，先读原文 |
| `cargo build` 报 `E0277: *mut i32 cannot be sent between threads`（M2） | 裸指针 `*mut T` **默认不实现 `Send`/`Sync`**；edition 2024 对 `unsafe impl Send` 语义收紧，`SendPtr` 包装不生效 | **裸指针跨线程 move 必须用安全抽象（Vec/切片所有权），不要裸 SendPtr**；「先串行保证正确性，多线程后续加」是 C ABI 桥接稳妥路径 |
| gradle 启动报 `native-platform.dll` 加载失败 / `FileSystemServices` 初始化失败（M3） | gradle 启动需**写临时目录 + 加载 native dll**，受限沙箱权限不足 | **「native 库加载失败」先查环境权限（沙箱/临时目录/写权限），再查配置**；gradle 需 danger-full-access |
| MSVC 编译含中文注释的 .c 报 `C4819` / `C2001`（M4） | 源文件 **UTF-8** 被 MSVC 按系统 code page（**936/GBK**）错解，中文注释被误判为代码 | **MSVC 编译含中文的 UTF-8 源文件必须加 `/utf-8`**；`C4819` 是编码不匹配明确信号；与 AGENTS.md §八.2 同源（UTF-8 在 GBK 环境错位） |
