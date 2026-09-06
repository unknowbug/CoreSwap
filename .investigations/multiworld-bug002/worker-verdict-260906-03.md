# BUG-002 worker 裁决（multiworld-bug002 / worker-verdict 260906-03）

status: draft
角色：core.worker（只读源码分析 + 设计验证命令模板；无 shell，命令由主会话执行）
输入：scout-map-260906-03.md + NoiseChunkGeneratorMixin.java + CppBridge.java + BenchMod.java（全量源码核对）

---

## 1. 核心裁决：wgIsEnd() 反射在生产环境（Forge + Sinytra Connector）会失败

### 1.1 源码事实

- 硬编码字段名：`getDeclaredField("biomeSource")` —— **[已验证-源码引用]** NoiseChunkGeneratorMixin.java:30。这是 **Yarn 名**（源码包名 `net.minecraft.world.gen.chunk.ChunkGenerator`、`net.minecraft.world.biome.source.TheEndBiomeSource` 全套 Yarn 布局，L34/L30）。
- 反射失败路径：`catch (Throwable t) { return false; }` —— **[已验证-源码引用]** L35-37。返回 false = 「不是 end」= **不豁免**。字段查找失败时 `wgBiomeSourceField` 保持 null，每次调用都重试再失败，行为稳定 = 恒 false，不是偶发。静默（无任何日志），与「无诊断输出」现状一致。
- mixin 内字符串常量不被 remapper 重映射：mixin remapping（Connector 的 intermediary→Forge 运行时名变换）重映射的是 **类/字段/方法引用（常量池符号引用）与注解中的 method 签名**（如 L41-46 那串 Yarn 方法描述符，会被 Connector 正确重写），**不会重写方法体内的字符串字面量** `"biomeSource"`。所以 L30 的 Yarn 字段名以字面量形式原样到达生产运行时。**[推测-高置信]**（mixin remap 语义为业界标准行为；Connector 的核心 remap 管线同理只变换引用，文档明示对裸反射+硬编码名不提供保护）。

### 1.2 运行时命名裁决

- **Forge 1.20.1 生产环境成员名 = SRG（`f_XXXXX_`/`m_XXXXX_`），不是 Mojmap 全名**。类名是官方（Mojmap）名，但字段/方法成员是 SRG 编号名（Forge 崩溃栈里 `m_130010_` 类条目即为该机制的直接可观察证据）。`ChunkGenerator.biomeSource` 在生产运行时的真实字段名是 `f_XXXXX_` 形式，**不存在名为 `biomeSource` 的字段** → `getDeclaredField("biomeSource")` 抛 `NoSuchFieldException` → catch → 返回 false。**[推测-高置信，基于 Forge 1.17+ 生产 runtime SRG 成员名的既定机制；未在本机实测]**
- 推论：**在开发环境（loom/ForgeGradle dev runtime = 全 Mojmap 名）反射会成功**——Yarn 字段名 `biomeSource` 与 Mojmap 字段名恰好同拼写（两套映射对该字段同名），dev 里一切正常；**只有生产环境稳定失败**。这精确解释了「生产 38% 差异」类 bug 为何在 dev 测试中不可见。
- Sinytra Connector 不改变此结论：Connector 把 Fabric mod 的 intermediary 引用重映射到 Forge 运行时名（成员同样落到 SRG），字符串字面量仍不重映射。**[推测-中高置信]**

### 1.3 判定

> **反射失败（高置信，机制链完整）：wgIsEnd() 恒返回 false → end 豁免失效 → 末地（minY 0/height 256，与 nether 同形状）落入 nether 分支，被 nether.json 句柄接管生成 → 用 nether 的 noise_settings/surface_rule/biome_params 生成 end 地形 → 大面积错误地形（38% 裸 bedrock 与 nether 床岩地板表面规则输出指纹相容）。**
>
> 证据闭环：① dev 正常 + 生产坏（SRG vs Mojmap 唯一差异）② 失败静默且稳定 ③ 症状分布精确匹配（overworld 不走 wgIsEnd 正常、nether 本身 wgIsEnd=false 本就该接管正常、end 同形状且豁免失效→坏）④ crystal 0.8%/chaos 0.02% 正常 → 它们形状 ≠ 0/256 → 根本不触发 nether 分支（候选 A）。
>
> 注意：此为静态机制推断 [高置信推测]，非运行时已证——第 2 节命令一锤定音。反射「成功」分支若被日志证实，则 38% 另有原因（fallback：nether 句柄数据本身与 end settings 的浮点/结构差异需另查）。

## 2. 廉价验证命令模板（成本升序，主会话复制执行）

### V1（零成本-静态）SRG 成员名核证
```powershell
# Forge gradle 缓存里的 SRG 映射（本机曾构建过 Forge 1.20.1 即存在）
Get-ChildItem "$env:USERPROFILE\.gradle\caches\forge_gradle" -Recurse -Filter "*.tsrg" -ErrorAction SilentlyContinue |
  Select-String -Pattern 'biomeSource' | Select-Object -First 5 -ExpandProperty Line
```
判别式：命中行形如 `biomeSource => f_64609_` 之类（Yarn/Mojmap 名 → SRG 名映射）→ **证明生产运行时该字段不叫 biomeSource，反射必失败（裁决锁定）**；无任何 tsrg 命中 → 本机无 Forge 缓存，跳到 V2。

### V2（一次运行-日志，最直接判别）生产环境末地接管日志
```powershell
# 生产环境启动，进入末地（/execute in minecraft:the_end 或折跃门），飞行加载 ≥10 个 end chunk，退出后：
Select-String -Path "<server>\logs\latest.log" -Pattern '\[Mixin\] populateNoise\(nether\) intercepted chunk\(' |
  Select-Object -ExpandProperty Line
```
判别式：
- **末地探索时间段内出现 `populateNoise(nether) intercepted` 行** → end 被 nether 句柄误接管 = 裁决成立，BUG-002 根因锁定。
- 完全无该行（但 nether 维度内有对应行作对照）→ 反射成功，38% 另有原因，回 Phase 1 重新勘探。
- 连 nether 内也无该行 → netherActive=false / mixin 未生效，是另一层问题，先修再判。

### V3（一次编译-最小复现）给 wgIsEnd 失败路径加一行日志（模板，改动由主会话做）
```java
// NoiseChunkGeneratorMixin.java L35 catch 块改为：
} catch (Throwable t) {
    System.out.println("[CoreSwap] wgIsEnd reflection FAILED: " + t);
    return false;
}
```
生产环境任一 nether 形状 chunk 生成时输出：`[CoreSwap] wgIsEnd reflection FAILED: java.lang.NoSuchFieldException: biomeSource` → 失败直接可见且给出真实运行时字段名线索。建议与 V2 同跑（改动同时消除静默，符合「不吞异常」铁律）。

### V4（支持 §3）mod 维度反例豁免（候选 A）验证
```powershell
# 1) 解出目标 mod 的 dimension_type 形状参数（水晶/混沌对应 mod jar）
Expand-Archive "<modjar>.jar" -Destination "$env:TEMP\bug002-modx" -Force
Get-ChildItem -Recurse "$env:TEMP\bug002-modx\data" -Filter "*.json" |
  Select-String -Pattern '"min_y"|"height"' | Select-Object -ExpandProperty Path, Line
# 2) 生产环境进入该维度加载 chunk 后查拦截日志
Select-String -Path "<server>\logs\latest.log" -Pattern '\[Mixin\] (populateNoise|buildSurface)' | Select-Object -ExpandProperty Line
```
判别式：维度 JSON `min_y/height` ≠ (0,256) 且 ≠ (-64,384)，且该维度探索期间**零** `[Mixin]` 日志行 → 候选 A 成立（形状不匹配 → vanilla 放行，残差 0.02~0.8% 为 vanilla 浮点小噪声）。若 = (0,256) 且出现 nether-intercept 行 → 候选 B（被 nether 误接管），该维度也应大面积损坏。

## 3. 候选 A（mod 维度反例豁免）验证方式小结

- 静态：V4-1（dimension_type JSON 形状参数直接读出）——零运行成本，**首选**。
- 日志：V4-2（该维度零拦截行 = vanilla 放行的直接证据）。
- 交叉：候选 A 成立 ⇔ 该维度残差(0.8%/0.02%)为 vanilla 本底噪声，与 overworld/nether 之外第三维度「正常」现象一致。

## 4. 假设成立性判定 + 放行策略证据支持度

### 「未识别未放行」假设判定
**基本成立，但需精确化为双分支**（对 scout 结论 §6 的修正性细化）：
- 修复建议方向之一曾假设「反射失败 → end 被误接管」——本裁决将其升级为**高置信主因**（§1.3），38% end 差异的主解释。
- mod 维度不是「被接管」而是「未识别 → 形状不匹配 → 意外安全放行」（候选 A）——即「未识别」确实存在，但后果是**侥幸正确**而非损坏；一旦某 mod 维度形状 = 0/256（候选 B），将复现 end 式灾难。**这两个分支共享同一根因：维度识别缺失 + end 豁免依赖裸反射字符串。**

### Phase 2 放行策略两候选的证据支持度
| 候选 | 证据支持度 | 依据 |
|---|---|---|
| **维度白名单**（显式列出 overworld/nether/…） | **低-中**：是现状的显式化，能修 end（把 end 从 nether 分支排除）但延续「每维度硬编码」（scout H8 同族），mod 维度仍是未定义行为； scout §2 ③ 的缺口（判定无维度识别）只是被绕开未闭合 |
| **资源感知自动放行**（registry-key/settings 识别 + 本地数据存在性检查：只有 CoreSwap 资源层含该维度 noise_settings 数据时才接管） | **高**：① 直接消掉反射依赖（用 chunk 所属 Level 的 dimension() ResourceKey 替代 biomeSource instanceof，无字符串脆弱性）② end.json 已在资源层（scout §5），按 key 识别可立即支持 end 正确接管或显式豁免 ③ create_for_dim 已参数化（scout §3，API 就绪），缺的恰是「判定层维度识别」这一环 ④ mod 维度数据不在资源层 → 自动落 vanilla，候选 B 灾难从机制上不可能发生 |

**建议**：Phase 2 以资源感知自动放行为主方向（数据在则接管、数据不在则放行 + 维度 key 日志），白名单仅作为 end 显式豁免的过渡补丁。

## 5. 本产物未做/边界

- 无运行时验证（subagent 无 shell）——V1-V4 全部待主会话执行；反射失败为高置信静态推断，非已证事实。
- 未实测目标 mod（水晶/混沌/aether 类）的 dimension_type 数值——V4-1 补。
- 未核对 `f_64609_` 具体 SRG 编号（不在产物中断言具体编号）。
