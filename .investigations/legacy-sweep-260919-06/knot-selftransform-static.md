# Knot/Sponge transformer 拒绝自变换的源码级机制（260918-03 open① 静态面）

- status: draft（静态推断，未运行时验证；推断等级 = 机制方向，带一手字节码/源码证据的分支定位）
- 日期标签: 260919-06（宿主锚 2026-09-19；任务方 backlog-map.md §7 open①）
- 验证分层: **Degraded（静态审查）**——sponge-mixin 侧为字节码 javap 判读（无 sources jar），fabric-loader 侧为一手 sources jar
- 声明: 本文件只做纯静态文件核对，未跑任何构建/游戏 run，未采集动态 cause 链

## 1. 本地一手材料定位

| 材料 | 路径 | 性质 |
|---|---|---|
| fabric-loader 0.15.11 **sources** | `%USERPROFILE%\.gradle\caches\modules-2\files-2.1\net.fabricmc\fabric-loader\0.15.11\ee8baabf67a4b0489227a2c2bf8618295ec1d401\fabric-loader-0.15.11-sources.jar` → 解压 `.tmp\fabric-loader-src-260919-06\` | 一手源码 |
| sponge-mixin 0.13.3+mixin.0.8.5（**无 sources jar**，仅 class jar） | `...\net.fabricmc\sponge-mixin\0.13.3+mixin.0.8.5\9527e6b0...\sponge-mixin-0.13.3+mixin.0.8.5.jar` → 解压 `.tmp\sponge-mixin-cls-260919-06\` | 一手**字节码**（javap -c 判读，禁直信单反编译输出——本件只有字节码可读，作 Degraded 证据声明） |
| 版本归属 | gradle cache 内 fabric-loader 仅有 0.15.11 一个版本、sponge-mixin 仅有 0.13.3+mixin.0.8.5 一个版本 ⇒ 两棵树（1.20.1/1.21.6）解析到的即此版本组合（间接证据，未读 loom lock file 精确核对） | 推断 |

解压副作用登记（§9.8）：derived 类（新增 `.tmp\fabric-loader-src-260919-06\`、`.tmp\sponge-mixin-cls-260919-06\` 两个新临时目录，临时区唯一区纪律合规），无 in-place 覆盖，无需逆。

## 2. 源码定位（file:line 截引，全部本 session 实读）

### 2.1 Knot 侧（一手 sources）

- `net/fabricmc/loader/impl/launch/knot/KnotClassDelegate.java:414-429` `getPostMixinClassByteArray`：
  - `:422` `return getMixinTransformer().transformClassBytes(name, name, transformedClassArray);`——**所有**经 Knot 载入的类（含 mixin 类本体）都无差别送进 Sponge transformer；Knot 自身没有「mixin 包拦截/放行」分支（`canTransformClass` :468-… 只排除个别 loader 内部类，不排除 mixin 包）。
  - `:423-427` `catch (Throwable t) { String msg = String.format("Mixin transformation of %s failed", name); … throw new RuntimeException(msg, t); }`——**这正是首证日志里那句顶层消息的出处**；cause `t` 一直挂在 RuntimeException 上，真正吞掉它的是应用侧 `BlobProbe.java:46`（只打印 `+ t` 顶层消息）。
- `:417`/`:443` `if (!transformInitialized || !canTransformClass(name))`——transform 初始化后普通类与 mixin 类走同一条 `transformClassBytes` 路径，无自载豁免。

### 2.2 Sponge 侧（一手字节码 javap 判读，`org/spongepowered/asm/mixin/transformer/MixinProcessor.class`）

- `MixinProcessor.applyMixins`（synchronized）内，偏移 ~415-430：当被变换类被识别为「已注册 mixin 类 / mixin 包成员」时，调用 `getInvalidClassError(String, ClassNode, MixinConfig)` 取消息文本，然后 `new IllegalClassLoadError(msg)` **athrow**。
- `MixinProcessor.getInvalidClassError` 内三条消息分支（常量池字符串直读）：
  1. `Illegal classload request for %s. Mixin is defined in %s and cannot be referenced directly`（类命中某 config 的已注册 mixin 列表）
  2. `Illegal classload request for accessor mixin %s. The mixin is missing from %s which owns package %s* and the mixin has not been applied.`（accessor 变体）
  3. `%s is in a defined mixin package %s* owned by %s and cannot be referenced directly`（包级兜底）
- 另在场的兄弟分支（本例不命中，但同属「拒绝」面）：`ReEntrantTransformerError`（`Re-entrance detected…`，applyMixins 两处）与 `ClassAlreadyLoadedException`（`… was already classloaded`，audit 强载路径）——列此为排除参照。
- 对应上游 sponge-mixin/MixinProcessor 源码方法 `getInvalidClassError`（0.8.x 系同名方法存在公开源码，语义一致——此处不引外部 file:line，避免无一手核对编号）。

### 2.3 应用侧（1.20.1 树，本仓库一手源码）

- `versions/1.20.1/java/src/main/java/wg/bench/BlobProbe.java:39,42`：`Class.forName("wg.bench.mixin.BlobProbeMixin")`——全 run 唯一一次加载 mixin 类本体。
- `:46`：`catch (Throwable t) { System.out.println("... stats read failed: " + t); }`——只打印顶层 `RuntimeException: Mixin transformation of ... failed`，`.getCause()` 链未展开 ⇒ cause 被吞的落点确认。

## 3. 机制链（静态推断）

1. mixin 注入（织入目标类）不需要加载 mixin 类本体——所以启动期 `handler active` 等全部正常（与首证 :137-139 一致）。
2. stats 读取时 `Class.forName("wg.bench.mixin.BlobProbeMixin")` 走 KnotClassLoader → `KnotClassDelegate.getPostMixinClassByteArray`（:414）→ `:422` 无差别送 `IMixinTransformer.transformClassBytes`。
3. Sponge `MixinProcessor.applyMixins` 识别出该类名是**已注册 mixin 类**（wg 的 mixin config 已注册 `wg.bench.mixin.BlobProbeMixin`）→ 走 `getInvalidClassError` 分支 1：**「Mixin is defined in … and cannot be referenced directly」→ 抛 `IllegalClassLoadError`**。即：mixin 类本体在 mixin 体系里被定义为「只能被织入引擎以字节码方式消费的中间体，禁止直接 classload 引用」——这是一条**显式设计性拒绝**，不是兼容性/字节码版本问题（候选 B 证伪与此自洽）。
4. `IllegalClassLoadError` 传回 Knot，`:423-427` 包装成 `RuntimeException("Mixin transformation of wg.bench.mixin.BlobProbeMixin failed", t)`。
5. `BlobProbe.java:46` 只打印顶层消息 ⇒ 日志里看不到 `IllegalClassLoadError: Illegal classload request for wg.bench.mixin.BlobProbeMixin …` 这层 cause。

**最可能 cause 一句话**：`Class.forName` 直接引用已注册 mixin 类触发 Sponge Mixin 的设计性拒绝——`MixinProcessor` 对「直接 classload mixin 类本体」抛 `IllegalClassLoadError`（"cannot be referenced directly"），经 `KnotClassDelegate.java:427` 包装、被 `BlobProbe.java:46` 吞掉 cause。

## 4. 推断等级与验证边界声明

- **证据等级**：fabric-loader 侧 = 一手 sources（强）；sponge-mixin 侧 = 一手 jar 的 javap 字节码判读（Degraded，未读 sources/上游 git，未动态采集）；版本归属（loader 0.15.11 + mixin 0.13.3）= cache 单版本推断（弱）。
- **推断等级**：机制方向（⚠️ 未运行时验证）。字节码只能证明「存在该拒绝分支且消息与现场吻合」，**未证明**本例实际走的正是分支 1 而非分支 3（包级兜底）——两者消息不同但结局相同（IllegalClassLoadError），对结论无影响。
- **定论条件（动态面，超出本任务范围）**：临时补一行 `t.getCause()` 打点跑一次最小 run，或断点 `MixinProcessor.getInvalidClassError`，即可把本推断升 candidate。修复方向不受影响：计数器外移出 mixin 类（1.21.6 已照此修复）。

## 5. 结论

- open① 的静态面**闭合到分支级**：拒绝不在 Knot（Knot 只是包装层 :423-427），而在 Sponge `MixinProcessor` 的「mixin 类禁止直接引用」设计分支（getInvalidClassError → IllegalClassLoadError）。
- 顶层消息出处 `KnotClassDelegate.java:424`；cause 吞点 `BlobProbe.java:46`；根拒绝点 `MixinProcessor`（字节码级定位，无 file:line 可引——类文件反汇编，偏移 ~415）。
