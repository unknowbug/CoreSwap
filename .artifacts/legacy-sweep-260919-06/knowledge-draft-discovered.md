# knowledge/discovered 追加草稿（260919-06 块；主会话应用，本文件只含草稿正文）

> 产出者：knowledge subagent（draft）；依据 judge-review-260919-06 落盘建议 ①②③④ + knot-static worker 件。
> 价值门核对：建议 ② 为中价值简记；建议 ① 为高价值错误优先；建议 ③ 中价值；建议 ④ 按 judge 原话「并入既有 §9.7/口径家族条目，若有则追加不新建」→ 落为 workflow-patterns #100 的追加注记，不新建条目。
> 编号现状：workflow-patterns 至 #194、build-tooling 至 #159、compiler-idioms 至 #27（主会话应用前请重核文件末尾，防并行块撞号）。
> 一次性结论（Scope B/C 的 19/8/10/1 diff 计数、B3 余额 2/4/3、CP-5 参数表数值等）按价值门**不进 discovered**，只进 10 时间线。

---

## 草稿 1（建议①，最高价值）→ build-tooling.md 追加「发现 #160」

### 发现 #160（最高价值·错误优先）: Sponge MixinProcessor 对已注册 mixin 类的 classload 是设计性拒绝（IllegalClassLoadError "cannot be referenced directly"）——三段链 = Sponge 拒绝点 → Knot 包装点 → 应用层吞 cause（260919-06）

- **发现时间 / 发现者 / 置信度 / module**：260919-06（2026-09-19）；core.worker（静态溯源）+ knowledge subagent 起草；**draft**（机制方向，未运行时验证；sponge-mixin 侧为字节码判读 = Degraded；升 candidate 条件 = getCause() 打点一次最小 run）；build-tooling / mixin 运行时失败机制面（#157 的机制层下沉，#176 的机制根）。
- **来源定位**：`.investigations/legacy-sweep-260919-06/knot-selftransform-static.md`（§2 逐点 file:line）；一手材料 = gradle cache fabric-loader 0.15.11 **sources jar**（强）+ sponge-mixin 0.13.3+mixin.0.8.5 **class jar javap 字节码判读**（Degraded，无 sources）。
- **五段式**：
  - **现象**：`Class.forName("wg.bench.mixin.BlobProbeMixin")` 反射读 stats → `RuntimeException: Mixin transformation of wg.bench.mixin.BlobProbeMixin failed`，日志里看不到底层 cause；注入/织入全程正常（boot 期 `handler active` 在位）。
  - **根因（机制，三段链）**：① **Sponge 拒绝点**（根）：`MixinProcessor.applyMixins` 内识别「被变换类 = 已注册 mixin 类/包成员」→ `getInvalidClassError` 取消息（三条分支：已注册 mixin 类 / accessor 变体 / mixin 包级兜底）→ `new IllegalClassLoadError(msg)` athrow——mixin 类本体被设计为「只能被织入引擎以字节码方式消费的中间体，禁止直接 classload 引用」，是显式设计分支而非兼容性问题；② **Knot 包装点**：`KnotClassDelegate.getPostMixinClassByteArray:422` 把所有经 Knot 的类（含 mixin 本体）无差别送 transformer，`:423-427` catch 后包装为 `RuntimeException("Mixin transformation of %s failed")`——顶层消息的出处；③ **应用层吞 cause**：`BlobProbe.java:46` 只打印 `+ t` 顶层消息，`.getCause()` 链未展开——cause 消失的落点在应用侧，不在框架侧。
  - **定位（怎么发现的）**：gradle cache 提一手 sources（fabric-loader）+ javap -c 常量池字符串直读（sponge-mixin 无 sources 时的 Degraded 判读法）——从顶层消息字符串反查 Knot 包装点 file:line，再进 transformer 字节码找拒绝分支与消息常量；**判错签名 = 「stats read failed + 只见顶层 Mixin transformation 消息」即直接展开 cause 链/查反射自载路径**，不要先疑版本兼容/字节码版本。
  - **修复**：计数器等可观测状态**外移普通 holder 类**（1.21.6 已照此修复 = `BlobProbeStats`；1.20.1 出货树用户拍板不动、登记已知限制）。**版本归属弱推断警告**：cache 单版本归属是间接证据，未读 loom lock file 精确核对（诚实声明形态）。
  - **教训（可复用判据）**：① mixin 类**不可被应用代码以任何形式直接引用**（Class.forName/直接 import 均触发）——诊断/计数状态一律放普通类；② 「cause 被吞」先查应用侧 catch 打印深度再怪框架；③ 家族分工：#40（json 失同步·装载面）、#157（两家族时点判据）、#176（修复模式）、**本条 = 机制根（拒绝点的源码级定位）**，四条合读。
- **家族索引**：#40 / #157 / workflow-patterns #176；本条为其机制层根。

---

## 草稿 2（建议②，中价值简记）→ build-tooling.md 追加「发现 #161 简记」

### 发现 #161 简记: 比较器抽样显示缺陷模式——对 str key 做序列切片/索引 → 显示失真但计数正确；「计数与显示分离核」一招（260919-06）

- **观察**：`cmp_t8.py:16` `f"{c[0]},{c[1]}"` 对字符串 key（"13,-1"）做序列切片 → sample 行显示为 "1,3"（坐标失真）；**计数与 VERDICT 基于完整 str key 集合计算，不受影响**（judge 独立复算 diff=4223 逐位吻合）。工具坑家族：Python 的 str 也是序列，对「本应是 tuple 的 key」做 `k[0],k[1]` 不报错——静默产出失真显示。
- **判据（可复用）**：① 比较器/对账工具的 sample/展示路径与计数路径是两条代码路，判读时**先分离核对**：抽 1 个 sample 行手工对原始 key 验显示正确性，再信计数；② 显示失真 ≠ 计数错误，反之亦然——交付时诚实登记缺陷层次（本例：显示层缺陷 + 计数不受影响 + 修复一行即可），不静默不夸大；③ 修复形态 = str key 直接打印 key 本身，不做解构。
- **来源定位**：`.artifacts/legacy-sweep-260919-06/record-lowcost-260919-06.md` §T8；judge-review-260919-06 §T8.5（N5）。置信度 candidate（实测单例 + 机制直读）。

---

## 草稿 3（建议③，中价值）→ workflow-patterns.md 追加「发现 #195」

### 发现 #195: 双臂对拍判据预登记模板四件套——preconditions 表（含负自证硬门）+ 存在性判据无阈值带 + VOID 非零退出 + 覆盖面/外推边界声明（260919-06，T8 实例化模板）

- **观察**：T8（域批值 vs vanilla 双臂对拍）全链程序合规且经 judge 独立复算验证，其判据形态可抽为可复用模板（`.tmp/legacy-sweep-260919-06/criteria-t8.md`）：
  1. **preconditions 表**：每臂 key/expected/check 三元组——含**正自证**（本臂特性在位：lightInit=1/hook=1/dll sha）与**负自证硬门**（对照臂特性必缺：lightInit=0/[LightRust]=0/hook=0；#118）；
  2. **存在性判据优先**：首轮对拍用「diff>0 即 FAIL」的存在性判据，**无阈值带**——量级对比属解读层且 MUST 声明 §9.7 档位（见 #100 追加注记），机制归因留在判据层外；
  3. **VOID 分支机械表达**：前置不满足 `sys.exit(2)` 非零退出（#160 协议 / #172 门要能失败），不静默降级；
  4. **覆盖面/外推边界声明**：单 seed × 载具 × n=1 明示，不外推（「未证不可能」维持）。
- **判据（可复用）**：双臂对拍立项时按四件套预登记；判据/驱动/比较器**同批定稿且先于采集**（mtime 序可核，#112）；预登记读法被机械执行 = 无事后挑读法。与 §9.7（等价分层）/§15.1（前置集）/ #188（数据通道覆盖执行形态）家族互补——本条是「双臂对拍」场景的组装模板。
- **来源定位**：`.tmp/legacy-sweep-260919-06/criteria-t8.md` + `.artifacts/legacy-sweep-260919-06/record-lowcost-260919-06.md` §T8 + judge-review-260919-06 §T8.1-3。置信度 candidate（单实例模板化，未跨场景复用验证）。

---

## 草稿 4（建议④）→ workflow-patterns.md **既有 #100 追加注记**（不新建条目）

> 260919-06 追加（E1/E2 档位维）：量级旁证引用噪声带 MUST 声明档位差——E1（同构建态 run 间）噪声带**不是** E2（跨执行体）差异的上界；实例：T8 对拍「44.7% ≫ 0.8% 噪声带」句中 0.8% 为 G17 的 E1 带，本对拍为 E2，该句仅作「远超 run 级摆动」的量级示意、FAIL verdict 不依赖它（预登记存在性判据 diff>0）——judge N2 判定结论不受影响但说服力打折。判据：量级对比句若引用跨档位带，MUST 同行声明「档位不同、不作上界」，且主判据不依赖该句（存在性判据优先，#195）。来源：judge-review-260919-06 N2 + record §T8。

---

## 归类建议汇总（供主会话应用）

| judge 建议 | 载体 | 编号建议 | 价值档 |
|---|---|---|---|
| ① Sponge mixin 拒绝自 classload 链 | build-tooling.md | #160（最高价值） | 高（错误优先·五段式） |
| ② 比较器显示/计数分离坑 | build-tooling.md | #161 简记 | 中 |
| ③ 双臂判据预登记模板 | workflow-patterns.md | #195 | 中 |
| ④ E1/E2 噪声带档位 | workflow-patterns.md #100 追加注记 | 不新建 | 中（判据延伸） |

不进 discovered（价值门）：Scope B/C 重算计数、B3 余额表、CP-5 参数表数值、T7 零回退计数等一次性结论 → 10 时间线。
