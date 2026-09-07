# A 组引擎 4 硬缺口改造 — 交付（260907-05，draft→candidate 建议）

> 任务：NEXT_SESSION 开工点 1（多世界数据驱动核心链 4 硬缺口）。swe 域，主会话收敛闭环。
> 设计拍板（用户 confirmed）：D1 命名空间从 settings_name 解析 / D2 注册 API + 告警日志 / D3 JSON 优先 + fallback。

## 改动清单（git 未提交，工作区 diff）

1. **缺口 1**（`worldgen-core/src/blocks.rs`）：BlockRegistry RwLock 化；`id()` 未注册名一次性 stderr 告警（不再静默 AIR）；新增 `register(name)->id`（动态 id = max+1 递增、双检、id_to_name 同步）；`name()` 返回 String（11 处调用点适配）；新增单元测试 register_and_unknown_warn。
2. **缺口 1 API**（`api.rs` + `worldgen_handle.rs`）：新 C ABI `wg_register_block(handle,name)->c_int`（mod 方块注册 Rust 侧前置；mod 侧 JNI 接线仍挂起）。
3. **缺口 2**（`surface_rules.rs` + `worldgen_handle.rs`）：SurfaceBuilder 增 `default_block` 字段 + `set_default_block`；fill_terrain_column / diag_pre_surface_column / build_surface / place_badlands_pillar 四处原 `blocks.id("minecraft:stone")` 改消费 settings.default_block；解析支持字符串与 {Name} 两形态，缺失 fallback stone + 一次性日志；诊断覆盖 env `CORESWAP_DEFAULT_BLOCK`（创建期读一次）。
4. **缺口 3**（`worldgen_handle.rs` + `feature_loader.rs`）：settings_name 支持 `"modid:name[.json]"` → data_ns；noise_settings / density_function / configured_carver 路径按 (data_ns, 短名) 合成；feature_loader 6 处路径改 `data_path()` 按 **feature id 自带命名空间** 解析；无冒号 = minecraft（vanilla 路径逐字节不变）。
5. **缺口 4**（`biome.rs` + `worldgen_handle.rs`）：`load_carvers/load_features` 增 `dir_ns` 参数（id 命名空间不匹配跳过、文件名取短名）；handle 对 vanilla 目录 + mod 目录（data_ns≠minecraft 时）各 load 一次（同名覆盖 = 命名空间 override 语义）；carver_probe 调用点适配。

## 验证记录（Phase 3）

- **cargo test**：6/6 通过（原 5 golden + 新增 register 测试）。
- **workspace 全量 build**：`cargo build --offline --release` Finished（#23 判据；本块未加 enum variant，仍全量）。
- **dll**：target/release/worldgen.dll（含 O1 修复终版），SHA256 = D9085130F035F2B2…（2167808B）；中间版 15B2AA176898A3B4…（O1 修复前，已被取代）。
- **确定性 A/B dump（default_block 行为恒等）**：nether 与 end 各 8×8 chunks（seed 12345，IDK7 载体），A 臂（JSON default_block = netherrack/end_stone）vs B 臂（CORESWAP_DEFAULT_BLOCK=minecraft:stone 强制旧行为）——**end A=B=D45E938E…，nether A=B=1AC5965B…（逐位恒等；hash-文件名对应已按 Get-FileHash 输出顺序核对，2026-09-07 更正此前草稿的对调转录）**。B 臂即改造前语义 → 改造对 nether/end 已 confirmed 对齐态零扰动。
- **§9.7 可比性声明**：A/B 载体 = 纯 Rust fill_chunk_blocks dump（无 Java 装饰层），覆盖面 = nether/end 各 64 chunks 单 seed；与历史 Chunky/存档口径不可比。overworld 无 A/B dump——恒等依据 = 解析值与原硬编码同为 minecraft:stone id（静态同源）+ minecraft 命名空间路径字符串逐字节不变；覆盖面盲区如实声明。
- **降级声明**：wg_register_block / mod 命名空间数据路径无实测消费方（mod 侧接线挂起），仅单元测试 + 路径合成逻辑验证——Degraded 分层，如实标注。

## idk / 边界

- biome 参数文件（MultiNoise 参数集）仍由 biome_params_file 显式传参（既有设计），mod 维度需自带参数文件——未属本块范围。
- base_3d_noise.json legacy 路径仍硬编码 overworld 目录（仅 transpiler legacy 分支消费，overworld 专属语义，未动）。
- 同名 biome 跨命名空间 override 语义（mod 覆盖 vanilla）未实测。
