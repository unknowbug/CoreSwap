# 草稿：knowledge/discovered 条目（260903-14）

> 状态：草稿（core.worker 产出，主会话应用）。
> 归类判断：
> - 通用判据（预加载/运行时查询集合同步 + expect 型查表低频分支缺失 + 大 region sweep 暴露手段）→ **workflow-patterns.md**（工作流/排查判据，非构建工具坑）→ **发现 #26**（当前末条 #25）。
> - E1（`-PcppWorldgenDir` 必带项）→ **build-tooling.md**（gradle run 参数坑，与 #8/#9/#13「参数遗漏静默不生效」同族）→ **发现 #15**（当前末条 #14）。

---

## 一、workflow-patterns.md 末尾追加全文

> 目标文件：`knowledge/discovered/workflow-patterns.md`；插入位置：文件末尾（发现 #25 补充案例之后），追加不覆盖。

## 发现 #26: 预加载/注册表与运行时查询集合同步——expect 型查表的缺失只在低频分支触发，小样本全绿≠无缺失，大 region sweep 是暴露手段（260903-14）

- **现象**：estopt 64×64 sweep 至 ~2304-2560 chunk 处 panic：`surface_rules.rs:505 missing noise sampler`（260903-12 课题 #2）。此前所有小样本对比/回归全绿——同一二进制在 64×64 sweep 内稳定跑过 ~2300 chunk 才首次命中崩溃分支。
- **根因（机制）**：overworld 预加载 noise key 静态清单（`worldgen_handle.rs` L272）缺 `minecraft:badlands_pillar_roof`，`place_badlands_pillar`（`surface_rules.rs:1372`）运行时 `get_noise` → `expect` panic。**预加载集合（启动期建表）与运行时查询集合（surface rule 里的 expect 型查表调用点）是两份独立维护的清单**——新增查表调用点时没有同步预加载来源，静态清单没有「运行时全集」的机械校验，缺项只能等运行时 expect 爆。触发面极窄（仅 eroded_badlands biome 列且侵蚀度 e>0），是典型低频分支。
- **定位**：panic 点反查调用链（expect ← get_noise ← place_badlands_pillar ← 预加载清单），清单 grep 缺失 key 即闭合——expect panic 自带 key 名，链条短；难的不是定位而是**让崩溃先发生**（小样本永远到不了该分支）。
- **修复**：预加载清单补一行。
- **教训（可复用判据）**：
  1. **新增任何 expect 型查表调用点（get_noise / get_rule / get_* → expect/unwrap）必须同步预加载/注册表来源**——同一 PR/同一 commit 内两处一起改；更优做法是启动期校验「运行时引用的 key ⊆ 预加载集合」（机械校验替代人肉同步）。
  2. **expect 型缺失是小样本测试的盲区**：低频分支（罕见 biome/条件组合）触不到就测不到，「回归全绿」对缺失类缺陷**无证据力**——绿 ≠ 无缺失。
  3. **大 region sweep 是暴露手段**：存档口径级大样本（数千 chunk）覆盖低频分支；对 expect 密集的代码路径，sweep 应作为常规回归而非一次性验证（本次正是常规 sweep 抓到）。
- **同族**：#12（哨兵结论须配已知值哨兵点——「测试绿」同样要有覆盖面背书）、#14（大 region 预生成三件套前置——sweep 可执行的工程前提）。

---

## 二、build-tooling.md 末尾追加全文（E1）

> 目标文件：`knowledge/discovered/build-tooling.md`；插入位置：文件末尾（发现 #14 之后），追加不覆盖。

## 发现 #15: gradle run 存档口径照抄历史 run 完整参数清单——裁剪属性列表会裁掉历史踩坑后的必带项（-PcppWorldgenDir）（260903-14）

- **现象**：不带 `-PcppWorldgenDir` 跑 `-PcppReplace=true -PreadWorldProbe=true`，server started 即抛 `IllegalStateException: worldgen-data not found in mod resources`（CoreSwapFixHelper.extractWorldgenDir:48），服务器立即停止。
- **根因**：jar 内资源布局 `worldgen-data/{minecraft, blocks.json, …}`（minecraft 直下），而 marker 检查路径是 `wgDir/data/minecraft/worldgen/noise_settings/overworld.json`（多一层 `data/`）——资源解压路径与 marker 路径两条布局约定不同步，解压分支的 marker 永远不存在 → 必然二次抛异常。**解压路径本身是死路**，历史 run 全部靠显式 `-PcppWorldgenDir=<工作区 data/worldgen>` 绕过解压。
- **定位**：读 CoreSwapFixHelper.java marker 路径 + `Get-ChildItem src/main/resources/worldgen-data` 对照布局；再查历史 run 日志确认全部显式传参绕过——「为什么历史没炸」的答案是历史从来没走过解压分支。
- **修复**：run 命令补 `-PcppWorldgenDir=...`（workaround）；资源布局与 marker 不一致未改，列为升级点。
- **教训**：**跑存档口径 run 照抄历史 run 的完整参数清单，不要凭 build.gradle 属性列表自行裁剪**——属性列表只声明「存在」，不声明「必带」；裁掉的可能是历史踩坑后的必带项。同族：#8（gradle -P→-D 映射遗漏静默不生效）、#9（缺映射行静默不生效）——本条补「不能反向从属性列表推断可省略项」维度。根治方向（升级点）：marker 路径与资源布局对齐，或解压失败 fail-fast 时提示带 `-PcppWorldgenDir`。

---

## 三、INDEX.md 追加文本

> 目标文件：`knowledge/INDEX.md`；两处均为在既有行末追加（用 ` + ` 连接，格式随既有行）。

1. **「工作流模式」行（workflow-patterns.md，现 L23）行末追加**：

   ```
   + 预加载/注册表与运行时查询集合同步——新增 expect 型查表调用点必须同步预加载来源，缺失只在低频分支触发，小样本全绿≠无缺失，大 region sweep 是暴露手段（发现 #26，260903-14）
   ```

2. **「构建/工具链坑」行（build-tooling.md，现 L20）行末追加**：

   ```
   + gradle run 存档口径照抄历史 run 完整参数清单——裁剪属性列表会裁掉历史踩坑后的必带项（-PcppWorldgenDir，资源布局与 marker 路径不一致的绕过项）（发现 #15，260903-14）
   ```
