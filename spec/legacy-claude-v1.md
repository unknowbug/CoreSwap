# CLAUDE.md — 逆向工程项目模板
#
# 使用方法：
#   1. 复制此文件到你的 RE 项目根目录，重命名为 CLAUDE.md
#   2. 按项目实际情况修改「项目信息」段
#   3. Claude Code 启动时会自动加载，所有行为受此约束
#
# 方法论来源：多Agent自主管理上下文的逆向思路 (BitWarden, 看雪 2026)
# 框架包：参见 RE-Framework/README.md

## ═══════════════════════════════════════════════════════════
## 项目信息（按实际修改）
## ═══════════════════════════════════════════════════════════

本项目是一个逆向工程项目。

目标文件:
  - <binary名称>  (SHA256: <hash>)
  - ...

Practify 验证状态（如实声明，不模糊）:
  - [ ] Practify 已安装且目标函数可独立编译 → 全功能: 自动验证 + 置信度自动提升
  - [ ] Practify 已安装但目标函数依赖外部符号无法编译 → 半功能: anchor 记录数据来源，验证待补
  - [ ] Practify 未安装 → 降级模式: 验证依赖人工审查，confidence 不会自动提升，confirmed 需要更严格的人工对照（必须有 trace 证据）
  （降级不是错误——是诚实地标注当前能做到什么。详见 Phase 2.5）

## ═══════════════════════════════════════════════════════════
## 核心铁律 — 任何时候都必须遵守
## ═══════════════════════════════════════════════════════════

1. 置信度状态机:
   - 任何AI产出默认是 `draft` 或 `candidate`，绝非最终真理
   - `confirmed` 只有用户亲自拍板后才能标记 — AI永远不能自己写 confirmed
   - 阅读他人产出时要带怀疑态度，检查置信度标记
   - 审查Agent只出审查意见，不直接改status

2. 产物必须落盘:
   - 分析结果 → .artifacts/
   - 思维链/推理过程 → .investigations/
   - 不要只留在对话里

3. 禁止直接信任F5:
   - IDA F5 结果是参考，不是真理
   - 高精度还原必须走 Lift 流程（见下方）
   - 尤其注意浮点精度、间接调用、虚函数表

4. 上下文隔离:
   - 单个任务过长时主动拆分子任务
   - 每个子任务产物独立存放
   - 避免在同一个对话里塞过多汇编代码

5. 可验证的正确性（Practify 集成）:
   - Lift 产出 .cpp 时，必须同时产出 test anchor（从汇编trace中提取测试向量）
   - 每个 test anchor 必须附带数据来源（trace_id / 寄存器快照 / 观测时间）— 不可凭空编造
   - 不确定的部分用 @i_dont_know 显式标注，不许闷在注释里
   - "只进攻不防御" — 每个结论要么有可追溯来源的 test 验证，要么诚实标注 i_dont_know
   - 如果 .cpp 因外部依赖无法独立编译 → 诚实声明，不可假装验证了

## ═══════════════════════════════════════════════════════════
## 目录结构
## ═══════════════════════════════════════════════════════════

<project>/
├── CLAUDE.md               ← 本文件
├── .mcp.json               ← MCP 配置 (IDA/Frida/x64dbg 等)
├── pract_stub.py           ← Practify 桩文件（如果启用 Practify 集成）
│
├── .artifacts/             ═══ AI 分析产物 ═══
│   ├── index.yaml          ← 主索引（地址 → 产物路径）
│   ├── <binary>/
│   │   ├── classes/
│   │   │   └── <ClassName>/
│   │   │       ├── class.yaml           # 类元数据
│   │   │       └── methods/
│   │   │           ├── <method>.yaml     # 当前活跃版本
│   │   │           ├── <method>.cpp      # Lift 产物
│   │   │           ├── <method>_anchors.py  # Practify test anchors（如启用）
│   │   │           ├── <method>.vN.yaml  # 历史版本
│   │   │           └── <method>.bN.yaml  # fan-out 候选分支
│   │   └── functions/
│   │       ├── 0x<address>.yaml          # 未归类函数
│   │       ├── 0x<address>.cpp           # Lift 产物
│   │       └── 0x<address>_anchors.py    # Practify anchors（如启用）
│   └── cross_refs/
│       └── xref_<NNN>.yaml              # 跨binary引用
│
├── .investigations/        ═══ AI 思维链 ═══
│   ├── 000-<阶段名>/
│   │   ├── 任务.md
│   │   ├── 发现*.md
│   │   └── 待挖清单.md
│   └── 001-<任务名>/
│       ├── 任务.md
│       ├── 假设A-*.md
│       ├── 假设B-*.md
│       └── 结论.md
│
└── knowledge/              ═══ 知识库 ═══
    ├── INDEX.md              ← 总索引（Agent遇到问题时先查这里）
    ├── builtin/               ← 预置知识（模板自带，一般不改）
    │   ├── calling-conventions.md   # x86/x64/ARM 调用约定速查
    │   ├── cpp-abi.md               # C++ ABI: vtable布局/RTTI/异常/多重继承
    │   ├── common-patterns.md       # 常见算法模式: CRC/哈希/XOR/压缩/加密
    │   ├── assembly-reference.md    # x64汇编参考: 常见指令模式/lea用法/间接调用
    │   └── anti-re.md               # 常见反逆向手法: 花指令/壳/混淆模式
    └── discovered/            ← 项目中发现的知识（AI运行时自动写入）
        ├── compiler-idioms.md       # 本项目编译器特有的代码生成模式
        ├── f5-bugs.md               # 本项目遇到的F5误译模式及修正方法
        ├── algorithm-fingerprints.md # 已确认的算法实例: 常量/特征/调用链
        └── anti-patterns.md         # 本项目特有的反逆向/混淆手法

## ═══════════════════════════════════════════════════════════
## 产物文件格式
## ═══════════════════════════════════════════════════════════

### .artifacts/index.yaml — 主索引

```yaml
schema_version: 1
binaries:
  - name: foo.dll
    sha256: <hash>
    arch: x64
addresses:
  'foo.dll:0x1400077c0':
    path: 'foo.dll/classes/Animal/methods/speak.yaml'
    kind: method
  'foo.dll:0x140007550':
    path: 'foo.dll/functions/0x140007550.yaml'
    kind: function
classes:
  - id: 'foo.dll:Animal'
    path: 'foo.dll/classes/Animal/'
```

### 类文件 (.artifacts/<binary>/classes/<Name>/class.yaml)

```yaml
schema_version: 1
binary: foo.dll
class_name: Animal
address: 0x140007000          # vtable 地址
status: candidate             # draft | candidate | confirmed
vtable:
  address: 0x140007000
  methods:
    - offset: 0x00
      name: speak
      address: 0x1400077c0
    - offset: 0x08
      name: get_age
      address: 0x140007740
members:
  - offset: 0x04
    type: 'int'
    name: age                # 推测名
    evidence: '[this+4] 在 speak() 中作为计数器使用'
  - offset: 0x08
    type: 'void*'
    name: pUnknown
non_virtual_methods:
  - name: ctor
    address: 0x140007500
    calling_convention: thiscall
methods:
  - speak
  - get_age
  - ctor
```

### 方法文件 (.artifacts/<binary>/classes/<Name>/methods/<method>.yaml)

```yaml
schema_version: 1
binary: foo.dll
class: Animal
method_name: speak
address: 0x1400077c0
status: candidate             # draft | candidate | confirmed
signature: 'void __thiscall Animal::speak(int volume)'
calling_convention: thiscall
parameters:
  - name: this
    type: 'Animal*'
    register: rcx
  - name: volume
    type: int
    register: edx
return_type: void
pseudocode_path: 'Animal/methods/speak.cpp'   # Lift 产物
anchor_path: 'Animal/methods/speak_anchors.py'   # Practify anchor（如启用）
uncertain_areas:              # 来自 @i_dont_know 列表
  - 'volume > 100 的行为未覆盖'
  - 'AudioDevice::write 返回值是否影响 speak 返回值'
control_flow:
  - type: if_else
    condition: 'volume > 0'
  - type: virtual_call
    target: '[rcx+8]'
    hypothesis: '可能是 AudioDevice::write()'
dependencies:
  - address: 'foo.dll:0x140008000'
    type: callee
    name_hint: 'AudioDevice::write'
notes: |
  第3行 lea 用于计算偏移，已折叠。
  F5 结果中第12行的 while 循环实际是 do-while，
  经骨架还原确认。
```

### 未归类函数 (.artifacts/<binary>/functions/0x<addr>.yaml)

```yaml
schema_version: 1
binary: foo.dll
address: 0x140007550
status: candidate
signature: 'int __fastcall sub_140007550(__int64 a1, int a2)'
calling_convention: fastcall
parameters:
  - {name: a1, type: '__int64', register: rcx}
  - {name: a2, type: int, register: edx}
return_type: int
pseudocode_path: 'functions/0x140007550.cpp'
notes: |
  疑似一个初始化函数，调用者来自 0x140003200。
  待归类到某个类的方法后迁移。
```

### 跨引用 (.artifacts/cross_refs/xref_<NNN>.yaml)

```yaml
编号: xref_001
状态: confirmed
版本: 1
从:
  binary: bar.exe
  地址: '0x140003200'
到:
  binary: foo.dll
  目标类型: 方法
  类: Animal
  方法名: speak
  地址: '0x1400077c0'
关系类型: 调用
证据: 间接调用 [rcx+8]，rcx 在 0x14000312A 处加载 Animal::vftable
```

## ═══════════════════════════════════════════════════════════
## Phase 0: 架构设计（强制前置 — 任何RE工作开始前必须完成）
## ═══════════════════════════════════════════════════════════

### 原则

在动手分析之前，必须先有架构。架构决定了:
- 任务如何拆分（单Agent搞不定大任务）
- 上下文如何隔离（每个子任务独立 investigation）
- 哪些可以并行（独立的 binary / 独立的函数 / 独立的假设）
- 用户需要参与哪些决策（用户只做决策，AI负责执行）

### 决策树

```
用户给出RE任务
    │
    ├── 轻量判断: 单一函数/小范围/用户已明确指定路径?
    │   └── YES → 轻量模式:
    │       1. 我自动评估，生成简版架构计划
    │       2. 展示给用户确认（≤3个要点）
    │       3. 用户点头后直接执行
    │
    └── 重量判断: 多binary/大范围/目标模糊/类层次复杂?
        └── YES → 重量模式:
            1. 进入计划模式 (EnterPlanMode)
            2. 与用户讨论:
               - 任务拆解方案
               - Agent角色分配
               - 并行策略
               - 关键决策点（哪些环节需要人工打断）
            3. 用户批准后才开始执行
```

### 轻量模式 — 架构计划模板

输出到 `.investigations/000-架构设计/架构计划.md`:

```markdown
---
编号: 000
任务: <一句话描述>
任务类型: <算法还原 | 协议分析 | 定位 | 脱壳 | ...>
模式档位: 轻量
创建时间: <ISO日期>
---

## 范围
- 目标 binary: <name>
- 目标地址/函数: <list>
- 边界: <明确不做什么>

## 任务拆解
1. [ ] <子任务1> → 产物: <预期文件>
2. [ ] <子任务2> → 产物: <预期文件>

## 并行策略
- <哪些可以同时做，哪些有依赖>

## 关键HOOK点
- <哪些节点需要中断让用户决策>

## 验证方式
- <怎么确认结果正确>
```

### 重量模式 — 架构设计文档模板

输出到 `.investigations/000-架构设计/架构设计.md`:

```markdown
---
编号: 000
任务: <一句话描述>
任务类型: <算法还原 | 协议分析 | 定位 | 脱壳 | ...>
模式档位: 重量
创建时间: <ISO日期>
状态: 待批准
---

## 1. 全局视图
- 涉及 binary: <列表，含依赖关系>
- 目标范围: <总体描述>
- 不在此次范围内: <明确排除>

## 2. Agent 角色分配

| 角色 | 负责 | 模型建议 | 备注 |
|------|------|---------|------|
| Scout/勘探 | 定位关键地址、交叉引用搜索 | 快速模型 | 只读，不产出正式产物 |
| Worker/分析 | 具体函数/类还原 | 主力模型 | 产出 draft/candidate |
| Judge/审查 | 独立验证 Worker 产出 | 主力模型 | 只出意见，不改status |
| Scribe/文档 | 整理产物、更新索引 | 快速模型 | 格式转换、索引维护 |

## 3. 任务拆解 & 依赖图

Phase 1: <阶段名>
├── T1.1 <任务名> [可并行]
│   ├── 输入: <依赖>
│   ├── 角色: <Scout/Worker/Judge>
│   ├── 产物: <.artifacts/ 路径>
│   └── 验证: <标准>
├── T1.2 <任务名> [依赖 T1.1]
│   └── ...
└── T1.3 <任务名> [可并行，与 T1.1 无关]

Phase 2: <阶段名>
└── ...

## 4. 并行执行计划

- 第一波（并行）: T1.1 + T1.3
- 第二波（串行）: T1.2 ← 等待 T1.1
- ...

## 5. 人工决策HOOK点

| 节点 | 触发条件 | 决策内容 |
|------|---------|---------|
| T1.1 完成后 | 发现 RTTI / 未发现 RTTI | 决定类分析策略 |
| T1.2 产出 candidate | Worker 置信度 < 90% | 是否需要补充证据 |
| Phase 1 结案 | 所有 candidate 就位 | 批量 confirmed / 打回 |

## 6. 风险 & 回退

- 风险1: <描述> → 回退: <方案>
- 风险2: <描述> → 回退: <方案>
```

### 架构确认后

- 轻量: 用户口头确认即可，开始执行
- 重量: 用户明确说"开始"或"批准"，然后按 Phase 顺序推进

### 架构变更

- 执行中发现架构不适用 → 暂停，回到本Phase更新架构设计
- 小的方向微调（不改变任务依赖关系）不需要重新架构
- 新增 binary / 新增大规模分析范围 → 必须重新评估

## ═══════════════════════════════════════════════════════════
## 模型选择策略
## ═══════════════════════════════════════════════════════════

逆向工程中不同角色的任务特征决定了模型选择:

| 角色 | 任务特征 | 模型要求 | 推荐 |
|------|---------|---------|------|
| 主脑 (Coordinator) | 语义理解、任务拆解、决策 | 强推理能力，指令跟随精准 | DeepSeek V4 / Claude Opus |
| Scout/勘探 | 大范围搜索、交叉引用 | 速度快、上下文窗口大 | GLM 5.1 / DeepSeek V3 |
| Worker/Lift | 精确汇编翻译、算法还原 | 对汇编语义理解深度强 | Claude 4.7 / GLM 5.1 |
| Worker/Class | 类结构识别、vtable分析 | C++ ABI 熟悉度高 | Claude 4.7 / DeepSeek V4 |
| Judge/审查 | 多版本对比、逻辑一致性检查 | 批判性思维，能独立挑错 | Claude Opus / DeepSeek V4 |
| Scribe/文档 | 格式转换、索引更新 | 速度快即可 | GLM 5.1 / DeepSeek V3 |

核心逻辑:
- **计算密集型**(Lift/Class-identify) → 用最强推理模型，宁可慢不要错
- **IO密集型**(Scout/Scribe) → 用快速模型，不浪费主力模型的token
- **关键决策** → 主脑+用户配合，AI出建议，用户拍板
- 国产/内网环境: 主脑用 DeepSeek V4，干活用 GLM 5.1，完全独立于 Claude

## ═══════════════════════════════════════════════════════════
## Knowledge 知识库（自带生长机制）
## ═══════════════════════════════════════════════════════════

### 设计理念

AI不懂真正的逆向——它只认识汇编和F5输出，缺乏编译原理和C++ ABI的底层认知。
知识库就是把人类RE专家的基础知识蒸馏成结构化文档，让Agent在需要时查阅。

**关键创新: 知识库不是死的。** 逆向过程本身就是知识发现过程——
每次遇到一个新的编译器惯用法、一个F5误译模式、一个算法特征，
都应自动沉淀到 `discovered/`，后续所有分析直接受益。

### 目录结构

```
knowledge/
├── INDEX.md                  ← 总索引（Agent 遇到问题时先查这里）
├── builtin/                  ← 预置知识（模板自带，不随项目变化）
│   ├── calling-conventions.md
│   ├── cpp-abi.md
│   ├── common-patterns.md
│   ├── assembly-reference.md
│   └── anti-re.md
└── discovered/               ← 项目中发现的知识（AI 运行时自动写入）
    ├── compiler-idioms.md
    ├── f5-bugs.md
    ├── algorithm-fingerprints.md
    └── anti-patterns.md
```

### INDEX.md — 入口（自动维护）

```markdown
# 知识库索引

## 预置知识 (builtin/)
- [调用约定速查](builtin/calling-conventions.md)
- [C++ ABI](builtin/cpp-abi.md)
- [常见算法模式](builtin/common-patterns.md)
- [汇编参考](builtin/assembly-reference.md)
- [反逆向手法](builtin/anti-re.md)

## 项目发现 (discovered/)
<!-- AI: 每次新增发现后，在此追加一行 -->

### 编译器惯用法
- [compiler-idioms.md](discovered/compiler-idioms.md)

### F5误译 & 修正
- [f5-bugs.md](discovered/f5-bugs.md)

### 算法指纹
- [algorithm-fingerprints.md](discovered/algorithm-fingerprints.md)

### 反逆向手法
- [anti-patterns.md](discovered/anti-patterns.md)
```

### 自动贡献机制 — 什么时候写

在任何阶段（Scout / Worker / Judge），遇到以下情况**立即写入 discovered/**：

| 触发条件 | 写入文件 | 示例 |
|---------|---------|------|
| 发现本项目编译器特有的代码生成模式 | `compiler-idioms.md` | "本项目 MSVC 对虚函数调用始终生成 `mov rax,[rcx]; call [rax+N]`" |
| 发现 F5 明确误译，且找到了修正方法 | `f5-bugs.md` | "F5 把 do-while 译成 while，特征: 循环末尾有 `test al,al; jnz`" |
| 确认了某个算法及其常量/特征 | `algorithm-fingerprints.md` | "0x1400077c0 = CRC32/MPEG2, 多项式 0xEDB88320, 初始值 0xFFFFFFFF" |
| 发现本项目特有的反逆向手法 | `anti-patterns.md` | "字符串表全部 XOR 0xAA 加密，解密函数在 0x140001000" |
| 发现跨 binary 的通用结构模式 | `compiler-idioms.md` | "所有 DLL 导出类都通过工厂函数返回，工厂命名 `Create<X>`" |
| 多次遇到同一类汇编模式，值得总结 | 新建 `discovered/<topic>.md` | "连续3个函数都用 `lea rdx,[rip+xxx]` 加载常量池" |

### 写入格式

每次发现写一个独立条目，不要合并到已有段落:

```markdown
## 发现 #<N>: <简短标题>

**发现时间:** <ISO日期>
**发现者:** <Scout/Worker/Judge — 哪个阶段发现的>
**来源地址:** <触发此发现的地址>
**置信度:** <draft | candidate | confirmed>

### 观察
<描述观察到的现象>

### 证据
- 地址 A: <具体表现>
- 地址 B: <同样模式重复出现>

### 如何利用
<后续分析中如何使用这条知识>
- 下次遇到 <条件> → <应用方式>
```

### 使用规则

- **分析前先查 INDEX.md** → 确认是否有相关预置知识或已发现知识
- 如果 builtin/ 和 discovered/ 都没有 → 不做猜测，标注"需要查外部资料"，让用户决定
- **发现即写入** → 不要拖到"回头再记"，当时就写
- 写入后**更新 INDEX.md** → 在对应分类下加一行链接
- 如果 discovered/ 下需要新建文件 → 同步更新 INDEX.md 的分类结构
- 知识库文件是 Markdown，不限格式，但要可检索
- 用户也可以手动编辑 discovered/ 文件补充

### 知识库的生长循环

```
Scout 勘探
  ├── 遇到不认识的东西 → 查 INDEX.md → 没有 → 标注"未知"
  └── 遇到重复模式 → 写入 discovered/ → 更新 INDEX.md
        │
Worker 分析
  ├── 开始前查 INDEX.md → 发现 Scout 刚写的知识 → 直接用
  ├── 分析中确认新事实 → 写入 discovered/（置信度提升为 candidate）
  └── 分析中修正旧发现 → 更新对应的 discovered/ 文件
        │
Judge 审查
  ├── 交叉验证多个 worker 产出 → 确认通用模式
  └── 写入 discovered/（置信度提升为 confirmed）
        │
下次分析 → INDEX.md 里已有了 → 不再重复发现
```

## ═══════════════════════════════════════════════════════════
## Phase 1: Scout — 勘探定位
## ═══════════════════════════════════════════════════════════

### 角色

Scout 是只读的勘探阶段，发生在 Worker 精确分析之前。
任务: 快速摸清道路，找到需要下钻的点位，不产出正式产物。

### 职责

1. **入口定位**: 从已知地址/字符串/导入表出发，找关键函数入口
2. **交叉引用扫描**: xref 图、字符串引用、虚函数表引用
3. **粗略分类**: 区分这是类方法/普通函数/库函数/系统API
4. **依赖摸底**: 该函数调用了哪些其他函数/引用了哪些数据
5. **输出**: 写入 `.investigations/<编号>-Scout/` 目录

### Scout 产物格式

写入 `.investigations/<编号>-Scout/发现.md`:

```markdown
## 勘探总结

### 关键地址清单
| 地址 | 类型 | 初步判断 | 置信度 | 建议 |
|------|------|---------|--------|------|
| 0x1400077c0 | thiscall | Animal::speak? | draft | 交给 Worker 做 Lift |
| 0x140007740 | 数据引用 | 可能是 vtable | draft | 交给 Worker 做 Class-identify |

### 交叉引用图（文字描述）
0x140003200 ──call──> 0x1400077c0 (thiscall, rcx=vtable)
0x1400077c0 ──间接调用──> [rcx+8h] (目标待定，需继续勘探)

### 待深入的点
- [ ] 0x140008000 处有未识别函数，可能是 AudioDevice::write
- [ ] rcx 来源追溯到 0x14000312A，需要确认 vtable 初始化逻辑

### 影响架构的变化
- <如果发现改变了 Phase 0 的假设，在此标注>
```

### Scout 规则

- Scout **不产出 .artifacts/** — 只写 .investigations/
- Scout 不做精确还原 — 那是 Worker 的活
- Scout 发现的地址写入架构设计的依赖图，更新 Phase 0
- 如果 Scout 发现与架构设计不符 → 暂停，重新评估架构

## ═══════════════════════════════════════════════════════════
## Phase 2+: Worker & Judge — 精确分析
## ═══════════════════════════════════════════════════════════

### Lift — 高精度汇编→C++ 还原（禁止直接信任F5）

当需要高精度还原函数时，严格按以下5步执行，产物输出为 .cpp 文件:

步骤1: 骨架还原 — 识别控制流壳 (if/while/for/switch/do-while)，
       条件和函数体用占位符填充。
       重点：F5常把do-while还原成while，需要回汇编确认。

步骤2: 逐行机械翻译 — asm → C伪代码，变量名直接使用寄存器名，
       不做任何优化和识别。每行汇编对应一行C注释。

步骤3: 签名草稿 — 从调用方视角推断:
       - 调用约定 (cdecl/stdcall/fastcall/thiscall)
       - 参数个数和类型
       - 返回值类型
       标记为待修正。

步骤4: 优化 — 约定寄存器识别 / 折叠中间寄存器 /
       内存操作分类 (栈变量 vs 堆对象 vs 全局变量) /
       lea 指令处理 (取地址 vs 算术运算)

       本步骤中同步提取测试向量（含数据来源——这是强制字段）:
       - 从汇编 trace 中记录每条数据的来源: trace_id, 地址偏移, 寄存器/内存快照值, 观测时间
       - 从汇编逻辑中推断关键分支条件和期望输出
       - 记录无法覆盖的分支（这些将成为 @i_dont_know）
       - 来源信息格式: `trace:<binary>!<function>#<id>, offset=0x<addr>, <reg>=<val> observed <ISO时间>`

步骤5: 语义折叠 — 识别算法模式 (CRC/哈希/加密/压缩等) /
       改有意义变量名 / 修正签名 /
       标注置信度低的区域

步骤6: 附带 Anchor（如果项目启用了 Practify 集成）:
       - 从步骤4提取的测试向量生成 _anchors.py 文件
       - 每个明确可验证的输入→输出对 → @pract.test
       - 每个无法确定的行为边界 → @pract.i_dont_know
       - 格式见下方 Anchor 文件规范

产物命名: <函数名或地址>.cpp 和 <函数名或地址>_anchors.py，置于对应 .artifacts 目录下。

### Anchor 文件规范（用于 Lift 步骤6）

每个 .cpp 产物对应一个 `_anchors.py` 文件:

```python
# <binary>/classes/<Class>/methods/<method>_anchors.py
# Auto-generated by Lift step 6. Do not hand-edit without trace evidence.
from pract_stub import test as pt, i_dont_know as idk

# ── Verified: 从汇编 trace 直接提取，每个 @pt 必须带 source ──
@pt("x64 thiscall: rcx=this_ptr, edx=volume(0) → eax=0",
    lambda: speak(Animal(), 0) == 0,
    source="trace:foo.dll!speak#001, offset=0x00, rcx=0x12345678 edx=0 eax=0 observed 2026-06-18T10:00:00Z")
@pt("volume=5 → 分支进入 AudioDevice::write 路径, eax=1",
    lambda: speak(Animal(), 5) == 1,
    source="trace:foo.dll!speak#002, offset=0x1A, rcx=0x12345678 edx=5 eax=1 observed 2026-06-18T10:00:05Z")
@pt("volume=-1 → 提前返回, eax=0, 不调用任何虚函数",
    lambda: speak(Animal(), -1) == 0,
    source="trace:foo.dll!speak#003, offset=0x08, rcx=0x12345678 edx=-1 eax=0 observed 2026-06-18T10:00:10Z")

# ── Uncertain: trace 中未覆盖的行为边界 ──
@idk("volume=100 时是否会溢出？F5 显示 cmp edx, 64h 但未在 trace 中触发过",
     source="static:foo.dll!speak@0x1400077c0, F5 显示分支 je short loc_xxx 但动态 trace 未覆盖 edx≥100 的路径")
@idk("this==NULL 时的行为未观察，调用方似乎从不传空指针")
@idk("AudioDevice::write 的返回值是否影响 speak 的返回值？F5 显示 eax 被覆盖")

def speak(this: 'Animal', volume: int) -> int:
    """还原自 foo.dll:0x1400077c0, status: candidate"""
    ...
```

### Anchor source 字段规范（强制）

每个 @pt 锚点必须包含 `source` 字段，格式:

```
source="<来源类型>:<binary>!<function>#<trace_id>, offset=0x<addr>, <关键寄存器/内存快照> observed <ISO8601时间>"
```

| 来源类型 | 含义 | 示例 |
|---------|------|------|
| `trace` | 动态调试产生的寄存器/内存快照 | `trace:foo.dll!speak#003, offset=0x1A, eax=1 observed 2026-06-18T10:00:05Z` |
| `static` | 静态分析推断（仅限 @idk 使用，不能用于 @pt） | `static:foo.dll!speak@0x1400077c0, F5 显示分支 je` |
| `memory` | 内存 dump 提取的常量/表 | `memory:foo.dll!sbox@0x14001000, dump at 2026-06-18` |

- @pt 的 source **必须**是 `trace` 或 `memory` 类型 — 必须有 A 层数据来源
- @idk 的 source 可以是 `static` 类型 — 诚实标注"这个未知是从静态分析中推断的"
- 无 source 字段的 @pt → Judge 审查时**直接驳回**（视为凭空编造）
- source 字段同时记录在对应 .yaml 文件的方法/函数条目中

Anchor 质量要求:
- test anchor 的数据必须来自汇编 trace 或动态调试观察值，不能凭空编造
- i_dont_know 必须具体到一个可验证的条件，不能写成"可能有bug"
- 每个分支路径至少一个 test anchor；如果某个分支覆盖不到 → 必须有 i_dont_know
- @i_dont_know 本质上是"等待被推翻的临时结论"，不是免死金牌
- 如果后续动态调试获得了新 trace 数据 → 更新 anchor，将 i_dont_know 转化为 test

### Class-identify — 类结构识别

优先级依次递减:

1. RTTI 优先:
   - 搜索 `??_R0?AV` / `??_7` / `.rdata` 中的 type_info
   - 有 RTTI → 整套继承图、类名、虚函数表直接拿到
   - 不需要后续步骤

2. 无RTTI时，用户给起点:
   - 用户指定某个 ctor 或 thiscall 方法的地址作为分析起点
   - 从该函数中提取 vtable 引用

3. 通过 vtable 列虚方法:
   - 读取 vtable 数组，每个槽位是一个虚方法地址
   - 对每个地址执行粗粒度反编译，确认是否为方法

4. thiscall 偏移聚类列非虚方法:
   - 搜索所有 `mov rcx, <vtable_base>` 附近的调用
   - 按 [this+N] 偏移聚类，得成员清单

5. 方法体粗还原:
   - 汇总所有 [this+N] 引用
   - 推断成员类型和语义

产物: class.yaml（格式见上方）

### 函数→类迁移流程

当发现某个 functions/ 下的函数实为某类的方法时:

1. 在 classes/<X>/methods/<name>.yaml 写新产物
2. 更新 index.yaml 中对应地址的 path 和 kind (function→method)
3. 更新 class.yaml 的方法清单
4. 删除旧位置 functions/<addr>.yaml
5. 如果有对应的 _anchors.py，一并迁移

## ═══════════════════════════════════════════════════════════
## Phase 2.5: Verify — 利用 Practify 自动验证 Lift 产出
## ═══════════════════════════════════════════════════════════

### 前提：三种运行模式（诚实声明，不假装）

Phase 2.5 的行为取决于项目的 Practify 实际状态:

**模式 A — 全功能:** Practify 已安装 AND .cpp 可独立编译
→ 走完整自动验证流程（practify test → 置信度自动提升）

**模式 B — 半功能:** Practify 已安装 BUT .cpp 无法独立编译（依赖外部符号/类型/内存布局）
→ anchor 仍产出，source 字段仍记录，但 practify test 无法运行
→ 写入 `.artifacts/uncompilable_functions.yaml` 清单，标注每个函数缺什么依赖
→ 验证推迟到动态调试阶段（有了可运行的 mock 环境后再补）

**模式 C — 降级:** Practify 未安装
→ 产物落盘仍然完整（.cpp + _anchors.py）
→ 置信度永远不会自动提升——必须用户手动 confirmed
→ confirmed 前必须有人工对照 trace 证据的审查记录

**模式选择决策表（Lift 步骤6 产出前执行）:**

```
Lift 产出 .cpp
    │
    ├── 函数是纯逻辑（无外部调用、无全局变量引用、无自定义类型）?
    │   └── YES → 标记为 "self-contained" → 可进模式 A
    │
    ├── 函数调用了其他已知函数 / 引用了全局 / 使用了自定义结构体?
    │   └── YES → 标记为 "has_deps: <列出依赖>"
    │       ├── 所有依赖都已在本项目中 Lift 且自包含?
    │       │   └── YES → 组合编译 → 可进模式 A
    │       └── 有未解析的依赖?
    │           └── → 模式 B，写入 uncompilable_functions.yaml
    │
    └── Practify 未安装?
        └── → 模式 C
```

### 流程（模式 A 全功能时）

```
Lift 产出 .cpp + _anchors.py（含 source 字段）
        │
        ▼
practify test 运行所有 test anchor
        │
        ├── 全部通过 → status 从 draft 升为 candidate
        │              置信度自动提升（不再仅靠人工判断）
        │
        ├── 部分失败 → 检查失败原因:
        │   ├── anchor 写错了 → 修正 anchor（重查 source 字段对应的 trace）
        │   ├── Lift 有问题 → 回 Phase 2 修正 .cpp
        │   └── 不确定 → 降级为 @i_dont_know，生成 Noise Card
        │
        └── @i_dont_know 项:
            ├── 标注在函数 yaml 的 uncertain_areas 字段
            ├── 累积到项目的噪声卡文件 .artifacts/noise_cards.json
            └── 等待后续动态调试补全
```

### 重试硬性上限（防止过程熵增）

同一函数的 Lift→Verify 循环**最多 3 次**:

```
retry_count = 0
while test_failed and retry_count < 3:
    fix_lift_or_anchor()
    rerun_practify_test()
    retry_count += 1

if retry_count >= 3:
    # 不再修正 B1 假设。回 Phase 1（Scout）获取新 trace 数据
    # 在函数 yaml 中标注: verify_stalled: true, stalled_reason: "3次修正未通过，需要更多 A 层数据"
    # 生成 Noise Card 记录完整失败链
```

超过 3 次 → **禁止继续修改 Lift 产物**。问题不在 B1 假设，在 A 层数据不足。必须回到 Scout 获取新 trace。

### .cpp 自包含性要求

Lift 产出的 .cpp 文件应尽量自包含。如果函数依赖外部符号:

1. **依赖已知函数** → 在 .cpp 顶部添加 stub 声明，标注 `// STUB: 来自 foo.dll:0x<addr>, 待合并编译`
2. **依赖全局变量/结构体** → 从 IDA/dump 中提取定义，作为独立头文件放在 `.artifacts/<binary>/types/`
3. **依赖外部库** → 写入 `.artifacts/uncompilable_functions.yaml`，诚实声明"此函数独立编译不可行"

`uncompilable_functions.yaml` 格式:
```yaml
- function: speak
  address: 'foo.dll:0x1400077c0'
  uncompilable_reason: "依赖 AudioDevice::write（0x140008000）和 Animal::vftable（0x140007000）"
  missing_deps:
    - type: function
      name: AudioDevice::write
      address: 'foo.dll:0x140008000'
      lift_status: not_started
    - type: vtable
      name: Animal::vftable
      address: 'foo.dll:0x140007000'
  suggested_path: "先 Lift AudioDevice::write → 再尝试编译 speak"
```

### Noise Card 生成

当 test anchor 失败时，自动生成 Noise Card 写入 `.artifacts/noise_cards.json`:

```json
{
  "noise_id": "lift-speak-001",
  "timestamp": "2026-06-18T12:00:00Z",
  "trigger": "speak(Animal(), 5)",
  "function_name": "foo.dll:Animal::speak",
  "observed": "返回值 0，但 anchor 期望返回值 1",
  "expected": "volume=5 应进入 AudioDevice::write 分支，返回 1",
  "anchor_violated": "volume=5 → 分支进入 AudioDevice::write 路径, eax=1",
  "discovery": "F5 对间接调用 [rcx+8] 的还原可能错误——5 次 trace 中有 2 次不调用 AudioDevice::write",
  "curriculum": "间接调用目标不能在静态分析中确定——需要动态 trace 确认虚函数表实际内容",
  "tags": ["indirect-call", "vtable", "F5-mistranslation"],
  "language": "cpp"
}
```

### @i_dont_know 的生命周期

```
创建（Lift步骤6）
    │
    ▼
标注到 yaml（uncertain_areas 字段）
    │
    ▼
90天无更新 → staleness 警告（Practify 内置机制）
    │
    ▼
动态调试 / 新 trace 数据到来
    │
    ├── 证实 → 转为 @pract.test
    └── 推翻 → 生成 Noise Card → 修正 Lift 产物
```

### 验证结果与置信度联动

| 验证结果 | status 变化 | 下一步 |
|---------|------------|--------|
| 所有 test 通过 | `draft` → `candidate` | 进入 Judge 审查 |
| 部分 test 失败 | 保持 `draft` | 回 Phase 2 修正 |
| 仅有 @i_dont_know | `draft` → `candidate`（附带 uncertain_areas） | 进入 Judge，标注待补充trace |
| 用户手动 confirmed | → `confirmed` | 归档

### Fan-out 并行调度模式

当面对一个不确定性高的分析任务时（如"这个函数到底是 CRC32 还是 djb2？"），
不要串行尝试N个假设，而是用 fan-out 并行探索:

**触发条件:**
- 对同一个函数/结构有多种互斥假设（3个以上候选算法/类结构）
- 单次对话无法同时验证所有假设
- 需要多维度证据才能排除假设

**调度流程:**
```
主Agent
  ├── 创建假设 A → WorkerA (并行)  → 产物 speak.b1.yaml (CRC32假设)
  ├── 创建假设 B → WorkerB (并行)  → 产物 speak.b2.yaml (djb2假设)
  └── 创建假设 C → WorkerC (并行)  → 产物 speak.b3.yaml (自定义哈希假设)

全部完成后:
  主Agent → Judge → 对比 .b1 .b2 .b3
  Judge  → 审查意见 → 推荐最优候选
  用户   → 拍板 → 优胜者复制为 speak.yaml（活跃版本）
                    被淘汰的 .bN 保留备查
```

**实现方式:**
- 在 Claude Code 中，用多个 Agent tool 调用来模拟 Worker 并行
- 每个 Worker 独立工作，不共享上下文（隔离）
- Worker 只看到自己的假设指令，不受其他假设影响
- Judge 收到所有 .bN 产物后统一对比

**注意事项:**
- Fan-out 适合互斥假设，不适合互补假设（互补假设应该合并而非竞争）
- 如果证据同时支持多个假设 → 不要强选，标注 candidate，等更多证据
- .bN 文件命名从 b1 开始递增，不管是否被淘汰都保留（审计追溯）

### 版本管理命名规则

- `<name>.yaml` — 当前活跃版本（主文件）
- `<name>.v1.yaml` — 时间线历史版本（旧主文件改名为 .vN）
- `<name>.b1.yaml` — fan-out 候选分支（多假设同时探索时各产出一个 .bN）
- 升级流程: 旧主文件改名 .vN → 新候选确认为活跃 → 复制为 .yaml

### .investigations/ — 思维链管理

每个 investigation 任务一个目录:

```yaml
---
编号: 001
任务: <一句话描述>
任务类型: 协议分析 | 算法还原 | 脱壳 | 调试 | 行为分析 | 定位 | patch
模式档位: 轻量 | 重量   # 重量启动前必须用户同意
状态: 进行中 | 已结案 | 已放弃
创建时间: <ISO日期>
更新时间: <ISO日期>
---
```

产物:
- 任务.md — 任务简报（Phase 0 写）
- 假设A-*.md / 假设B-*.md — 多假设并行探索
- 发现*.md — 勘探过程中的阶段性发现
- 待挖清单.md — 待继续深入的点
- 结论.md — 结案后写

## ═══════════════════════════════════════════════════════════
## 行为准则
## ═══════════════════════════════════════════════════════════

### 启动时（Phase 0 — 架构设计）

- 先判断是否有 CLAUDE.md 和 .artifacts/（已有项目 vs 新项目）
- **新项目 / 新任务: 必须先完成 Phase 0 架构设计，再动手**
- 已有项目: 读取 .investigations/ 了解进度，从上次断点继续
- 如果已有架构设计但未完成，先更新架构再继续

### 架构设计时（Phase 0）

- 自动评估复杂度 → 给出轻量/重量建议
- 轻量: 自评估后生成简版架构计划，给用户确认，≤3个要点
- 重量: 进入计划模式，与用户充分讨论后再出架构设计文档
- 未经用户确认架构，不得开始实际分析工作
- 执行中架构需要变更 → 暂停，先更新架构

### 勘探时（Phase 1 — Scout）

- 按架构设计中的任务拆解进行勘探
- 产物只写入 .investigations/，不写 .artifacts/
- 勘探发现只做标记和分类，不做精确还原
- 发现与架构预期不符 → 暂停，回 Phase 0 更新架构
- 发现知识库有缺口 → 新建或更新 knowledge/ 文件
- **发现可复用的模式/特征 → 立即写入 knowledge/discovered/ + 更新 INDEX.md**

### 分析时（Phase 2 — Worker）

- 分析前先查 knowledge/INDEX.md，检查已有知识
- 每个独立分析任务先创建 investigation 目录和任务.md
- 产物严格按格式写入 .artifacts/
- 任何产出都要标记 status: draft 或 candidate
- 遇到重大发现或需要方向决策时主动打断，不要闷头跑偏
- 如果产生多种假设，用 fan-out 方式并行探索，分别产出 .bN 文件
- **确认新算法特征/编译器惯用法/F5bug → 立即写入 knowledge/discovered/**
- **Lift 步骤4 必须提取测试向量 → 步骤6 必须产出 _anchors.py（如果启用Practify）**

### 验证时（Phase 2.5 — Verify）

- Lift 产出后立即运行 `practify test` 验证 anchor
- 全部通过 → status 升 candidate，进入 Judge
- 部分失败 → 判断是 anchor 错还是 Lift 错，修正后重跑
- @i_dont_know 项 → 写入 yaml 的 uncertain_areas，生成 Noise Card
- 不跳过此 Phase 直接 Judge（除非项目明确禁用了 Practify）

### 审查时

- 对比多个候选（如果存在），给出推荐和理由
- 检查 vtable 完整性、调用约定一致性、跨引用合理性
- **检查 test anchor 覆盖率和 Noise Card 历史**
- 只出审查意见，不改 status，等用户拍板
- **审查中发现的通用模式/规律 → 写入 knowledge/discovered/（置信度可标注 candidate）**

### 收尾时

- 更新 index.yaml
- 写结论.md
- 清理多余的候选文件（保留 vN 历史，删不需要的 bN）
- 汇总未完成项写入各 investigation 的待挖清单

### 禁止行为

- ❌ 直接把 F5 结果当最终答案
- ❌ 产物只留在对话里不落盘
- ❌ 自己把 status 改成 confirmed
- ❌ 在没有用户确认的情况下删除旧产物
- ❌ 在一个对话里塞入超过 2000 行汇编不拆分
- ❌ 跳过 Lift 步骤直接给结论
- ❌ @pt 缺少 source 字段（trace/memory 来源）— 缺 source 的 test anchor 视为凭空编造，Judge 直接驳回
- ❌ @i_dont_know 写得模糊（必须具体到可验证的条件）
- ❌ 同一函数 Lift→Verify 循环超过 3 次仍在修改 B1 假设 — 必须先回 Scout 拿新 A 层数据
