# Scout：外部 mod 调研（Voxy / Distant Horizons / 史诗地形）

- 日期标签：260908-02
- 目标版本：Minecraft 1.20.1（Forge 为主，兼顾 Fabric）
- 用途：CoreSwap（Rust worldgen 接管 mod）兼容性评估的外部事实输入
- 方法：web_search 公开资料；未查到的项如实标「未查到」，不做推断
- 置信度：全部 draft（外部网页信息，未做源码级验证）

---

## 1. Voxy（Vulkan 体素/LOD 渲染器）

### 机制事实（带来源）
- 定位：An LoD rendering mod for minecraft，Vulkan 渲染的 LOD/远视野渲染 mod。
  - GitHub（代码镜像/仓库）：https://github.com/m3t4f1v3/voxy
- 平台：原版 Voxy 面向 **Fabric**（Modrinth 条目 voxy：https://modrinth.com/mod/voxy）。
  **1.20.1 Forge 无官方版**；Forge 使用依赖第三方移植 **XingPeng-Pixel/voxy-1.20.1**（如 `Voxy v1.20.1-forge-0.2.6.4-alpha`）：
  - https://github.com/XingPeng-Pixel/voxy-1.20.1/releases/tag/v0.2.6-alpha-build.7
  - https://github.com/XingPeng-Pixel/voxy-1.20.1/releases
- 与 Distant Horizons 的关系：定位为 DH 的替代/竞争者（同为 LOD 远视野渲染），社区有直接对比文：
  - https://gurugamer.com/pc-console/distant-horizons-vs-voxy-the-ultimate-lod-mod-showdown-for-minecraft-26068/amp
  - https://vortexgaming.io/en/postdetail/623690
  - MC百科收录 Voxy：https://www.mcmod.cn/class/diff/0-366887.html

### worldgen 触碰面
- **Voxy 本体：纯渲染侧（client-side）**——公开资料一致描述为 LOD 渲染器，其 LOD 数据来源依赖真实 chunk（玩家走过/已生成区域），公开资料中未见本体自带独立 worldgen。
- **关键例外：Voxy WorldGen 生态 addon（iSeeEthan/voxy_worldgen_v2，Modrinth "Voxy WorldGen"）**——这是一个**独立 addon，显式进入 worldgen 领域**（名称即 voxy_worldgen；issue 中可见 mixin force-disable 等侵入行为）：
  - https://github.com/iSeeEthan/voxy_worldgen_v2/issues/20
  - https://github.com/iSeeEthan/voxy_worldgen_v2/issues/38
  - https://modrinth.com/mod/voxy-worldgen/versions
  - ⚠️ 该 addon 的 LOD 生成机制细节（是否走服务器正常 chunk 管线）本次未查到一手文档，标「未查到细节」。

### client/server 归属
- Voxy 本体 = **client-side**（渲染器）；另有 **VoxyServer**（CurseForge：https://www.curseforge.com/minecraft/mc-mods/voxyserver/files/all?page=1&pageSize=20&showAlphaFiles=hide&sortBy=dateCreated&sortOrder=desc）服务端配套组件存在。

### mixin 注入面 / 已知兼容性问题
- mixin 注入面细节（具体类/方法）：**未查到一手清单**。旁证：voxy_worldgen_v2 issue 中出现「Force-disabling mixin」日志，说明其生态存在对 vanilla/其他 mod mixin 的覆盖行为（https://github.com/iSeeEthan/voxy_worldgen_v2/issues/20）。
- 已知兼容性问题：以 Forge 移植版 alpha 性质为主（版本号即 alpha，https://github.com/XingPeng-Pixel/voxy-1.20.1/releases）；具体冲突记录未查到系统性列表。

---

## 2. Distant Horizons（DH）

### 机制事实（带来源）
- Modrinth 条目（forge/fabric 双 loader，1.20.1 有版本线，如 `3.0.2-b-1.20.1`（Beta，forge/fabric）、`2.3.4-b-1.20.1`）：
  - https://modrinth.com/mod/distanthorizons
  - https://modrinth.com/mod/distanthorizons/version/3.0.2-b-1.20.1
  - https://modrinth.com/mod/distanthorizons/version/MhxUKxWI

### LOD 生成机制（CoreSwap 关键问题：走服务器 worldgen 管线还是自带复刻？）
- **DH 两条路径并存，由配置项 `distantGeneratorMode` 控制**（API javadoc 为一手证据）：
  - API `IDhApiWorldGenerationConfig.distantGeneratorMode()`：
    https://distant-horizons-team.gitlab.io/distant-horizons/com/seibel/distanthorizons/api/interfaces/config/both/IDhApiWorldGenerationConfig.html#distantGeneratorMode()
  - 模式枚举 `EDhApiDistantGeneratorMode`（含 `FEATURE_GENERATOR` / `UNLOADED` / `INTERNAL` 等模式名，具体语义见枚举页）：
    https://distant-horizons-team.gitlab.io/distant-horizons/com/seibel/distanthorizons/api/enums/worldGeneration/EDhApiDistantGeneratorMode.html#complexity
  - `IDhApiWorldGenerator.generateChunks(...)` API 允许第三方**注册自定义 LOD 生成器接管 DH 的远处生成**：
    https://distant-horizons-team.gitlab.io/distant-horizons/com/seibel/distanthorizons/api/interfaces/override/worldGenerator/IDhApiWorldGenerator.html#generateChunks(int,int,int,byte,com.seibel.distanthorizons.api.enums.worldGeneration.EDhApiDistantGeneratorMode,java.util.concurrent.ExecutorService,java.util.function.Consumer)
- 语义要点（来自 wiki/API/社区，综合判断为 candidate 级）：
  - DH 在**单机/集成服务器**场景下可驱动 vanilla worldgen 管线生成 LOD 数据（FEATURE_GENERATOR 模式：跑 vanilla 特性生成到 LOD 精度）——即**该模式下会触发正常 chunk 生成管线，从而受 CoreSwap 接管影响**；
  - 多人服务器场景：**服务端不装 DH 时，客户端 DH 侧的「未探索远处」没有数据可渲染**（DH 官方 Server Owners wiki 明确讨论 server 侧预生成/How it works），说明客户端不会凭空复刻 worldgen，远处 LOD 要么来自服务器/预生成数据，要么走本机 worldgen 管线：
    - https://gitlab.com/distant-horizons-team/distant-horizons/-/wikis/1-user-guide/1-frequently-asked-questions/5-server-owners/Server-Owners/diff?version_id=10c33cb7aee6ab7ad3421ce348122689a650a4c7
  - INTERNAL 模式存在「DH 内部简化生成器」的概念（即 DH 自带一个低保真生成路径，不等于 vanilla 复刻）——**具体实现细节未查到一手源码级文档**，标「未查到细节」。
- 对 CoreSwap 的含义（推导，非事实）：CoreSwap 接管 worldgen 后，DH 在「本机驱动 vanilla 管线」模式下的 LOD 应自动继承 Rust 结果；若 DH 走 INTERNAL 简化生成或第三方接管生成器（API override），则**绕过** CoreSwap。

### 与第三方 worldgen mod 的兼容/冲突记录
- DH 官方 wiki 有 server owners 指南涉及预生成（见上 GitLab wiki 链接）。
- 社区问答（answeroverflow，DH Discord 存档）多条讨论 LOD 生成与服务器行为：
  - https://www.answeroverflow.com/m/1333611137304100903
  - https://www.answeroverflow.com/m/1374435699394085015
  - https://www.answeroverflow.com/m/1381651632282009761?focus=1382012113102438461
  - https://www.answeroverflow.com/m/1218128202581413929
- DH 与 Tectonic/Terralith 的直接冲突记录：本次未搜到 DH 侧专门 issue；**Terralith 侧倒是记录了 Tectonic+Terralith 组合自身的冲突**（与 DH 无关，见 §3）。

### 1.20.1 Forge 成熟度
- 1.20.1 有持续更新的 forge/fabric 构建（3.0.x 为 Beta 线，Modrinth 显示 4-5 年前发布线、2 个月前仍在更新，页面时间口径以来源为准）：
  - https://modrinth.com/mod/distanthorizons/version/3.0.2-b-1.20.1
  - https://modrinth.com/mod/distanthorizons/version/MhxUKxWI
- 「成熟度」结论：有活跃版本线但 3.0.x 标 Beta（candidate 级，无 crash 统计数据）。

---

## 3. 史诗地形（= **Epic Terrain / ETN**，非 Tectonic/Terralith）

### 名称确认
- 中文名「史诗地形」对应英文 mod = **[ETN] Epic Terrain**（MC百科收录页）：
  - https://www.mcmod.cn/class/15808.html
- 它**不是** Tectonic / Terralith / Terra——三者是独立 mod（Terralith/Tectonic 为 Stardust Labs 数据包型 worldgen mod；此区分证据见 §2 中 Terralith issue 链接与 Tectonic 社区视频 https://www.snm0516.aisee.tv/video/BV1oTszz7EuL/ ）。

### 机制事实
- **形态：世界生成数据包（datapack 形态发布在 Modrinth datapack 分类）**，MIT 协议，支持 1.19–1.21.x（含 1.20.x）：
  - https://modrinth.com/datapack/epicterrain
  - https://modrinth.com/datapack/epicterrain/version/0.0.9
  - 社区称其为「"国产" epicterrain 地形数据包」：https://www.bilibili.com/video/BV1rf421q7ox/
- 机制推断（数据包型 worldgen = vanilla datapack 机制覆盖 noise_settings/density function/biome，**candidate 级，未做文件级验证**）：数据包形态本身即说明其通过 vanilla 数据驱动 worldgen 数据（noise_settings/density_function/biome JSON）实现，而非字节码 mixin。
  - MC百科「改动对比」页可佐证其随版本改动的是数据内容：https://www.mcmod.cn/class/diff/0-412323.html
  - ⚠️ 具体是否含自定义 biome/climate 点位、密度函数清单：**未查到一手 changelog/文件清单**，标「未查到细节」。
- 官方/社区兼容组件：存在「Epic Terrain Compatible（史诗地形兼容）」附属（https://www.mclists.cn/mod/sc69VpnK/epic-terrain-compatible.html ，Modrinth 侧 https://www.mcmod.cn/class/18286.html 为 MC百科收录），并有「史诗地形"全新兼容"」宣传视频（https://www.snm0516.aisee.tv/video/BV1etdAY2E62/ ）——说明其生态**主动维护与其他 mod 的兼容**。
- 1.20.1 Forge 可用性：Modrinth 兼容矩阵含 1.20.x；数据包/世界生成型内容通常 loader 无关，但**有无专门 Forge mod 打包版未确认**，标「未查到」。
- 与 worldgen 接管类 mod 的冲突记录：未搜到 Epic Terrain 与 CoreSwap 类接管 mod（worldgen 重写）的直接冲突 issue——**未查到**。风险点（推导）：数据包覆盖 noise_settings/density_function 的场景下，接管 mod 若按 vanilla 数据生成，需以 Epic Terrain 的 JSON 为准才保持地形一致（同 DH+数据包地形 mod 的一般性问题）。

---

## 汇总（对 CoreSwap 兼容性评估的直接输入）

| mod | loader(1.20.1) | 侧 | 触碰 worldgen? | 对 CoreSwap 的暴露面 |
|---|---|---|---|---|
| Voxy 本体 | Fabric（Forge=第三方移植 alpha） | client 渲染 | 否（纯渲染） | 低：只消费已生成 chunk |
| Voxy WorldGen addon | 存在（iSeeEthan） | 未确认 | **是（显式 worldgen addon）** | 高：需单独调研其生成路径 |
| Distant Horizons | Forge+Fabric 官方 | 两端均可 | **模式依赖**：FEATURE_GENERATOR 走 vanilla 管线（被接管影响）；INTERNAL/API override 绕过 | 高：distantGeneratorMode 决定接管是否生效 |
| Epic Terrain (ETN) | datapack 形态（1.20.x 支持） | 数据驱动 | 是（数据包覆盖 worldgen JSON） | 高：CoreSwap 需读取其 JSON 数据才保地形一致 |

### 未查到清单（不猜）
- Voxy 本体 mixin 注入面一手清单；Voxy 原作者身份的权威确认（m3t4f1v3 为 GitHub 仓库/镜像署名，未与 Modrinth 作者页交叉验证）。
- Voxy WorldGen addon 的 LOD 生成是否走服务器 chunk 管线的机制细节。
- DH INTERNAL 简化生成器的实现细节；DH 与数据包地形 mod 的专门冲突 issue。
- Epic Terrain 的自定义 biome/密度函数清单；1.20.1 Forge 专属打包版。
