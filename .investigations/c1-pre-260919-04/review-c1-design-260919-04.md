# judge review：C-1 立项前置设计文档 design-c1-260919-04（260919-04）

- 审查角色：core.judge（subagent，只出意见不改 status）
- 审查对象：.investigations/c1-pre-260919-04/design-c1-260919-04.md（draft）
- 三源基线：scout-map.md（同目录）/ verdict-260919-03（confirmed）§7 / verdict-260919-02（confirmed）§4 C-1 行 / git HEAD aebf0f6 + 一手源码抽查

## 结论：PASS-with-conditions（推荐保持 draft）

## 独立抽查（3 处，全部核实）

1. fill 逐格纯查表+列内 col_max、邻接无关 — ✅ worldgen-core/src/light/mod.rs:508-545
2. try_spread 单调 new_level>light[n] — ✅ mod.rs:694-697（BFS 边界依赖 DOM 常量 mod.rs:657-674）
3. light_decode_packed_into chunk_bases 泛化 — ✅ mod.rs:229-248 + doc 注释 mod.rs:225-228

另核：9 中心循环 mod.rs:441-468、JNI copyin jni_bridge.rs:461、（C-4 thread_local 依 scout §4 + verdict-03 §4.2 旁证）。

## 逐项

- 设计断言 vs scout：全部有 file:line 依据，无超出勘探证据的机制断言 ✅
- 方向 vs verdict-03 §7.3：一致（C-1 为下一主收益面、本档即 G3 前置设计）✅
- 上界口径 vs verdict-02 §4 C-1 行：一致（13.51/37.9%、9.3ms/26%、≤59%），#191 双侧列账已列 ✅
- G3 保持论证：成立（共享件均为本帧快照 chunk 局部纯函数；BFS 每中心独立重算、Scratch 每相清零）✅
- C-1b 排除：充分（唯一安全形态=memoization；穷尽性声明建议补一句，非阻塞）
- 判据草案：两个 open 前置正确标 MUST 阻塞 ✅；缺 G3 收敛性门（S2）
- 过度声明：未发现（上界措辞、静态推演如实标注、status=draft）

## 条件清单

- S1 形态口径写死：C-1a 的 BFS 形态（9×3×3 窗 vs 5×5 单域）在 §0/§3 与 §1/§3.3/§4 间表述不一——Scratch ×2.78 / 装载新增成本 / 位等价门 open ② 仅对 5×5 单域形态成立。立项前 MUST 写死形态并逐条对齐判据与成本。
- S2 判据补 G3 收敛性门：verdict-260919-02 §7.3 明载「一次重载收敛不动点性质验证」为 C-1 专项验收前置，§4 表缺此项（重载 settle 复测 changed≈噪声地板）。
- S3 机制解释置信分级保留：12 篇 confirmed 的只是经验收敛链；「无状态纯函数」为静态推演/candidate 级——引用时保留分级。
- S4 RSS 判据化：Scratch 常驻 ~7.5MB/线程风险从散文升为判据行或预登记 idk（v0.22 §15.1）。

## 非阻塞建议

1. C-1b 段补「合法形态穷尽性系静态论证」声明。
2. 明确是否纳入 verdict-03 §7.2/idk-1 的「Scratch/b9 复用后重测 B-C4」。
3. 本目录（design+scout+本审查）尚未入库（aebf0f6 工作区 untracked），立项前建议 commit 固化。
