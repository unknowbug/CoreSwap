# P-A：ch0 跨语言通道级闭合（260903-06）

- **课题**：lossless-accel / P-A「ch0 跨语言通道级闭合」
- **status**：**confirmed（260903-06 用户拍板；judge 已过，CONCERN 清偿后提交 dc865fe）**——数据层三方对拍全绿 + 假残差定案有 C++ oracle 逐点背书。
- **前包**：260903-05（commit 80c9e95）；fan-out 三候选 x2-ch0-b{A,B,C}-260903-05.md 合流：GPU 可信、transpiler cache_2d 过度缓存（density.rs:335）致 ch0 列常量化；残留 idk「macro vs GPU ch0 残差 0.03-0.23」。
- **验证分层声明**：**Partial（oracle 探针级）**——新 oracle 工具（density_probe.cpp `-dfDump` 模式）+ GPU dump（bin-diag ch0_gpu_dump.rs）+ Rust macro 直采，通道级逐点数值对拍；非 final 层 block_probe 逐位 Full 口径。
- **§9.7 验证可比性声明（三要素）**：
  1. **载体**：C++ CPU DF 库点采样 oracle（delegate cell min-corner 点值 ≡ interp d000，即角点 fx=fy=fz=0）vs GPU shader ch0 通道 dump vs Rust macro 生产默认 sampler；
  2. **覆盖面**：3 列 × 48 角点 = 144 点（列 (4,16) / (3208,3208) / (-36,-76)，y=-64..319 步 4），f32 口径，major 阈值 1e-4，seed 8576294172403134396；
  3. **与既有口径可比性**：本口径为**通道级新口径**（ch0 单通道逐点），与既有 final 层 3.128e-07（final density 组合后）与 block 99.99%（端到端逐位）**不同层级、不可直接数值比较**——本包首次建立通道级 oracle 基线。

## 一、数据层事实（原始输出可引用）

- **oracle 构建**：density_probe.cpp 新增 `-dfDump` 模式（任意 DF JSON 点采样列，y=-64..319 步 4）；`.tmp/ch0_probe.json` = ch0 delegate（blend_density 复合树，从 overworld.json final_density **程序化提取**——避免转录错）。C++ CPU ch0 oracle = delegate 在 cell min-corner 点值 ≡ interp d000（角点 fx=fy=fz=0）。
- **GPU ch0 dump**：`WorldgenRust/src/bin-diag/ch0_gpu_dump.rs`（bin-diag，非正式 bin，临时区纪律合规）。
- **三方对拍结果**（对比脚本 `.tmp/ch0_compare.py` / `.tmp/ch0_macro_compare.py`；原始输出 `.investigations/lossless-accel/cmd-output/ch0-cpp-dump-{c0-4-16,c200-3208-3208,cneg-m36-m76}-260903-06.txt` + `ch0-gpu-dump-260903-06.txt`）：

| 对比对 | 点数 | major_diff(>1e-4) | max_diff | 判读 |
|---|---|---|---|---|
| GPU ch0 vs C++ oracle | 144（3 列 × 48 角点） | **0** | 1.795e-6（f32 ULP 级，worst @ (4,24,16)） | GPU ch0 = C++/Java 语义 ✅ |
| Rust macro（生产默认）vs C++ oracle | 48 | **0** | 5.0e-7 | macro ch0 通道级正确 ✅ |
| transpiler vs C++ oracle（修复前） | 48 | **28/48** | 0.2299 | y≥32 段纯线性化，步进 0.246875 = YClampedGradient 线性分量（现象确认；机制归因见 §15.4 第二条——真根因为漏设 blended_noise）❌→已修复 |

## 二、假残差定案（supersedes 候选）

260903-05 主记录（route2-260903-05.md「ch0 三方新事实」节）把 GPU @ (4,80,**z=0**)=-1.096655 与 macro @ (4,80,**z=16**)=-1.216371 混为同列对拍，得出「macro vs GPU 残差 0.03-0.23」残留 idk。本包 C++ oracle 证实**两点各自精确正确**：

- C++ @ (4,80,0) = **-1.0966554**（≡ GPU -1.096655）
- C++ @ (4,80,16) = **-1.2163714**（≡ macro -1.216371）

即「macro vs GPU ch0 残差 0.03-0.23」为**探针坐标混列假象**（z=0 与 z=16 是同一 x,y 不同 z 的两点，非同点两实现差）。macro 与 GPU 通道级均 = C++/Java 语义。

## 三、§15.4 结论取代链条目（草稿，主会话应用）

```yaml
supersedes:
  target: "route2-260903-05.md 残留 idk：macro ch0 vs GPU ch0 残差 0.03-0.23，跨语言 ch0 通道级残差未闭合"
  reason_one_line: "坐标混列假象——GPU 取 z=0、macro 取 z=16 被误作同点对拍；C++ oracle 证实两点各自精确正确（C++ @ (4,80,0)=-1.0966554、@ (4,80,16)=-1.2163714），macro 与 GPU 通道级均=C++/Java 语义"
  evidence: ".artifacts/lossless-accel/pa-ch0-closure-260903-06.md（260903-06）+ cmd-output/ch0-cpp-dump-* / ch0-gpu-dump-260903-06.txt"
```

（原结论正文按 §15.4 不删不改；本条为取代记录草稿。）

### §15.4 第二条 supersedes（P-B 误归因部分取代，260903-06 更新）

```yaml
supersedes:
  target: "x2-ch0-bA-260903-05.md 判定：Rust transpiler ch0 y 依赖丢失 = transpiler_cache_2d(id, x, z, || y=0) 闭包整列复用致生成物常量化（部分取代）"
  reason_one_line: "丢失机制 = worldgen_handle.rs 构造 NoiseSet 漏设 blended_noise（sample_blended_noise 返回 0.0），非生成物闭包压平——单点隔离探针（bin-diag/ch0_single_point.rs，设 blended_noise 后直接调生成函数）@ (4,80,16)=-1.216371343、(4,200,16)=-5.059816 等全部命中正确值（生成函数本身正确），清 C2D_CACHE 不改变 handle 环境错误值（排除缓存污染）"
  retained: "bA 子结论保留：macro_sampler 忠实、GPU ch0 无罪、ch0 异常仅 transpiler 路径"
  evidence: ".artifacts/lossless-accel/pa-ch0-closure-260903-06.md（260903-06 P-B 更新）+ 修复后 cmd-output/ch0-macro-vs-transpiler-fixed-260903-06.txt"
```

## 四、ch0 缺陷影响面（最终版，P-B 修复后定案，260903-06 更新）

| 路径 | 通道级判定 | 说明 |
|---|---|---|
| C++ final shader / GPU 通道（含 ch0） | **无缺陷** | 144 点 major=0，max=1.795e-6（f32 ULP 级） |
| Rust macro_sampler（生产默认） | **无缺陷** | 48 点 major=0，max=5.0e-7 |
| Rust transpiler | **缺陷 = 漏设 blended_noise（已修复 260903-06）** | 真根因：worldgen_handle.rs 构造 TranspilerDensity 的 NoiseSet **漏设 blended_noise**（old_blended_noise = base_3d_noise）→ sample_blended_noise 返回 0.0 → ch0 丢 base_3d 分量，列扫呈 depth*factor 线性。前包 bA「生成物 cache_2d y=0 闭包压平」为误归因（见 §15.4 第二条取代）。修复 = 新增 build_transpiler_noises()（DoublePerlin 集 + blended_noise，scale/factor/smear 数据驱动自 base_3d_noise.json，octave -15/-7 legacy 为 Java 构造器固定参数），WG_TRANSPILER 与 WG_GPU_CHANNELS fallback 两处构造共用 |
| GpuChannelDensity CPU fallback | **随源头修复一并正确** | 源头修复后 fallback=TranspilerDensity 已正确，前包「fallback 改绑 DfcDensity」方案不再需要 |

即：ch0 通道级缺陷**仅存于 transpiler 构造路径一处（漏设 blended_noise），已修复**；macro 生产路径与 GPU 通道均无缺陷。修复后验证：ch0_macro_vs_transpiler 全列 diff=0.000000（transpiler≡macro）；gpu_channel_probe 5 chunks × 5 通道对角 major=0（f32 ULP 级 max 1.96e-5 @ ch0 far chunk）+ combine 抽样 major=0 → 整体 PASS，**WG_GPU_CHANNELS 生产门解锁**。原始输出：cmd-output/ch0-macro-vs-transpiler-fixed-260903-06.txt、gpu-channel-probe-fixed-260903-06.txt。

## 五、错误记录（五段式）——workflow-patterns 发现 #17 草稿：三方对拍混列假残差

- **现象**：260903-05 三方 ch0 对拍得出「macro 与 GPU 各差 0.03-0.23、三值互不相等」，并派生残留 idk「跨语言 ch0 通道级残差未闭合」——实际数值两两各自与 C++ oracle 精确吻合（≤5e-7 / ≤1.8e-6），残差不存在。
- **根因**：对拍脚本把 GPU @ (4,80,**z=0**) 与 macro @ (4,80,**z=16**) 当同点比较——**跨探针数值比对未先钉死「同列同坐标」**，打印/罗列坐标 ≠ 实际采样坐标；两个真值不同（z 差 16 在该列差 ~0.12）被读成两实现差。
- **定位**：新建 C++ CPU oracle（-dfDump 逐点采样）后**逐点复核**——发现「GPU 值 = C++ @ z=0」「macro 值 = C++ @ z=16」，各点均能在 oracle 中找到精确对应点，混列即暴露。对照 oracle 逐点复核是识别混列的廉价手段（一轮对拍脚本成本）。
- **修复**：本包以 C++ oracle 为共同基准重建三方对拍（ch0_compare.py 统一坐标），假残差以 §15.4 取代记录定案；探针判据层面，任何跨工具对比先打印并断言两侧坐标序列逐点一致。
- **教训（可复用判据）**：**跨探针数值比对 MUST 先钉死同列同坐标**——打印坐标 ≠ 采样坐标时结论无效；「两个实现互不相等且差值随点漂移」与「坐标错位」的现象签名高度相似，先怀疑后者（与 seed/坐标三查铁律同族：符号/量级级差异先查对位再查实现）。对照独立 oracle 逐点复核是识别混列的最廉价手段。

### knowledge/discovered/workflow-patterns.md 追加文本（草稿，主会话应用）

```markdown
## 发现 #17: 跨探针对比坐标钉死律——打印坐标≠采样坐标时结论无效，oracle 逐点复核识别混列（260903-06）

- **时间/置信度/module**：260903-05→06，candidate，re-code/swe 通用。
- **来源定位**：lossless-accel P-A ch0 通道级闭合（pa-ch0-closure-260903-06.md）；手段 = density_probe -dfDump C++ CPU oracle + ch0_compare.py 统一坐标重建对拍。
- **观察**：三方 ch0 对拍「macro vs GPU 残差 0.03-0.23」实为 GPU @ (4,80,z=0) 与 macro @ (4,80,z=16) 混列——两点各自与 C++ oracle 精确吻合（≤5e-7/1.8e-6），残差不存在。构建 oracle 后逐点复核：每个"差异值"都能在 oracle 中找到**另一个坐标**的精确对应 → 混列暴露。
- **判据**：① 跨探针数值比对第一动作 = 断言两侧坐标序列逐点一致（打印不算、要断言）；② 「两实现互不相等、差值随点漂移」与坐标错位签名同构，先查对位（seed/坐标三查同族）；③ 对照独立 oracle 逐点复核是识别混列的廉价手段（一轮脚本成本）。
- **同族**：#12 对拍对象错级、#13 sanity check、#15 配对采样边界——本条补「坐标维度对位」。**补充要点（P-B，260903-06）：判别探针前置——静态归因先做单点隔离复测**：bA 的闭包压平归因源自静态括号配平+生成物结构审查，未先做「绕过外层环境直接单点调用生成函数」的隔离复测；单点隔离探针一轮即可证伪结构层归因（本例生成函数全对，错在构造环境）。静态归因（配平/结构审查）只能出候选，候选消费前 MUST 单点隔离复测。
```

## 五.2 错误记录 #2（五段式）——「坑记录在诊断侧未吸收进生产构造路径」+「静态配平归因误判」

- **现象**：transpiler ch0 列扫呈 depth*factor 纯线性（major 28/48，max 0.2299），260903-05 fan-out .bA 归因为「cache_2d y=0 闭包整列复用致生成物常量化」（density.rs:335），据此规划 cache_2d 失效策略 + fallback 改绑 DfcDensity。
- **根因（两层）**：① 直接根因 = worldgen_handle.rs 构造 TranspilerDensity 的 NoiseSet **漏设 blended_noise**（old_blended_noise = base_3d_noise）→ sample_blended_noise 返回 0.0 → ch0 丢 base_3d 分量，列扫天然线性；② 流程根因 = 该坑在 diag bin 注释里早有记载（transpiler_slices_ch0.rs:33、transpiler_alignment_expanded.rs:45「漏设则 ch0 系统性偏差」），但**只记在诊断工具侧，生产构造路径未吸收**——写坑的人没把坑修在生产里，读代码的人没读诊断注释。
- **定位**：P-B 单点隔离探针（WorldgenRust/src/bin-diag/ch0_single_point.rs：设 blended_noise 后单点调用生成函数）@ (4,80,16)=-1.216371343、(4,200,16)=-5.059816 等全部命中正确值 → 生成函数本身正确，闭包压平归因被证伪；清 C2D_CACHE 不改变 handle 环境错误值 → 排除缓存污染 → 收敛到构造环境差异（NoiseSet 缺 blended_noise）。
- **修复**：worldgen_handle.rs 新增 build_transpiler_noises()（DoublePerlin 集 + blended_noise，scale/factor/smear 从 base_3d_noise.json 数据驱动读取，octave -15/-7 legacy 为 Java 构造器固定参数），WG_TRANSPILER 与 WG_GPU_CHANNELS fallback 两处构造共用；fallback 改绑 DfcDensity 方案废弃。修复后 transpiler≡macro 全列 diff=0，gpu_channel_probe PASS。
- **教训（两条可复用判据）**：① **坑记录在诊断侧 ≠ 已修**——诊断注释里的已知坑 MUST 同步进生产构造路径（或建回归探针），否则坑对生产路径持续生效且被后来者重新排查一遍；② **静态配平归因须经单点隔离探针证伪/证实**——结构审查（括号配平、生成物 diff）只能出候选机制，消费前 MUST 先做「隔离单变量直接复测目标函数」的一轮廉价探针（与发现 #17 补充要点同族）。

## 六、docs/10 时间线 260903-06 节草稿（简短，主会话应用）

```markdown
## 260903-06（lossless-accel P-A：ch0 跨语言通道级闭合——假残差定案 + transpiler 唯一缺陷定论）

### ✅ C++ CPU ch0 oracle 建立 + 三方对拍全绿
density_probe -dfDump（delegate 程序化提取自 overworld.json，避转录错）+ bin-diag ch0_gpu_dump：3 列 × 48 角点，GPU vs C++ major=0 max=1.795e-6（f32 ULP 级）；macro vs C++ major=0 max=5e-7 → GPU 与 macro 生产路径通道级均 = C++/Java 语义。✅

### ✅ 假残差定案（supersedes 260903-05 残留 idk）
「macro vs GPU ch0 残差 0.03-0.23」= 探针坐标混列假象（GPU 取 z=0、macro 取 z=16 误作同点）；C++ oracle 证实两点各自精确正确（(4,80,0)=-1.0966554 / (4,80,16)=-1.2163714）。✅ 已结案

### ✅ transpiler 缺陷复确认（正确坐标下）
transpiler vs C++ major=28/48，max=0.2299，y≥32 纯线性化步进 0.246875（YClampedGradient 线性分量）——缺陷确认在 transpiler 路径。✅

### ✅ P-B 真根因定案 + 修复（supersedes bA 闭包压平归因）
真根因 = worldgen_handle.rs NoiseSet 漏设 blended_noise（sample_blended_noise 返 0.0）；单点隔离探针（ch0_single_point.rs）证伪 cache_2d 闭包压平归因（生成函数全对、清缓存无效）。修复 = build_transpiler_noises()（blended_noise 数据驱动，两处构造共用）。修复后 transpiler≡macro 全列 diff=0；gpu_channel_probe 5×5 通道 major=0 + combine 抽样 major=0 → PASS，**WG_GPU_CHANNELS 生产门解锁**；fallback 改绑 DfcDensity 方案废弃。坑先前只记在 diag 注释侧（transpiler_slices_ch0.rs:33 等）未吸收进生产构造——错误台账 #2。✅ 已结案

### 📌 记录指引
- 取代链条目（§15.4 两条）+ workflow-patterns 发现 #17（跨探针对比坐标钉死律 + 单点隔离复测补充要点）草稿见 .artifacts/lossless-accel/pa-ch0-closure-260903-06.md。
- 状态：candidate（judge 待过）；ch0 通道级 oracle 基线已建立，WG_GPU_CHANNELS 门解锁待 judge + 用户确认。
```
