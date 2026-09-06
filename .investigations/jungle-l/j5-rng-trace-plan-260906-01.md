# J5-③④ 单棵 mega jungle 树 RNG 流逐消费 trace 方案（j5-rng-trace-plan-260906-01）

> 角色：re-code/swe 混合 worker；模板/判据风格沿用 b2-experiment.md；响应 judge C1（judge-verdict-260905-13.md）。
> 状态：**draft**（方案与代码草稿均未编译/未运行验证——沙箱无 shell，采集由主会话执行）。
> 目标：两侧在 mega jungle 树放置路径上逐随机消费打点，对拍定位**第一处消费分歧**，核销/证实「上游 RNG 流漂移下的相位重排」draft 解读。

## 〇、一手源码核对记录（禁止凭记忆，逐行已核）

| 参照 | 路径 | 关键行 |
|---|---|---|
| MegaJungleTrunkPlacer | `E:\PYTHON\MC\data\mc_src_extract\net\minecraft\world\gen\trunk\MegaJungleTrunkPlacer.java` | :31-53 generate；:37 for(i=height-2-nextInt(4); i>height/2; i-=2+nextInt(4))；:38 f=nextFloat\*(float)(π\*2)；:42-47 l=0..5 枝干 getAndSetState；:49 TreeNode(startPos.add(j,i,k),-2,false) |
| GiantTrunkPlacer | 同目录 `GiantTrunkPlacer.java` | :30-50 generate（dirt 四角 :34-37 + setLog 2×2 柱 :40-47）；:52-65 setLog→**trySetState** |
| TrunkPlacer | 同目录 `TrunkPlacer.java` | :54-56 getHeight；:62-66 setToDirt（static；consume iff forceDirt‖!canGenerate）；:68-86 getAndSetState（5 参版委托 6 参版；canReplace 在 trunkProvider.get **之前**，拒绝不消费）；:88-92 trySetState（canReplaceOrIsLog 门，拒绝则**不进** getAndSetState）；:94-96 canReplace |
| TreeFeature | `...\gen\feature\TreeFeature.java` | :53 static canReplace；:101 getTopPosition 用 config.trunkPlacer.canReplaceOrIsLog |
| Rust | `worldgen-core\src\tree.rs` | generate :536-612（[TH] :549）；mega_jungle_trunk :618-666（set_to_dirt ×4 :621-624；place_log 闭包 :626-632；i 初值 :644；角度 :646；枝干 l 循环 :649-661；步长 :663）；set_to_dirt :685-698 |
| Java 探针 | `runtime\1.20.1\java\src\main\java\wg\bench\` | WgDiag.java（curAllowed/banner）；mixin\TrunkPlacerMixin.java（[THJ] getHeight RETURN，chunk 过滤已验证） |

**消费点清单（mega 路径，两侧应一一对应）**：
getHeight 2 draws →（TreeFeature 前置 0 消费）→ setToDirt ×4（各条件消费 dirtProvider.get）→ 2×2 柱 trySetState→getAndSetState（ok 时 trunkProvider.get）→ nextInt(4)（i 初值）→ 每轮：nextFloat（角度）→ l=0..4 枝干 getAndSetState（ok 时消费）→ nextInt(4)（步长）→ 循环条件 i>height/2。

## §1 Rust diff 草稿（tree.rs，env 门控 `crate::placement::treediag_enabled()`，[MJT] 前缀）

行号以当前工作区文件为准（260905-13 J1 修复后）。

**A1 [MJT0] 树起点 + 高度**——mega_jungle_trunk 函数体最前（:620 注释行后、:621 set_to_dirt 前）插入：
```rust
if crate::placement::treediag_enabled() {
    eprintln!("[MJT0] t=({}, {}, {}) h={}", sx, sy, sz, height);
}
```

**A2 [MJTD] set_to_dirt 消费可见性**——set_to_dirt（:685-698），在 :695 `if self.force_dirt || !is_soil_not_grass_myc {` 之前插入：
```rust
if crate::placement::treediag_enabled() {
    eprintln!("[MJTD] p=({}, {}, {}) soil={} fd={}", x, y, z, is_soil_not_grass_myc, self.force_dirt);
}
```
（消费 iff `fd || !soil`，分析时推导；所有树型都会打，靠 chunk 过滤 + MJT0 邻近锚定。）

**A3 [MJTL] 2×2 柱 place_log**——闭包（:626-632）改为：
```rust
let mut place_log = |tx: i32, ty: i32, tz: i32, trunk_set: &mut Vec<[i32; 3]>| {
    let ok = can_replace(ctx, tx, ty, tz);
    if crate::placement::treediag_enabled() {
        eprintln!("[MJTL] p=({}, {}, {}) ok={}", tx, ty, tz, ok);
    }
    if ok {
        let state = self.trunk_provider.get(random);
        ctx.set_block(tx, ty, tz, state);
        trunk_set.push([tx, ty, tz]);
    }
};
```
⚠️ 注意此处**无 canReplaceOrIsLog（\|\| LOGS）检查**，与 Java trySetState（GiantTrunkPlacer.java:64）层级不同——预登记候选 C-R1（见 §5）。

**A4 [MJTI] i 初值 + 每轮循环顶 i**——:644 改为：
```rust
let mut i = height - 2 - random.next_int_bound(4);
while i > height / 2 {
    if crate::placement::treediag_enabled() {
        eprintln!("[MJTI] i={} h={}", i, height);
    }
```
（Java 侧对拍点取 nextFloat 前 capture，格式一致；ni0 = h-2-i 分析时推导。）

**A5 [MJTF] 角度**——:646 改为：
```rust
let f = random.next_float() as f32 * two_pi;
if crate::placement::treediag_enabled() {
    eprintln!("[MJTF] f={} bits={:#x}", f, f.to_bits());
}
```

**A6 [MJTB]/[MJTB-R] 枝干 l 循环**——:654-660 改为：
```rust
let ok = can_replace(ctx, px, py, pz);
if crate::placement::treediag_enabled() && !ok {
    eprintln!("[MJTB-R] p=({}, {}, {}) l={}", px, py, pz, l);
}
if ok {
    let state = self.trunk_provider.get(random);
    ctx.set_block(px, py, pz, state);
    trunk_set.push([px, py, pz]);   // J1 修复保留
    if crate::placement::treediag_enabled() {
        eprintln!("[MJTB] p=({}, {}, {}) l={}", px, py, pz, l);
    }
}
```

**A7 [MJTS] 步长**——:663 改为：
```rust
let ni1 = random.next_int_bound(4);
i -= 2 + ni1;
if crate::placement::treediag_enabled() {
    eprintln!("[MJTS] i_new={} h={}", i, height);
}
```

**A8 [MJTX] 收尾**——:665 `nodes` 返回前插入：
```rust
if crate::placement::treediag_enabled() {
    eprintln!("[MJTX] nodes={} t=({}, {}, {})", nodes.len(), sx, sy, sz);
}
```

[TH]（:549）与 [THJ] 已对拍，不动。Rust stderr → `2>` 落盘（§3）。

## §2 Java mixin 草稿（runtime/1.20.1/java，wg/bench）

签名均按一手源码核对；未编译验证（见 §5 风险）。

**§2.1 新文件 `wg/bench/mixin/MegaJungleTrunkPlacerMixin.java`**：
```java
package wg.bench.mixin;

import java.util.List;
import java.util.function.BiConsumer;
import net.minecraft.block.BlockState;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.random.Random;
import net.minecraft.world.TestableWorld;
import net.minecraft.world.gen.feature.TreeFeatureConfig;
import net.minecraft.world.gen.foliage.FoliagePlacer;
import net.minecraft.world.gen.trunk.MegaJungleTrunkPlacer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** j5-rng-trace-plan-260906-01：mega 树逐消费 [MJT0]/[MJTI]/[MJTF]/[MJTS]/[MJTX]。 */
@Mixin(MegaJungleTrunkPlacer.class)
public abstract class MegaJungleTrunkPlacerMixin {
    private static boolean wg$on() {
        return Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null;
    }
    private static boolean wg$gate() {
        if (!wg$on()) return false;
        if (wg.bench.WgDiag.curAllowed()) { wg.bench.WgDiag.banner(); return true; }
        return false;
    }

    // generate 入口：树起点 + height
    @Inject(method = "generate(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;ILnet/minecraft/util/math/BlockPos;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)Ljava/util/List;",
            at = @At("HEAD"))
    private void wg$mjt0(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random,
                         int height, BlockPos startPos, TreeFeatureConfig config,
                         CallbackInfoReturnable<List<FoliagePlacer.TreeNode>> cir) {
        if (wg$gate()) {
            System.out.println("[MJT0] t=(" + startPos.getX() + ", " + startPos.getY() + ", " + startPos.getZ() + ") h=" + height);
        }
    }

    // 每轮枝干：nextFloat 调用前 capture i（此时 i 已赋值、j=k=0）
    @Inject(method = "generate(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;ILnet/minecraft/util/math/BlockPos;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)Ljava/util/List;",
            at = @At(value = "INVOKE", target = "Lnet/minecraft/util/math/random/Random;nextFloat()F", ordinal = 0))
    private void wg$mjti(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random,
                         int height, BlockPos startPos, TreeFeatureConfig config, int i,
                         CallbackInfoReturnable<List<FoliagePlacer.TreeNode>> cir) {
        if (wg$gate()) {
            System.out.println("[MJTI] i=" + i + " h=" + height);
        }
    }

    // 每轮步长 nextInt(4)（ordinal 1）调用前 capture i（旧值）与 f（本轮角度，已赋值）
    @Inject(method = "generate(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;ILnet/minecraft/util/math/BlockPos;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)Ljava/util/List;",
            at = @At(value = "INVOKE", target = "Lnet/minecraft/util/math/random/Random;nextInt(I)I", ordinal = 1))
    private void wg$mjtf(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random,
                         int height, BlockPos startPos, TreeFeatureConfig config, int i, float f,
                         CallbackInfoReturnable<List<FoliagePlacer.TreeNode>> cir) {
        if (wg$gate()) {
            System.out.println("[MJTF] f=" + f + " bits=" + Float.floatToIntBits(f));
            System.out.println("[MJTS] i_old=" + i + " h=" + height);
        }
    }

    // 收尾：node 数（几何细节由 MJTI/MJTB 对拍，不取 TreeNode 内部字段降低编译风险）
    @Inject(method = "generate(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;ILnet/minecraft/util/math/BlockPos;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)Ljava/util/List;",
            at = @At("RETURN"))
    private void wg$mjtx(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random,
                         int height, BlockPos startPos, TreeFeatureConfig config,
                         CallbackInfoReturnable<List<FoliagePlacer.TreeNode>> cir) {
        if (wg$gate()) {
            System.out.println("[MJTX] nodes=" + cir.getReturnValue().size() + " t=(" + startPos.getX() + ", " + startPos.getY() + ", " + startPos.getZ() + ")");
        }
    }
}
```
locals capture 说明：`wg$mjti` 的额外 `int i` 按局部变量槽序匹配（方法参数之后第一个 int 局部 = i）；`wg$mjtf` 的 `(int i, float f)` 同理。nextInt ordinal：for-init（:37 前件）=0，for-update（步长）=1（字节码序）。

**§2.2 扩展 `wg/bench/mixin/TrunkPlacerMixin.java`**（追加 3 个 @Inject，消费/拒绝点）：
```java
    // setToDirt（static，TrunkPlacer.java:62-66）：位置打点，消费 iff forceDirt||!canGenerate（分析推导，与 Rust [MJTD] 同口径）
    @Inject(method = "setToDirt(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;Lnet/minecraft/util/math/BlockPos;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)V",
            at = @At("HEAD"))
    private static void wg$mjtd(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random,
                                BlockPos pos, TreeFeatureConfig config, org.spongepowered.asm.mixin.injection.callback.CallbackInfo ci) {
        if (wg.bench.WgDiag.curAllowed() && (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null)) {
            System.out.println("[MJTD] p=(" + pos.getX() + ", " + pos.getY() + ", " + pos.getZ() + ") soil=? fd=?");
        }
    }

    // trySetState（TrunkPlacer.java:88-92）：canReplaceOrIsLog 层入口（拒绝时下方 MJTG 不出现）
    @Inject(method = "trySetState(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;Lnet/minecraft/util/math/BlockPos$Mutable;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)V",
            at = @At("HEAD"))
    private void wg$mjtt(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random,
                         BlockPos.Mutable pos, TreeFeatureConfig config, org.spongepowered.asm.mixin.injection.callback.CallbackInfo ci) {
        if (wg.bench.WgDiag.curAllowed() && (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null)) {
            System.out.println("[MJTT] p=(" + pos.getX() + ", " + pos.getY() + ", " + pos.getZ() + ")");
        }
    }

    // getAndSetState 5 参版（TrunkPlacer.java:68-70 委托 6 参）：ok=true ⇒ 消费 trunkProvider.get 1 次
    @Inject(method = "getAndSetState(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;Lnet/minecraft/util/math/BlockPos;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)Z",
            at = @At("RETURN"))
    private void wg$mjtg(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random,
                         BlockPos pos, TreeFeatureConfig config, CallbackInfoReturnable<Boolean> cir) {
        if (wg.bench.WgDiag.curAllowed() && (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null)) {
            System.out.println("[MJTG] p=(" + pos.getX() + ", " + pos.getY() + ", " + pos.getZ() + ") ok=" + cir.getReturnValue());
        }
    }
```
对拍映射：Java `MJTT→(MJTG ok)` 链 ↔ Rust 单行 `[MJTL] ok=`；Java 枝干直呼 `MJTG ok` ↔ Rust `[MJTB]/[MJTB-R]`；Java `MJTD soil=? fd=?`（拿不到内部值）↔ Rust `MJTD soil= fd=`（分析以 Java 侧 MJTD 后是否有 dirtProvider 消费不可直接见——消费计数对拍以「MJTD 行数 ×4 + get 消费推导」处理，见 T6 注）。

纪律：全部打点走 `WgDiag.curAllowed()`（chunk 过滤）+ 首行 `banner()`；未设 `-Pdiagchunk` 时全量输出（旧行为）。格式与 Rust 逐字段对齐（含逗号后空格）。**mixin json 注册**：`MegaJungleTrunkPlacerMixin` 需加入 client/主 mixins json 清单（主会话应用时核对既有 `"mixin": [...]` 列表）。

## §3 采集命令模板（主会话执行，沿用 wgdiag_firstrun.ps1 流程；seed 8576294172403134396，chunk (29,-16)）

**Rust 侧**：
```powershell
cd E:\PYTHON\CoreSwap
cargo build --offline -p worldgen --release
$env:WG_TREEDIAG = "1"                    # [TH]/[MJT*]/[TREESET] 全开
# 若 WG_DIAGCHUNK 已在 Rust 侧接线：$env:WG_DIAGCHUNK = "29,-16"；否则采集后 grep 过滤
# 按既有 chunk 采集入口生成目标 chunk（同 b2 §1 流程），stderr 落盘：
#   ... 2> .investigations\jungle-l\cmd-output\j5-rust-mjt-chunk29-16.log
```
**Java 侧**（= wgdiag_firstrun.ps1 改三处：mixin 新增、日志名、其余不动——seed 三查/删 world/forceload 流程照抄）：
```powershell
$env:GRADLE_USER_HOME = "E:\PYTHON\CoreSwap\.gradle-home"
$env:JAVA_TOOL_OPTIONS = "-Djava.io.tmpdir=E:\PYTHON\CoreSwap\.tmp\java-tmp -DWG_SEEDLOG=1 -DWG_TREEDIAG=1 -DWG_DIAGCHUNK=29,-16"
gradle runServer -PcppVanilla=1 -PseedLog=1 -Ptreediag=1 -Pdiagchunk=29,-16 --console=plain
#   stdout 落 .investigations\jungle-l\cmd-output\j5-java-mjt-chunk29-16.log（stderr .log.err）
# 核验（同 wgdiag 三核验）：banner [DIAG] 出现；SEEDLOG population 行全量；[MJT*]/[THJ] 行 OFF-target=0
```

## §4 判读判据（每条一行，可核对）

- **T0（有效性）**：两侧日志各含 ≥1 条 `[MJT0]`（Java 另含 `[DIAG]` banner）→ 采集有效；缺失 → 降级 Degraded（@anchor.idk 挂起）。
- **T1（同树对拍）**：两侧存在同起点 (x,y,z) 的 [MJT0] → 对该树按行序逐 token 对拍，**首个字段不同的 [MJT\*] 行 = 第一消费分歧点**（直接回 C1）。
- **T2（树位层先分歧）**：两侧 [MJT0] 起点集不相交（b2 已预期）→ 分歧在 placement/CNT/SQ 树位选择层，**先于**树内消费 → 本实验转 T5 锚定模式，C1 的「树内相位重排」表述不成立（重排在更上游）。
- **T3（getHeight 差）**：同起点树 [MJTI] i 不同而此前 [MJTD]/[MJTL] 行全同 → 分歧在 getHeight（核对该树 [TH]/[THJ] 值是否相等定夺 TH 层还是后置消费）。
- **T4（角度差）**：[MJTF] bits 不同而 [MJTI] i 相同 → nextFloat 消费相位/实现分歧（区分：bits 差但 f 接近 = 前置相位差；bits 结构性差 = 实现差）。
- **T5（锚定替代判据，起点集不相交时启用）**：各取**各自第一棵 mega**（[MJT0] 首行 + h），对比消费序列形状：MJTD 行数=4、柱 ok 计数、[MJTI] i 首值、[MJTF] 行数（枝干轮数）、[MJTB] 计数=5×轮数、[MJTX] nodes=轮数+1；**形状级差异 = 结构性差定位点**（形状全同 → 仅上游流相位差，B2 结论强化）。
- **T6（消费计数）**：每棵树 total draws = getHeight(2) + Σdirt-get(0..4) + 柱-ok 数 + 1(ni0) + 轮数×(1+f+Σ枝干ok) + 轮数(ni1)；两侧计数不同 → 计数差所在 token 即分歧点（注：Java MJTD 的 dirt-get 消费不可直接见，以柱/枝干 ok 与 TH 值先对拍，dirt 差作 residual）。
- **T7（canReplace 语义差）**：同 pos 一侧 ok=true 一侧 false（[MJTL] vs [MJTG]/[MJTB]）→ 拒绝判定语义差（air/log/vine 集），非 RNG 差；柱位系统性 ok 差 → 证实 C-R1。

## §5 风险与降级声明

1. **Java 同棵树不存在**（b2 已证同 chunk 树位集两侧不相交）→ T1 大概率不可用，主判据 = T5 锚定模式；此时结论定性为「形状/结构对照」而非逐消费对拍，降级 Partial。
2. **mixin locals capture 编译/运行风险**（ordinal/shift/局部槽序错配会在运行期抛错而非编译期）→ 最低保障集 = [MJT0]+[MJTD]+[MJTT]+[MJTG]+[MJTX]（纯 HEAD/RETURN，无 capture）；[MJTI]/[MJTF] 失败不阻塞 T5。
3. **C-R1（预登记候选）**：Rust place_log（:627）缺 `canReplaceOrIsLog`（Java trySetState 先过 `\|\| LOGS`）→ Rust 可能多放柱块并多消费 RNG；T7 柱位 ok 差即证实，属 J5-③ 放置成功性差候选。
4. **载体差**：Rust stderr vs Java stdout——各自落盘后合并分析；行序锚定靠 [MJT0]/[TH] 邻接。
5. **覆盖面边界**：本 trace 只覆盖 generate→trunk 放置路径（getHeight 前的消费、foliage/decorator/vine 消费不在内）；若 trunk 段全同，分歧在 foliage/decorator 段 → 转后续（J5-①② 的 bush 树数/尝试数候选仍独立有效）。
6. **BlockStateProvider 消费**：若 jungle trunk/leaf provider 为 weighted 型，provider.get 内部也消费 random——两侧类型同源自 JSON 应一致，但 T6 计数若差 1~2 且无其他解释，先查 provider 类型。
7. 本文件所有 mixin 代码**未编译验证**（沙箱无 shell）；方法签名已逐行核对一手源码，但 descriptors/locals 仍以首次 `gradle build` 为准。

## §6 错误→教训（可沉淀点）

- **拒绝检查层级也要逐行对拍**：J1 修复只对了消费顺序，漏了 Java `trySetState` 的 canReplaceOrIsLog 层（Rust place_log 缺 `\|\| LOGS`）——「逐行对拍」必须覆盖**判定分支**不只 RNG 消费序（教训 → 若 T7 命中即升级为课题结论）。
- **capture 点选在「值已赋值」的下一次调用前**，不是 INVOKE AFTER（AFTER 仍在 store 之前，局部变量取不到新值）—— mixin locals capture 的通用坑，值得进 knowledge/discovered（本文件先记，转正式条目待 judge）。
- b2 教训沿用：J1 修复改变 Rust RNG 流，本 trace 必须在修复后代码上采集（当前工作区即修复后，✓）。
