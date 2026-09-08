# scout-1a：MC 1.21.6 Java 参照源可得性勘探

- 日期标签：260908-08（探查任务，产物随父工作块）
- 角色：recode.scout（只读勘探；未下载、未安装、未访问任何工作区外路径）
- 置信度：**draft**（全部基于工作区内静态文件核对 + 一次 web 搜索；无运行验证）

---

## 1. 现状 1.20.1 参照链路图（已核实）

```
一手 Java 参照权威链（1.20.1 现役）
─────────────────────────────────
runtime/1.20.1/java/                     ← gradle/loom 探针工程「worldgen-bench」
 ├─ build.gradle       fabric-loom 1.10.5
 │    minecraft com.mojang:minecraft:1.20.1
 │    mappings  net.fabricmc:yarn:1.20.1+build.10:v2
 │    modImplementation fabric-loader 0.15.11 + fabric-api 0.92.0+1.20.1
 │    Java 17（toolchain 17，options.release=17）
 │    loom.runs { server / client }：~80 个 -P 属性 → -D 系统属性的探针/参数通道
 │    processResources：从 E:/PYTHON/CoreSwap/target/release/worldgen.dll 同步打包 Rust dll
 │    -XX:ErrorFile → run/hs_err_%p.log
 ├─ gradle.properties  本地代理 127.0.0.1:9199；org.gradle.java.home=D:/Program Files/Java/jdk-17.0.12
 │    （注释自证：旧值指向已删除的 E:/python/MC/tools/jdk17 —— 外部路径残留已被清除）
 ├─ settings.gradle    pluginManagement: maven.fabricmc.net + mavenCentral + gradlePluginPortal；include 'content-test'
 ├─ src/main/java/wg/  CppWorldgen + bench/*（~25 个探针类：BlockProbe/DensityProbe/RouterProbe/WorldGenBench…）
 │    + bench/mixin/*（~20+ Mixin/Accessor 打进 vanilla 采集点）
 └─ run/               服务端运行目录（server.properties level-seed / world 由探针流程管理）

参照数据产出通道：
runtime/1.20.1/java 探针（-PbenchOut=E:/PYTHON/CoreSwap/versions/1.20.1/data 默认）
 → versions/1.20.1/data/vanilla_<seed>_<dim>_<x>_<z>.blocks / *_density_*.txt 等
 → versions/1.20.1/data/mc_src_extract/  ← 反编译/映射后 Java 源提取（net/minecraft/**，供静态参照）
 → versions/1.20.1/data/C2ME-fabric/      ← 第三方性能参照源码（git clone 落 data 内）

gradle 缓存（workspace 内，可复制布局给 1.21.6）：
.gradle-home/caches/fabric-loom/1.20.1/
 ├─ minecraft-merged.jar / minecraft-client.jar / minecraft-extracted_server.jar / minecraft-server.jar
 ├─ intermediary-v2.tiny
 └─ net.fabricmc.yarn.1_20_1.1.20.1+build.10-v2/   ← mappings 目录
（本次未在 loom cache 内找到 *sources* jar——sources 由 gradlew genSources 生成/或按需再生成，
 mc_src_extract/ 即其产物的提取版）
```

要点：**官方 jar + yarn mappings 由 loom 自动经 maven.fabricmc.net/mojang 拉取**，落 `.gradle-home/caches/fabric-loom/`；Java 源参照 = `gradlew genSources`（反编译 sources jar）→ 人工提取到 `versions/1.20.1/data/mc_src_extract/`。gradle.properties 走本地代理 9199 —— 1.21.6 下载同样依赖此通道可用。

## 2. 1.21.6 缺口清单

| # | 缺口 | 说明 |
|---|------|------|
| G1 | **探针工程版本四元组** | build.gradle 需改：minecraft `1.21.6`、mappings `yarn:1.21.6+build.?<v2>`（build 号需构建时查 maven.fabricmc.net，web 搜索确认 yarn 1.21.6 系列映射存在）、fabric-loader（1.21.6 需较新版本）、fabric-api 对应 1.21.6 build |
| G2 | **Java 21** | 1.21.x 要求 Java 21；现 gradle.properties 钉死 jdk-17.0.12 + toolchain 17 + release 17 → 需新 JDK21 路径 + toolchain/release 升 21（Mixin/class version 不兼容 17） |
| G3 | **loom 版本** | 现 1.10.5 较新，初步判断支持 1.21.6（loom 1.7+ 覆盖 1.21 系列）；需构建时确认，不匹配则升 loom |
| G4 | **Mixin/探针代码适配** | src/main/java/wg/bench/** 钉在 1.20.1 yarn 名（NoiseChunkGeneratorMixin 等）：1.21.6 worldgen 类名/签名/注册表结构有变化（1.21.x 系列多次 worldgen 重构），全部 mixin/probe 需逐个迁移——这是最大工作量项 |
| G5 | **runServer 参数通道** | build.gradle 里 benchVmArgs 探针属性通道与版本无关，可平移；但底层被探针的 vanilla 行为变化需重新验证 |
| G6 | **Rust dll 打包路径** | processResources 硬编码 target/release/worldgen.dll——1.21.6 若按 AGENTS 架构新建 versions/1.21.6/rust 薄壳，dll 路径可能同名复用（workspace 根 target 不分版本，**多版本并存时该路径冲突需设计**） |
| G7 | **参照数据目录** | 需建 versions/1.21.6/data/（含新 noise_params/biome_params JSON 从 1.21.6 jar 数据提取），与 1.20.1 数据隔离 |
| G8 | **vanilla-server 1.21.6** | runtime/1.20.1/vanilla-server/server.jar 对应物需另获取（官方 server jar，loom 亦会拉 merged jar，可能够用） |

## 3. 建议获取通道（★ = 需用户批准项）

1. ★ **新建探针工程**：建议 `runtime/1.21.6/java/` 复制 1.20.1 布局，改 build.gradle 四元组（G1/G3）+ gradle.properties JDK21（G2）。首次 `gradlew build` 需**联网**（本地代理 127.0.0.1:9199 或直连）下载 1.21.6 jar + yarn + loom 依赖 → 用户批准网络使用。
2. ★ **JDK 21**：本机未见 JDK21 安装记录（gradle.properties 只有 jdk-17）。需用户提供/批准安装 JDK 21（如 Temurin 21）。
3. **sources**：`gradlew genSources`（反编译，通道同上，无额外审批，属 1 的执行细节）；产物提取到 `versions/1.21.6/data/mc_src_extract/`。
4. **yarn build 号 / fabric-api 版本**：构建时查 https://maven.fabricmc.net/ （web 检索确认 [yarn 1.21.6 映射已发布](https://docs.fabricmc.net/develop/porting/mappings/index.html)，[1.21.6 系列 types 包存在](https://www.npmjs.com/package/@minecraft-types/yarn-1.21.6-rc1)；精确 build 号以 maven 目录为准，未联网核实）。
5. ★ **工作区外路径铁律重申**：外部获取一律落 workspace 内（runtime/1.21.6、versions/1.21.6），不触碰 E:\PYTHON\MC 等历史路径。

## 4. scout 备注

- 未做任何下载/安装/外网资源写入（角色边界）。
- 未运行 gradle（subagent 无 shell 执行探针工程构建的授权），G3 loom 兼容性是静态判断，待主会话实际构建确认。
- 已核实 gradle.properties 注释中 E:/python/MC 残留已被 260907-07 清理，现役 JDK 路径为本机 D 盘 jdk-17（1.21.6 不可用，见 G2）。
