# .b2 决定性实验设计（交主会话执行；worker 只定模板不跑命令）

前置事实（本 worker 已核实，2026-09-05）：
- `.tmp\jungle-l-260905-13\wgdiag-firstrun.log`（19MB）**只有 [SEEDLOG]/[SQX]/[SQZ] 级输出，无任何树级 decorator dump**（全日志 0 处 TREEDIAG/TreeDecorator 命中）→ Java 侧树集合 dump 通道**尚不存在，需加探针**。
- Rust 侧现有钩子：`WG_TREEDIAG`（placement.rs:281-305 `[CNT]/[SQ]`、tree.rs:541-542 `[TH]`、:438-439 `[BEE-MISS]`）——**无 chunk 过滤、无 trunk_set/leaves_set 序 dump**。`WG_FEATURELOG`（worldgen_handle.rs:526/989/1085）只有 feature 级 placed 行。`bin\tree_dump.rs` 是 density 树结构 dump，与本课题无关。

## §1 Rust 侧：单树集合序 dump（需 1 处小改 + 命令模板）

改动建议（主会话应用，env 门控零常开成本）：在 `worldgen-core\src\tree.rs:587`（decorator 循环前）加：

```rust
if crate::placement::treediag_enabled() {
    eprintln!("[TREESET] t=({}, {}, {}) trunk={}", bx, by, bz,
        trunk_set.iter().map(|p| format!("{}:{},{}", p[0], p[1], p[2])).collect::<Vec<_>>().join("|"));
    eprintln!("[TREELEAF] t=({}, {}, {}) leaves={}", bx, by, bz,
        leaves_set.iter().map(|p| format!("{}:{},{}", p[0], p[1], p[2])).collect::<Vec<_>>().join("|"));
}
```

构建与采集（seed 8576294172403134396，chunk (29,-16) = blocks 464..479, -256..-241）：

```powershell
cargo build --offline -p worldgen --release
$env:WG_TREEDIAG="1"                       # [CNT]/[SQ]/[TH]/[TREESET] 全开；可配合现有 chunk 过滤或在分析时 grep
# 按现有采集入口生成目标 chunk（主会话用既定 server/block_probe 流程；WG_DIAGCHUNK 若已接线则设 29,-16）
# stderr 落盘：
... 2> .investigations\jungle-l\cmd-output\b2-rust-treeset-chunk29-16.log
```

预期输出格式（分析判读对象）：
```
[TH] 8 @ (470,?, -250)                       ← 树位置 + 高度
[TREESET] t=(470,?,-250) trunk=470:80,-250|470:81,-250|...   ← 插入序，分号改冒号防与现有格式混淆
[TREELEAF] t=(470,?,-250) leaves=468:84,-252|...             ← 插入序
[CNT]/[SQ] 现有行照旧
```

## §2 Java 侧：同 dump 通道（需新探针）

对 `.tmp\scout-260905-08` 对应的探针工程加 mixin/patch（主会话按既有 WgDiag 注入方式），落点 = `TreeDecorator.Generator` 构造器末尾（TreeDecorator.java:52 之后）：

```java
if (WgDiag.enabled && WgDiag.chunkMatch(xref)) {
    WgDiag.out("[TREESET] t=" + java + " trunk=" + join(logPositions));   // 打印 Y 排序后最终 list 序
    WgDiag.out("[TREELEAF] t=" + java + " leaves=" + join(leavesPositions));
}
```

采集命令沿用 260905-13 的 `-Ptreediag=1 -Pdiagchunk=29,-16` 模板（wgdiag_firstrun.ps1 同目录），stderr 落
`.investigations\jungle-l\cmd-output\b2-java-treeset-chunk29-16.log`。

**免 mixin 的廉价旁证实验**（验证 §1.2 确定性结论，jshell/单文件 java 均可）：
按同插入序往 HashSet 塞一组 BlockPos，打印迭代序，连跑 2 次 + 换 Java 版本跑 1 次——预测三次全同（同插入序列 → 同桶序）。

## §3 判读判据（每条一行，可核对）

- **C1（J2 成立为贡献项）**：同一树两侧 `[TREESET]/[TREELEAF]` 同 Y 内序不一致，且该树存在碰撞几何（leaves 中同 Y、水平相距 2 的叶对），且两侧该树 vine 放置格数差 ≥1 → J2 实证贡献，量 = Σ 每树差。
- **C2（J2 对该树排除）**：同 Y 内序不一致但两侧 vine/cocoa 放置格集逐位相同（tie 只差 face/序、无链碰撞）→ 该树 0 贡献。
- **C3（J2 全排除）**：目标 chunk 全部树两侧同 Y 内序一致（如每 Y 唯一元素）或 vine 格集全等 → J2 出局，残差归因收敛到 J1/J3/J4。
- **C4（J2 降级为次要）**：region chunk 数 N 满足 12055/N ≫ 150（§3 域上界）→ 即使 C1 成立，J2 也只占小头，主嫌疑转 J1（先修）/J4（口径核对）。
- **C5（降级声明）**：Java 侧 dump 拿不到 → 只有 §2 旁证实验 + Rust 侧序 dump，结论降级 Degraded，占比不可归因（@anchor.idk 挂起）。

## §4 执行顺序建议

1. J4 口径核对（最廉价，scout 已排）先确认 region 计数是 id 还是 state → 决定 face/AGE 差是否计入 −12055。
2. J1 先修（确定性 trunk_set.push 缺失，scout A5）——**注意：J1 修复会改变 Rust RNG 流，J2 实验必须在 J1 修复后的代码上跑**，否则集合 dump 本身被 J1 污染。
3. 本实验（§1 Rust + §2 Java）→ §3 判据归因。
