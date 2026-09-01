# 草稿：docs/10-timewise-archive.md 时间线条目（2026-09-07）

> 用法：主会话追加到 `versions/1.20.1/docs/10-timewise-archive.md` 2026-09-07 日期下（追加不覆盖）。过程记录口径，每条带状态标注。

---

## 2026-09-07（nether-save-full 课题续）

- ✅ **C2 预加载表数据驱动化**（commit 709b006）：`worldgen_handle.rs` step4 新增 `collect_noise_keys()`，从 surface_rule JSON 构建期收集 noise_threshold 引用 key；overworld 保留静态清单（代码规则无 JSON 源）；nether 静态 6 key 清单删除（E7 手工修复的架构层收尾）。3 连跑 93.8988% 逐位同值无回归；judge C2 CONCERN 闭环。candidate。
- ✅ **P2 矿石归因重大转向——双重 feature 应用**（H_B'，judge PASS 建议 candidate）：发现 `wg_fill_blocks_multi` 内含 carver+feature 阶段（worldgen_handle.rs L442-449，WG_SKIP_CARVER/WG_SKIP_FEATURES env 门控）；存档链路 mixin 只拦 populateNoise + cancel buildSurface，Java CARVER/FEATURES 照跑 → 存档 = Rust+Java features 双跑。消融链：SKIP_FEATURES → 93.8988%→94.4241%（+5508），quartz 4478→2125（ref 1992）/ gold 1525→739（ref 728）/ magma 3814→1979（ref 1533）；SKIP_CARVER 仅再 +370。矿石 ~2.2× 偏高全额归因双跑。修正早前「features 只由 Java 运行一次（无双跑通道）」判断（09 篇原行加注记不删）。遗留：overworld 同路径双跑 vs 99.9% 对齐矛盾 → X1 FEATURELOG 裁决 🔍 进行中；修复方向 judge CONCERN = env 门进程全局，勿全局默认翻转，需句柄/调用级显式 flag。
- ✅ **B2 soul 家族定稿——上轮假设证伪**：V1 证伪「soul_soil 大头在 Java feature 阶段」（Rust 管线 soul_soil 1363 ≈ 存档 1334，缺口 4140 在 Rust 管线内）。V2 探针 180 点三签名：A biome 足迹偏移/收窄（valley 判 nether_wastes，聚簇 x≥3410 边界带）；B soul_soil 子分支失效（entered+selector<0 仍 applied=netherrack）；C floor 侧 soul_sand_layer 分支疑似缺失（组3 entered 0/60）。.b1a 结构差主导；.b1b 噪声值偏离 idk（缺 Java 同点对照）。Java features 对 soul_sand 净回补 +587。下一步：V3 Rust-vs-JSON 结构对拍（零成本最高优先）→ V4 RouterProbe 同点 selector → V5 biome 边界带。🔍 V3-V5 未做。
- 🔍 **X1 FEATURELOG 裁决**：overworld 双跑是否成立及为何对齐 99.9%，进行中，待回填。
- ⚠️ **环境坑 E8/E9**（详录 `.investigations/nether-save-full/nether-save-errors.md`）：E8 = 沙箱下 gradle runServer 提取 worldgen.dll AccessDeniedException → JAVA_TOOL_OPTIONS=-Djava.io.tmpdir 指工作区；E9 = WorldgenRust.dll mtime 因 fs::copy 保留时间戳不可信 → dll 新旧用二进制字符串探测；bin-diag bin 临时挪 src/bin/ 编译（init_vertical 需 pub 化）。
