# CoreSwap 验证协议（Verification Protocol）

> 融合：Anchorlaw Protocol v0.4（语言无关验证协议）+ RE-Framework（逆向方法论）+ CoreSwap 工程实践
> 适用：**一边逆向（Java 源码/javap → 还原 C++）一边编程（C++ 实现 + 逐位对齐验证）**的混合工程
> 来源协议：[Anchorlaw v0.4](E:\PYTHON\Anchorlaw\spec\protocol-v0.4.md)、[RE-Framework](E:\PYTHON\RE-Framework\CLAUDE.md)

---

## 0. 关键措辞与全称量词纪律

本协议中 MUST / MUST NOT / SHOULD / MAY 按 RFC 2119 解释。

**全称量词纪律**：任何全称声称（任何/全部/每个/从不）MUST 要么 (a) 在知识库中有验证记录，要么 (b) 明确限定当前实现范围。未验证的全称声称违反第二律（可证伪性）。每个全称声称在本文 §11 审计。

---

## 1. 第一律（可检验）：@anchor.test

> 任何声称都必须有可验证的实践锚点。

### C++ 注解语法（行注释，inert——删除协议零代码改动）

```cpp
// @anchor.test("clampedMap 插值映射对齐 Java map2 语义", source="probe:block_probe!densityBuf#001")
static double clampedMap(...);

// @anchor.idk("结构 Beardifier 密度修正未实现", source="static: 2026-08-08 -288 岛缺失根因确认")
class InterpolatedDF : public DensityFunction { ... };
```

### 规则

- `@anchor.test` **MUST 带 `source` 字段**（验证载体引用）。格式：`<载体>:<工具>!<条目>#<序号>`，如 `probe:block_probe!densityBuf#001`、`java:ChunkNoiseSampler.java:177`、`trace:cns-est#002`。
- `@anchor.idk` 不需要 source，但 SHOULD 带 `source`（说明这个未知从哪来——「没看过」还是「看了但无法确定」）。
- **验证载体 = 独立 probe binary**（不是运行时 assert）：block_probe（逐位对比）、router_probe（分量反射）、density_probe（密度反射）、got_export（densityDump）。载体输出 MUST 是可观测、可复现的（seed + 坐标 + 参照文件）。
- **@anchor.idk 与 TODO 的区别**：TODO = 知道要做什么还没做；idk = 还不知道正确行为是什么。

### 使用范围（工程判断：只标核心敏感函数）

| 优先级 | 函数类别 | 示例 | 必标 |
|---|---|---|---|
| P0 | 插值/随机/映射（防回归敏感） | clampedMap、InterpolatedDF、FlatCacheDF、Xoroshiro | ✅ MUST |
| P1 | 逆向还原点（Java→C++ 语义易错） | aquifer apply、surface rule、ore_vein | ✅ MUST |
| P2 | 聚合/流程（验证链覆盖） | buildGrid、fillOneChunk | SHOULD |
| P3 | 纯辅助 | 日志、格式化 | MAY |

---

## 2. 第二律（可证伪）：@anchor.idk + staleness

- `@anchor.idk` 是诚实声明认知边界，**主动邀请实践反馈**。
- **staleness**：idk 创建超过 **90 天** 且相关函数被修改过 → 升级提醒（INFO → WARNING）：「这个认知边界已声明 N 天，是否有足够实践数据转为 @anchor.test？」
- **anchor health states**：healthy（全测试过）/ unverified（只有 idk）/ degrading（有测试但部分失败）/ stale_unknown（idk>90 天且函数已改）/ skeleton（无锚点——违反第一律）/ uncompilable（有锚点但无法独立编译，RE 常见）。

---

## 3. 置信度状态机（来自 RE-Framework 铁律）

```
draft（AI 初稿）→ candidate（有验证证据）→ confirmed（用户拍板）
```

- 任何 AI 产出默认 `draft` 或 `candidate`，**绝非最终真理**。
- **`confirmed` 只有用户亲自拍板后才能标记**——AI 永远不能自己写 confirmed。
- 阅读他人产出要带怀疑态度，检查置信度标记。
- 审查者只出审查意见，不直接改 status。
- 映射 CoreSwap 知识库：`✅ 已确认` = 用户/双源确认；AI 只标 `candidate`；❌ = 排除的假说（同样要记录）。

---

## 4. 验证分层（Full / Partial / Degraded）

| 模式 | 条件 | 验证 | 置信度 |
|---|---|---|---|
| **Full** | C++ 可编译 + probe 可跑 + 参照可信 | block_probe 逐位对比全绿 | 可自动提升 draft→candidate |
| **Partial** | 依赖 Java 运行时/反射（density_probe、router_probe） | 反射对照 | 仅手动提升 |
| **Degraded** | 无法独立验证（javap 静态对照、无参照） | 静态审查 | 仅手动提升，confirmed 需更严格人工对照 |

**降级模式诚实声明**：Partial/Degraded 的输出 MUST 前缀声明「验证结果基于降级模式，置信度未自动提升」。

> CoreSwap 关键教训：cns 反射（blockStateSampler / CellCache）有缓存污染，**不可信**（2026-08-08 确认）；DensityProbe 导出必须 `CppBridge.enabled=false` 否则参照被 C++ 污染。

---

## 5. Noise Cards（噪声卡）

差块/未解之谜结构化，跨 session 可检索、可注入上下文。

```json
{
  "noise_id": "nc-2026-08-08-001",
  "timestamp": "2026-08-08T00:00:00Z",
  "trigger": "seed=-8248318472910187742 chunk(-16,-16) (-244,58,-256)",
  "function_name": "density/aquifer apply",
  "observed": "C++ density=-0.074 water; Java densityFunction.sample=+0.044 stone",
  "expected": "island (dirt 58-61)",
  "anchor_violated": null,
  "root_cause": "结构 Beardifier 密度修正未实现（结构附近 density 差 ~0.12 翻转 aquifer 判定）",
  "status": "confirmed_not_bug"
}
```

- 每条 noise card 记录 trigger/observed/expected/root_cause/status。
- CoreSwap 差块分类：真 bug（如 SurfaceCondC）/ 假 diff（FEATURE/结构，如 -288 岛）/ 待解之谜（如洞穴底 dirt）。

---

## 6. 逆向方法论层（来自 RE-Framework，选择性吸收）

### Phase 0 轻量架构计划（大任务前置）

大排查/版本迁移任务开始前 MUST 出轻量计划（范围/角色/验证方式/风险回退），日常小 diff 不需要。

### 多角色映射（Reasonix subagent 承载）

| 角色 | 职责 | CoreSwap 执行 |
|---|---|---|
| Scout | 定位差异范围 | block_probe 全区域对比，产出差异分布 |
| Worker | 深挖/还原 | javap/Java 源码 → C++ 修复，同时写 @anchor.test |
| Judge | 审查置信度 | 审查结论，confirmed 用户拍板 |

### Lift 原则（禁用直接信任反编译）

- **禁止直接信任 javap 反编译输出**——Java 源码（yarn mappings + sources jar）才是权威。
- 浮点精度、负坐标 floorDiv、int 溢出（如 `x * 3129871` int 乘法补码）是易错点，必须逐位验证。

---

## 7. 过程纪律（与「不放弃原则」的边界）

- **Retry cap**：同一假设 3 次验证失败 MUST 换方向（回 A 层：重新收集数据/Scout）。
- **⚠️ 与全局「不放弃原则」的边界**：不放弃 = 总方向（除非用户命令停止，否则持续推进）；retry cap = 单点假设策略（3 次失败换方向，不是放弃任务）。两者不冲突。
- **产物落盘**：分析结果 → 知识库 docs/；思维链 → 时间线 09 或排查记录；不要只留在对话里。
- **知识库链条铁律**：每条结论 MUST 带「猜测→验证→排除→发现」完整链条；被排除的假说（❌）也记录。

---

## 8. 第三律（可挑战）：规则挑战流程

知识库「已验证坑」/本协议的规则，若出现**验证过的误报（FP）证据** MUST 被降级或删除：

1. **报告**：最小复现（代码片段 + 证据）。
2. **验证**：维护者复现 FP，加入回归测试。
3. **裁决**：规则 (a) 精化排除该情况 / (b) 降级严重度 / (c) 从协议移除；理由 MUST 记录到 changelog。
4. **证据要求**：无 FP 证据不能削弱规则；有确认 FP 不能保持规则不变。

---

## 9. 使用流程（日常混合工作流）

```
1. 差异报告/新任务
2. Phase 0 轻量计划（大任务才需要）
3. Scout（可选，subagent）：block_probe 对比定位
4. Worker：逆向（Java 源码→C++）+ 编程（实现）+ 写 @anchor.test/@anchor.idk
5. Verify：anchorlaw-scanner 扫注解（source 校验）+ block_probe 逐位验证（Full/Partial）
6. Judge（可选，subagent）：审查置信度，confirmed 用户拍板
7. 知识库：记录链条 + noise card（差块分类）
8. 记录：差块分类（真 bug/假 diff/待解）进 noise cards，结论带链条进知识库
```

---

## 10. 工具链

| 工具 | 位置 | 用途 |
|---|---|---|
| anchorlaw-scanner（C++ 扫描） | E:\PYTHON\Anchorlaw\python\anchorlaw-scanner（PYTHONPATH 引用） | 提取 @anchor 注解、校验 source 必填 |
| block_probe | versions/1.20.1/cpp/build-msvc/bin | 逐位对比（Full 验证载体） |
| router_probe / density_probe | MC java（gradle runServer -P...Probe） | 分量/密度反射（Partial） |
| got_export | versions/1.20.1/cpp/build-msvc/bin | densityDump（无插值） |

---

## 11. 全称声称审计（本文档）

| 声称 | 位置 | 验证 | 状态 |
|---|---|---|---|
| `@anchor.test` MUST 带 source | §1 | scanner `scan_cpp_file` 校验（test_cpp.py 有测试） | ✅ 已实现 |
| C++ 注解 inert | §1 | 行注释天然 inert（by construction） | ✅ 已验证 |
| idk 90 天 staleness 升级 | §2 | Python 实现；「函数已修改」检测未实现 | ⚠️ partial |
| 「cns 反射不可信」 | §4 | 2026-08-08 -288 排查实测（缓存污染） | ✅ 已验证 |
| 差块分类（真 bug/假 diff/待解） | §5 | -288（假 diff=结构）、8576（真 bug=terracotta 带边缘）、洞穴底 dirt（待解） | ✅ 已验证 |
| 负坐标 density 已对齐 | §4 | est/分量/角点/插值全一致（2026-08-08） | ✅ 已验证（限 -288 区域，非全坐标声称） |
