package wg.bench;

import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.registry.Registries;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.Heightmap;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.gen.noise.NoiseConfig;
import wg.CppWorldgen;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

/*
 * ================================================================================================
 * 260912-01 D3 共享 Java 适配核 —— `wg.bench.CppBridge`（跨版本唯一共享源；"一个类只有一个家"）
 * ------------------------------------------------------------------------------------------------
 * 基线来源：versions/1.20.1/java/src/main/java/wg/bench/CppBridge.java（732 行，全文为底）。
 * 并入的 1.21.6 独有项（逐条；源锚点与目标锚点见
 * .investigations/shared-java-core-260912-01/scout-map.md §2.2）：
 *   B1 `rustFeaturesTakeover()` 谓词（源 1.21.6 :165-176）——共享后 1.20.1 侧无调用者
 *      （消费者 `mixin/ChunkGeneratorFeaturesMixin` 是 1.21.6 独有 mixin，留分版）⇒ 1.20.1 零行为影响；
 *   B2 `ChunkTiming` 分项钩子调用点（源 1.21.6 :385/:393 addJni、:399/:402 addScan、
 *      :406/:412 addWrite、:540/:548 addHmap）——共享 `ChunkTiming` 为两侧超集
 *      （1.20.1 的 `enter/exit`/`inflightEnter/Exit` 与 1.21.6 的 `addXxx`/`featTick` 同在一份）；
 *   B4 `MIXLOG` 文档信息（源 1.21.6 :30-35 的 R1 同族记载，并入基线注释；语义同、表述取信息量大者）；
 *   B5 `writeChunk` 内联 `stateById` ↔ 基线独立方法 `stateById` 语义相同（M14 注释同源），
 *      共享形态取基线（独立方法：`BulkWb` 依赖其包私有 `static` 可见性）。
 * 版本差异一律走 WgCompat（分版缝：versions/<ver>/java/src/main/java/wg/bench/WgCompat.java）：
 *   - A1a 写回跳空气默认开关 → `WgCompat.SKIPAIR_ON`（1.20.1=true 已验证 / 1.21.6=false 待翻转）；
 *   - C 线 bulk 写回默认开关 → `WgCompat.BULKWB_ON`（1.20.1=true 已验证 / 1.21.6=false 待自身 A/B 后翻转）；
 *   两处 property 解析统一 `WgCompat.flag(<prop>, <分版缺省>)`：未设 = 分版缺省；
 *   `-Dcoreswap.skipair=0|1` / `-Dcoreswap.bulkwb=0|1` 两版均可显式覆盖（同构建态单变量 A/B）。
 *   本文件**不含任何版本条件分支/映射差异写法**：无 `new Identifier`/`Identifier.of`、
 *   无 `getOrThrow`/`getEntry(RegistryKey)`、无版本判断、无第二套 API 形态（两版 registry/Identifier
 *   用法本文件内通用，scout §3.4 附 C 已核）。
 * 已批准的 1.21.6 行为变更（共享形态取基线 ⇒ 仅影响 1.21.6，注释内注明，**不加分支**）：
 *   A1b（overworld 生产侧 O(1) 短路 + 全量扫描/`buf-sparse` 移入 `MIXLOG`；
 *   nether/end 的 nzBuf 全量扫描 + 16 点读回整块移入 `MIXLOG`；A2 补 nether/end `[WG-CONTENT]`）；
 *   S-5（`StallWatch.start()` 接线，两版默认由 property 门控）。
 * 变更记录指针：
 *   .investigations/000-架构设计/架构计划-260912-01-共享Java适配核.md（§3 D-3 / §13 HOOK-1 / §14 V1b）
 *   .investigations/shared-java-core-260912-01/record-260912-01.md（V1b 声明）+ scout-map.md §2/§3。
 * ================================================================================================
 */

/**
 * CoreSwap worldgen 全局桥：持有 C++ 句柄，把 C++ 生成的整块写入 Chunk。
 * 启用：-Dcpp.replace=1（由 BenchMod 在 server started 时 init）。
 */
public final class CppBridge {
    private static volatile long handle;
    public static volatile boolean enabled;
    // 多世界（2026-08-30）：nether 维度句柄（min_y=0/height=256，nether.json + biome_params_nether.json）
    private static volatile long netherHandle;
    public static volatile boolean netherEnabled;
    // end 维度句柄（260906-04 end 接管：min_y=0/height=128，end.json；无 biome_params——非 MultiNoise）
    private static volatile long endHandle;
    public static volatile boolean endEnabled;
    private static final boolean DEBUG = System.getProperty("cpp.debug") != null;
    // 方块注册表缓存（RQ-005）：进程级静态，vanilla 注册表运行期冻结。
    // null = 未查过 registry（不能填 AIR：st==null 判断会永远 false 导致全写空气——历史根因）；
    // AtomicReferenceArray 保证并发可见性（多 worker 线程同时写同 id 同值，幂等安全）。
    private static final int MAX_ID = 4096;
    private static final java.util.concurrent.atomic.AtomicReferenceArray<BlockState> STATE_BY_ID =
            new java.util.concurrent.atomic.AtomicReferenceArray<>(MAX_ID);
    private static final BlockState AIR = Blocks.AIR.getDefaultState();
    // per-thread 输出 buffer（RQ-004）：M=1 无锁模型，每 worker 线程一个 16*16*384 int（~384KB）
    private static final ThreadLocal<int[]> BUF =
            ThreadLocal.withInitial(() -> new int[16 * 16 * 384]);
    /** 生成线程数（-Dcoreswap.threads=N 显式覆盖；否则模式自适应：
     *  服务端全核(-1)、客户端留 2 核(-2) 给渲染/主线程——Issue #7 + 用户设计） */
    private static final int THREADS = resolveThreads();

    private static int resolveThreads() {
        String explicit = System.getProperty("coreswap.threads");
        if (explicit != null) {
            try {
                return Integer.parseInt(explicit);
            } catch (NumberFormatException e) {
                return -1;  // 非法值兜底：服务端全核
            }
        }
        try {
            // 反射拿 Fabric 环境（编译期无 fabric-loader API 依赖；Forge+Connector 也可能没有）
            Object loader = Class.forName("net.fabricmc.loader.api.FabricLoader")
                    .getMethod("getInstance").invoke(null);
            Object env = loader.getClass().getMethod("getEnvironmentType").invoke(loader);
            return "SERVER".equals(env.toString()) ? -1 : -2;
        } catch (Throwable t) {
            return -1;  // 非 Fabric/未知：服务端全核兜底
        }
    }

    private CppBridge() {}

    // 句柄级阶段开关掩码（双跑修复 2026-09-08）：bit0=SKIP_CARVER bit1=SKIP_FEATURES。
    // 默认 0b011 = 存档链路关 Rust carver/features（这两阶段由 Java vanilla 执行，Rust 双跑 = confirmed 缺陷）。
    // -Dcoreswap.rust.stages=all 回旧行为（双跑对照）；其他值按 int 解析为原始 mask。
    private static int resolveStageMask() {
        String prop = System.getProperty("coreswap.rust.stages");
        if (prop == null || prop.isEmpty()) return 0b011;
        if ("all".equals(prop)) return 0;
        try { return Integer.parseInt(prop); } catch (NumberFormatException e) { return 0b011; }
    }

    public static void init(long seed) {
        String dir = System.getProperty("cpp.worldgen.dir");
        if (dir == null) dir = extractWorldgenDir();
        handle = CppWorldgen.init(seed, dir);
        enabled = handle != 0;
        if (enabled) {
            CppWorldgen.setFlags(handle, resolveStageMask());
        }
        System.out.println("[CppBridge] init seed=" + seed + " worldgenDir=" + dir + " enabled=" + enabled
                + " stageMask=" + (enabled ? CppWorldgen.getFlags(handle) : 0)
                + " env.CORESWAP_DEFAULT_BLOCK=" + System.getenv("CORESWAP_DEFAULT_BLOCK"));
        // 停滞看门狗（260910-06，-Dcoreswap.stallwatch=<秒> 才启动）：沙箱禁 JVM attach ⇒ 线程栈只能靠自打
        // 260912-01 D3：本接线取自 1.20.1 基线（1.21.6 原 `init()` 内无此调用），已批准进共享核——
        // 两版均由 property 门控默认关（scout §3.2 S-5）；共享源内不加版本分支。
        StallWatch.start();
        // 打印 dll 版本信息（排查旧缓存：用户加载的 dll 应与 jar 内的一致）
        try {
            java.nio.file.Path dllPath = java.nio.file.Path.of(CppWorldgen.getNativeLibraryPath());
            byte[] loaded = java.nio.file.Files.readAllBytes(dllPath);
            String sha = java.security.MessageDigest.getInstance("SHA-256").digest(loaded).length > 0
                    ? java.util.HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(loaded)) : "";
            System.out.println("[CppBridge] dll=" + dllPath + " size=" + loaded.length + " sha256=" + sha.substring(0, 16) + "...");
        } catch (Exception e) {
            System.out.println("[CppBridge] dll info failed: " + e);
        }
    }

    /**
     * 从 mod 内 worldgen-data/ 解压 C++ 所需 JSON 数据到临时目录（幂等：已存在即复用）。
     * 目标布局（对齐 C++ wg_create 的路径约定）：
     *   <tmp>/coreswap-data/worldgen/data/minecraft/worldgen/...  （JSON 数据）
     *   <tmp>/coreswap-data/blocks.json / biome_params.json      （wgDir/../ 查找）
     */
    private static String extractWorldgenDir() {
        // Forge+Connector 兼容：多级定位 jar（codeSource → ModOrigin → classloader）后 JarFile 提取
        return CoreSwapFixHelper.extractWorldgenDir();
    }

    private static void deleteRecursively(Path path) throws IOException {
        if (!Files.exists(path)) return;
        try (var stream = Files.walk(path)) {
            stream.sorted(java.util.Comparator.reverseOrder()).forEach(p -> {
                try { Files.deleteIfExists(p); } catch (IOException ignored) {}
            });
        }
    }

    /** 多世界：初始化 nether 维度句柄（失败不阻断主世界）。在 init 之后调用。 */
    public static void initNether(long seed) {
        try {
            String dir = System.getProperty("cpp.worldgen.dir");
            if (dir == null) dir = extractWorldgenDir();
            long h = CppWorldgen.initDim(seed, dir, "nether.json", "biome_params_nether.json", 256);
            netherHandle = h;
            netherEnabled = h != 0;
            if (netherEnabled) {
                CppWorldgen.setFlags(h, resolveStageMask());
            }
            System.out.println("[CppBridge] initNether seed=" + seed + " enabled=" + netherEnabled
                    + " stageMask=" + (netherEnabled ? CppWorldgen.getFlags(h) : 0));
        } catch (Throwable t) {
            System.out.println("[CppBridge] initNether failed: " + t);
        }
    }

    public static boolean netherActive() { return netherEnabled && netherHandle != 0; }

    /** end 维度：初始化句柄（260906-04）。end 无 biome_params（非 MultiNoise），空串占位。 */
    public static void initEnd(long seed) {
        try {
            String dir = System.getProperty("cpp.worldgen.dir");
            if (dir == null) dir = extractWorldgenDir();
            long h = CppWorldgen.initDim(seed, dir, "end.json", "", 128);
            endHandle = h;
            endEnabled = h != 0;
            if (endEnabled) {
                CppWorldgen.setFlags(h, resolveStageMask());
            }
            System.out.println("[CppBridge] initEnd seed=" + seed + " enabled=" + endEnabled
                    + " stageMask=" + (endEnabled ? CppWorldgen.getFlags(h) : 0));
        } catch (Throwable t) {
            System.out.println("[CppBridge] initEnd failed: " + t);
        }
    }

    public static boolean endActive() { return endEnabled && endHandle != 0; }

    /**
     * batchD（mc-1216 features 接管）：Rust features 接管判定——stageMask bit1（SKIP_FEATURES）清零时
     * Java features 让位（ChunkGeneratorFeaturesMixin 调用）。默认 mask=0b011 → 本判定 false（现状不变）；
     * -Dcoreswap.rust.stages=1（只 SKIP_CARVER）时 true = Rust features 接管 + Java vanilla 装饰器跳过，
     * 两侧同源 mask 驱动（单 flag 双向切换，无第三状态）。all（mask=0）语义 = 双跑对照（非接管），
     * bit1 清零含 0 → 本判定也 true——但 all 是显式对照意图，此处排除 0 防误判：
     * 接管判定严格 = mask 非零且 bit1 清零。
     *
     * <p>260912-01 D3 并入注记（scout §2.2 B1）：本谓词来自 1.21.6 独有项，共享后 1.20.1 侧
     * **无调用者**（消费者 `mixin/ChunkGeneratorFeaturesMixin` 是 1.21.6 独有 mixin，留分版）
     * ⇒ 1.20.1 零行为影响；谓词本体两版通用（只依赖共享 `resolveStageMask`）。
     */
    public static boolean rustFeaturesTakeover() {
        int mask = resolveStageMask();
        return enabled && mask != 0 && (mask & 0b010) == 0;
    }

    /**
     * mod 方块注册（260907-07 方案 C 接线；260907-08 候选 B 写回对齐）：-Dcpp.blockRegister 门控，默认关。
     * 有 java_raw 的方块（真实 mod 方块）→ registerBlockId 显式 id 注册（Rust id = Java raw id 同域，
     * 写回 Registries.BLOCK.get(id) 直查对齐，错位修复）；合成名冒烟回退（java_raw 不存在）→ 旧 registerBlock 动态 id。
     * 对齐自证：逐条打印 java_raw ↔ rust_id ↔ writeback_get（Registries.BLOCK.get(rust_id) 反查的名字），
     * 对齐成功时 writeback_get 应等于注册名。
     * 调用时序：init/initNether/initEnd 之后、任何 chunk 生成之前（Rust 创建期注册约定）。
     */
    public static void registerModBlocks() {
        String prop = System.getProperty("cpp.blockRegister");
        if (prop == null) return;  // 默认关
        int limit = -1;
        try { if (!prop.isEmpty()) limit = Integer.parseInt(prop); } catch (NumberFormatException ignored) {}
        int n = 0;
        for (net.minecraft.block.Block b : Registries.BLOCK) {
            net.minecraft.util.Identifier key = Registries.BLOCK.getId(b);
            if (key == null || "minecraft".equals(key.getNamespace())) continue;
            String name = key.toString();
            int javaRaw = Registries.BLOCK.getRawId(b);
            long[] handles = { handle, netherHandle, endHandle };
            String[] dims = { "overworld", "nether", "end" };
            StringBuilder ids = new StringBuilder();
            int ridOw = -1;
            for (int i = 0; i < handles.length; i++) {
                int rid = handles[i] != 0 ? CppWorldgen.registerBlockId(handles[i], name, javaRaw) : -1;
                if (i == 0) ridOw = rid;
                ids.append(dims[i]).append('=').append(rid).append(' ');
            }
            // 写回对齐自证（N5 改进，260907-09）：按 rust_id（而非 javaRaw）反查——
            // rust_id 是 Rust buf 实际携带的 id，写回链路用它直查注册表；id 域对齐时与 javaRaw 等价，
            // 错位时此行直接暴露（javaRaw 反查会掩盖错位，反查必须走链路真实消费的 id）。
            String writeback = "n/a";
            try {
                net.minecraft.block.Block wb = ridOw >= 0 ? Registries.BLOCK.get(ridOw) : null;
                writeback = wb == null ? "null" : Registries.BLOCK.getId(wb).toString();
            } catch (Throwable t) { writeback = "threw:" + t; }
            System.out.println("[BLOCKS-REG] " + name + " java_raw=" + javaRaw + " rust_id(" + ids.toString().trim() + ") writeback=" + writeback);
            n++;
            if (limit >= 0 && n >= limit) {
                System.out.println("[BLOCKS-REG] limit=" + limit + " reached, stop");
                break;
            }
        }
        System.out.println("[BLOCKS-REG] done count=" + n);
        if (n == 0 && !prop.isEmpty()) {
            // 冒烟回退（260907-07 合成；260907-08 候选 B 扩展）：环境无内容 mod 时，
            // ① aligned 探针 = 真实 vanilla raw id 显式注册（走候选 B registerBlockId 路径，
            //    机制等价真实 mod 方块：有 java_raw 的注册；断言 rust_id == java_raw 且写回反查还原本体）；
            // ② dynamic 探针 = 旧 registerBlock 动态 id 路径（合成名无 java_raw，保持不变，ABI 兼容哨兵）。
            int synth = 3;
            try { synth = Integer.parseInt(prop); } catch (NumberFormatException ignored) {}
            long[] handles = { handle, netherHandle, endHandle };
            String[] dims = { "overworld", "nether", "end" };
            // 候选 B 显式探针：模拟真实 mod raw id 位置 = Registries.BLOCK.size()（vanilla 表之后，
            // blocks.json max 1002 之外无冲突）。注：无 mod 时 Java 注册表同位置无条目 → writeback 反查为 null 属预期；
            // 断言核心 = rust_id == java_raw（id 域对齐）。260907-08 第一轮教训：用真实 vanilla 块（calcite raw=910）
            // 会被 blocks.json 同 id 正确拒绝——冲突拒绝语义按设计工作，探针必须用 vanilla 表后 id。
            int simulatedRaw = Registries.BLOCK.size();
            String alName = "testmod:aligned_probe";
            StringBuilder alIds = new StringBuilder();
            for (int h = 0; h < handles.length; h++) {
                int rid = handles[h] != 0 ? CppWorldgen.registerBlockId(handles[h], alName, simulatedRaw) : -1;
                alIds.append(dims[h]).append('=').append(rid).append(' ');
            }
            String wb = "n/a";
            try { net.minecraft.block.Block b2 = Registries.BLOCK.get(simulatedRaw); wb = b2 == null ? "null" : Registries.BLOCK.getId(b2).toString(); } catch (Throwable t) { wb = "threw"; }
            System.out.println("[BLOCKS-REG] " + alName + " java_raw=" + simulatedRaw + " rust_id(" + alIds.toString().trim() + ") writeback=" + wb + " mode=explicit");
            for (int i = 1; i <= Math.max(1, synth); i++) {
                String name = "testmod:probe_block_" + i;
                StringBuilder ids = new StringBuilder();
                for (int h = 0; h < handles.length; h++) {
                    int rid = handles[h] != 0 ? CppWorldgen.registerBlock(handles[h], name) : -1;
                    ids.append(dims[h]).append('=').append(rid).append(' ');
                }
                System.out.println("[BLOCKS-REG] " + name + " java_raw=N/A rust_id(" + ids.toString().trim() + ") mode=dynamic");
            }
        }
    }


    /** end 维度 Beardifier 喂入（end 城市结构；与 nether 同构） */
    public static void feedBeardifierEnd(Chunk chunk, net.minecraft.world.gen.StructureAccessor structures) {
        feedBeardifier(endHandle, chunk, structures);
    }

    /**
     * 喂 Beardifier（StructureWeightSampler）输入到 C++：在 populateNoise 拦截处、fillChunk 之前调用。
     * 用 vanilla createStructureWeightSampler 构造（结构与 Java 同源），反射提取 piece/junction 列表，
     * 序列化 int[] 传给 wg_set_beardifier。失败时降级：不喂数据（Beardifier=0，与现状一致），不阻断生成。
     */
    public static void feedBeardifier(Chunk chunk, net.minecraft.world.gen.StructureAccessor structures) {
        feedBeardifier(handle, chunk, structures);
    }

    /** nether 维度版（netherHandle） */
    public static void feedBeardifierNether(Chunk chunk, net.minecraft.world.gen.StructureAccessor structures) {
        feedBeardifier(netherHandle, chunk, structures);
    }

    // ===== BUG-001（260906 CoreSwap-Maint 卡）修复：双名反射解析 =====
    // Fabric tiny-remapper 只重映射 .class 引用，反射字符串字面量不映射：
    // 开发环境=Yarn 名（pieceIterator），生产 intermediary=field_28744 等。
    // 名字对 = 本工程 gradle 缓存 yarn-1.20.1+build.10 mappings.tiny 直查（非记忆）。
    private static volatile java.lang.reflect.Field fPieces, fJunctions;
    private static final java.util.concurrent.ConcurrentHashMap<String, java.lang.reflect.Method> WG_METHOD_CACHE =
            new java.util.concurrent.ConcurrentHashMap<>();
    private static final java.util.concurrent.atomic.AtomicLong WG_BEARD_FAILS = new java.util.concurrent.atomic.AtomicLong();
    private static final java.util.Set<String> WG_BEARD_FAIL_KEYS = java.util.concurrent.ConcurrentHashMap.newKeySet();

    private static java.lang.reflect.Field wgField(Class<?> c, String yarnName, String intermediaryName) throws NoSuchFieldException {
        if (yarnName.equals("pieceIterator") && fPieces != null) return fPieces;
        if (yarnName.equals("junctionIterator") && fJunctions != null) return fJunctions;
        NoSuchFieldException last = null;
        for (String n : new String[]{yarnName, intermediaryName}) {
            try { java.lang.reflect.Field f = c.getDeclaredField(n); f.setAccessible(true);
                if (yarnName.equals("pieceIterator")) fPieces = f;
                if (yarnName.equals("junctionIterator")) fJunctions = f;
                return f;
            } catch (NoSuchFieldException e) { last = e; }
        }
        throw last;
    }

    private static java.lang.reflect.Method wgMethod(Class<?> c, String yarnName, String intermediaryName) throws NoSuchMethodException {
        String key = c.getName() + '#' + yarnName;
        java.lang.reflect.Method cached = WG_METHOD_CACHE.get(key);
        if (cached != null) return cached;
        NoSuchMethodException last = null;
        for (String n : new String[]{yarnName, intermediaryName}) {
            try { java.lang.reflect.Method m = c.getMethod(n); m.setAccessible(true); WG_METHOD_CACHE.put(key, m); return m; }
            catch (NoSuchMethodException e) { last = e; }
        }
        throw last;
    }

    private static void feedBeardifier(long h, Chunk chunk, net.minecraft.world.gen.StructureAccessor structures) {
        if (h == 0) return;
        int cx = chunk.getPos().x, cz = chunk.getPos().z;
        try {
            net.minecraft.world.gen.StructureWeightSampler sws =
                    net.minecraft.world.gen.StructureWeightSampler.createStructureWeightSampler(structures, chunk.getPos());
            java.util.ArrayList<int[]> pieces = new java.util.ArrayList<>();
            java.util.ArrayList<int[]> junctions = new java.util.ArrayList<>();
            // ⚠️ 不得用 Class.forName(Yarn名)——生产 intermediary 下类名也不同（class_5817$class_7301 等），
            // 类引用必须走编译期直引（remapper 重写）或运行时 getClass()；方法查找用运行时实际类即可。
            java.lang.reflect.Field fp = wgField(sws.getClass(), "pieceIterator", "field_28744");
            java.lang.reflect.Field fj = wgField(sws.getClass(), "junctionIterator", "field_28745");
            fp.setAccessible(true);
            fj.setAccessible(true);
            // 名字对：Yarn ↔ intermediary（mappings.tiny build.10 实证：field_28744/28745、comp_682/683/684、
            // method_35415..35420（BlockBox）、method_16610/16611/16609（JigsawJunction））
            it.unimi.dsi.fastutil.objects.ObjectListIterator<?> pit = (it.unimi.dsi.fastutil.objects.ObjectListIterator<?>) fp.get(sws);
            while (pit.hasNext()) {
                Object piece = pit.next();
                Object box = wgMethod(piece.getClass(), "box", "comp_682").invoke(piece);
                int terrainOrd = ((Enum<?>) wgMethod(piece.getClass(), "terrainAdjustment", "comp_683").invoke(piece)).ordinal();
                int delta = (Integer) wgMethod(piece.getClass(), "groundLevelDelta", "comp_684").invoke(piece);
                pieces.add(new int[]{
                        (Integer) wgMethod(box.getClass(), "getMinX", "method_35415").invoke(box),
                        (Integer) wgMethod(box.getClass(), "getMinY", "method_35416").invoke(box),
                        (Integer) wgMethod(box.getClass(), "getMinZ", "method_35417").invoke(box),
                        (Integer) wgMethod(box.getClass(), "getMaxX", "method_35418").invoke(box),
                        (Integer) wgMethod(box.getClass(), "getMaxY", "method_35419").invoke(box),
                        (Integer) wgMethod(box.getClass(), "getMaxZ", "method_35420").invoke(box),
                        terrainOrd, delta});
            }
            it.unimi.dsi.fastutil.objects.ObjectListIterator<?> jit = (it.unimi.dsi.fastutil.objects.ObjectListIterator<?>) fj.get(sws);
            while (jit.hasNext()) {
                Object jj = jit.next();
                int sx = (Integer) wgMethod(jj.getClass(), "getSourceX", "method_16610").invoke(jj);
                int sy = (Integer) wgMethod(jj.getClass(), "getSourceGroundY", "method_16611").invoke(jj);
                int sz = (Integer) wgMethod(jj.getClass(), "getSourceZ", "method_16609").invoke(jj);
                junctions.add(new int[]{sx, sy, sz});
            }
            if (pieces.isEmpty() && junctions.isEmpty()) {
                CppWorldgen.setBeardifier(h, cx, cz, null, 0, null, 0);  // 清空该 chunk（防上一批残留）
            } else {
                int[] p = new int[pieces.size() * 8];
                for (int i = 0; i < pieces.size(); i++) System.arraycopy(pieces.get(i), 0, p, i * 8, 8);
                int[] j = new int[junctions.size() * 3];
                for (int i = 0; i < junctions.size(); i++) System.arraycopy(junctions.get(i), 0, j, i * 3, 3);
                CppWorldgen.setBeardifier(h, cx, cz, p, pieces.size(), j, junctions.size());
            }
        } catch (Throwable t) {
            // 降级：不喂数据（Beardifier=0）。日志节流（BUG-001 止血）：同 key 首次全打 + 每 1000 次汇总 1 条
            long n = WG_BEARD_FAILS.incrementAndGet();
            String key = t.getClass().getSimpleName() + ":" + t.getMessage();
            if (WG_BEARD_FAIL_KEYS.add(key)) {
                System.out.println("[CppBridge] feedBeardifier failed chunk(" + cx + "," + cz + ") total=" + n + ": " + t);
            } else if (n % 1000 == 0) {
                System.out.println("[CppBridge] feedBeardifier failed x" + n + " (last chunk " + cx + "," + cz + "): " + key);
            }
        }
    }

    /**
     * 用 C++ 结果整块填充 Chunk（NOISE 阶段的方块 + 高度图）。
     * 并发模型（RQ-001~004 改造，2026-08-11）：M=1 非空即处理，无全局锁——
     * 每个 mixin worker 线程直接调 JNI fillBlocks（JNI 本身多线程安全；C++ 池已改任务队列
     * 模型，批间也真并行）；writeChunk 写独立 Chunk 对象天然并行。per-thread buffer 消除共享池。
     */
    /**
     * per-chunk {@code [WG-FILL]} 读回自证行门控（260910-06，1.21.6 260910-05 R1 同族拉平）：
     * nether/end 的该行为**无条件 println**，在 23 线程并发下是 log4j（同步 appender）串行点，
     * 也是生产 jar 的真实成本。默认关（生产语义），诊断 {@code -Pmixlog=1} / {@code -Dcoreswap.mixlog=1} 打开。
     * 260911-05 A1b 修正（judge C2）：原注释「只门控 println，nzBuf 扫描与 16 点读回仍每 chunk 执行」
     * 已**失效**——三者（nzBuf 全量扫描、16 点读回、[WG-CONTENT] 指纹）现统一由本门控整块控制；
     * overworld 的「整块全空气」异常信号改由 O(1) 短路探测保留（见 fillChunk）。
     *
     * <p>260912-01 D3 并入注记（scout §2.2 B4 + §2.1 A1b）：1.21.6 同族记载为「4225 chunks ⇒ 4225 次
     * log4j 同步 appender 写入，与 overworld R1 修的是同一类病理」——语义与本注释同，取本注释（信息量大者）。
     * 1.21.6 原形态（overworld 无条件 98304 次全量扫描 + 无条件 println；nether/end 无条件 nzBuf 扫描 +
     * 16 点读回、仅 println 门控）在共享形态下**整块移入门控** = 已批准的 1.21.6 行为变更（A1b），
     * 共享源内不加分支。
     */
    private static final boolean MIXLOG = System.getProperty("coreswap.mixlog") != null;

    /**
     * 原生输出**逐 chunk 内容指纹**（260910-06 行为门专用，MIXLOG 门控）：对 {@code fillBlocks} 写回的
     * 整个 buffer 算 FNV-1a 64，打印 {@code [WG-CONTENT] chunk(x,z) hash=<hex> nz=<n>}。
     *
     * <p>存在理由：1.20.1 的 region 逐块对拍被**运行级非确定**（Java carver/feature 顺序，实测同形态
     * run-to-run 0.021-0.047%）淹没，跨形态差（0.058-0.065%）落在噪声带附近 ⇒ 该载体门**检验力不足**。
     * 而 R3 只改「重活在哪个线程跑」，若 Rust 侧 per-chunk 输出是纯函数，则两形态应给出**逐 chunk 相同**
     * 的指纹——这是噪声无关的强判据（同时也能暴露 Rust 侧是否真有跨 chunk 状态）。
     */
    private static long wgBufHash(int[] buf) {
        long h = 0xcbf29ce484222325L;  // FNV-1a 64 offset basis
        for (int v : buf) {
            // 逐 int 混入（含 0 = air，位置敏感：同一个值出现在不同下标得到不同 hash）
            for (int b = 0; b < 4; b++) {
                h ^= (v >>> (b * 8)) & 0xFF;
                h *= 0x100000001b3L;   // FNV prime
            }
        }
        return h;
    }

    public static void fillChunk(Chunk chunk) {
        long h = handle;  // 本地快照：destroy 后置 0，拦截后续调用（不 use-after-free）
        if (!enabled || h == 0) return;
        int cx = chunk.getPos().x, cz = chunk.getPos().z;
        int[] buf = BUF.get();  // per-thread buffer（ThreadLocal，~384KB/worker，RQ-004）
        int got = 0;
        long tj0 = ChunkTiming.ON ? System.nanoTime() : 0L;   // B2 并入（1.21.6 :385）
        try {
            got = CppWorldgen.fillBlocks(h, new int[]{cx}, new int[]{cz},
                    new int[][]{buf}, THREADS);
        } catch (Throwable t) {
            System.out.println("[CppBridge] DIAG fillBlocks threw chunk(" + cx + "," + cz + "): " + t);
            return;
        }
        ChunkTiming.addJni(System.nanoTime() - tj0);   // B2 并入（1.21.6 :393）
        if (got != 1) {
            System.out.println("[CppBridge] DIAG fillBlocks got=" + got + " chunk(" + cx + "," + cz + ")");
            return;
        }
        // 生产侧 O(1) 短路探测（260911-05 A1b + judge C2）：整块全空气 = 「Rust 输出全 0」异常信号，
        // 原为无条件 println + 98,304 次全量计数。改为「遇到首个非零即停」的短路扫描：典型 buf[0] 非零
        // ⇒ 单次数组读即返回，既保留该信号又不吃热路径成本。原 `buf-sparse`（nz<1000）分级诊断随全量
        // 扫描一并移除（需要时用 -Pmixlog=1 的 nz 字段）。
        // 260912-01 D3 并入注记：1.21.6 原形态为**无条件** 98,304 次全量 nz 计数 + 无条件
        // println（`buf-all-air` / `buf-sparse` 两条）；共享形态取本基线（O(1) 短路保留 all-air 信号，
        // `buf-sparse` 随全量扫描移除）⇒ 1.21.6 侧为已批准的行为变更（A1b），共享源内不加分支。
        // ChunkTiming 门控口径随之变化：B2 的 `addScan` 仍在（1.21.6 :399/:402），但计量对象从
        // 「全量 nz 计数」变为「O(1) 短路探测」⇒ 1.21.6 的 [CHUNKTIME] `scan` 列将趋近 0（分项计时口径变更，
        // 属 A1b 的必然结果，非仪器缺陷；两臂同尺子仍然成立）。
        long ts0 = ChunkTiming.ON ? System.nanoTime() : 0L;   // B2 并入（1.21.6 :399）
        if (buf[0] == 0) {
            boolean allAir = true;
            for (int k = 1; k < buf.length; k++) if (buf[k] != 0) { allAir = false; break; }
            if (allAir) System.out.println("[CppBridge] DIAG buf-all-air chunk(" + cx + "," + cz + ")");
        }
        ChunkTiming.addScan(System.nanoTime() - ts0);   // B2 并入（1.21.6 :402）
        if (MIXLOG) {
            int nz = 0;
            for (int k = 0; k < buf.length; k++) if (buf[k] != 0) nz++;
            // 行为门指纹（260910-06）：应在 sync/async 两形态下逐 chunk 相同
            System.out.println("[WG-CONTENT] chunk(" + cx + "," + cz + ") hash="
                    + Long.toHexString(wgBufHash(buf)) + " nz=" + nz);
        }
        long tw0 = ChunkTiming.ON ? System.nanoTime() : 0L;   // B2 并入（1.21.6 :406）
        try {
            writeChunk(chunk, cx, cz, buf, 384);
        } catch (Throwable t) {
            System.out.println("[CppBridge] DIAG write threw chunk(" + cx + "," + cz + "): " + t);
        }
        ChunkTiming.addWrite(System.nanoTime() - tw0);   // B2 并入（1.21.6 :412）
    }

    // nether 维度：min_y=0/height=256（buffer 16*16*256），netherHandle 分派
    private static final ThreadLocal<int[]> BUF_NETHER =
            ThreadLocal.withInitial(() -> new int[16 * 16 * 256]);

    public static void fillChunkNether(Chunk chunk) {
        long h = netherHandle;  // 本地快照
        if (!netherEnabled || h == 0) return;
        int cx = chunk.getPos().x, cz = chunk.getPos().z;
        int[] buf = BUF_NETHER.get();
        int got = 0;
        try {
            got = CppWorldgen.fillBlocks(h, new int[]{cx}, new int[]{cz},
                    new int[][]{buf}, THREADS);
        } catch (Throwable t) {
            System.out.println("[CppBridge] DIAG fillBlocks(nether) threw chunk(" + cx + "," + cz + "): " + t);
            return;
        }
        if (got != 1) {
            System.out.println("[CppBridge] DIAG fillBlocks(nether) got=" + got + " chunk(" + cx + "," + cz + ")");
            return;
        }
        try {
            writeChunk(chunk, cx, cz, buf, 256);
        } catch (Throwable t) {
            System.out.println("[CppBridge] DIAG write(nether) threw chunk(" + cx + "," + cz + "): " + t);
        }
        // 读回自证 + 行为门指纹（260911-05 A1b：整块门控——nzBuf 扫描与 16 点读回只被门控内打印消费）
        // 260912-01 D3 并入注记：1.21.6 原形态 = nzBuf 全量扫描 + 16 点读回**无条件**执行、仅 println 门控
        // （1.21.6 :442-456，且无 [WG-CONTENT] 行）；共享形态整块移入门控 + 补 [WG-CONTENT]（A2）
        // ⇒ 1.21.6 侧为已批准的行为变更（热路径去诊断成本 + 新增门控内诊断行），共享源内不加分支。
        if (MIXLOG) {
            int nzBuf = 0;
            for (int v : buf) if (v != 0) nzBuf++;
            System.out.println("[WG-CONTENT] chunk(" + cx + "," + cz + ") hash="
                    + Long.toHexString(wgBufHash(buf)) + " nz=" + nzBuf);
            try {
                var s4 = chunk.getSection(4);
                int na = 0;
                for (int x = 0; x < 16; x += 4) {
                    for (int z = 0; z < 16; z += 4) {
                        if (!s4.getBlockState(x, 8, z).isAir()) na++;
                    }
                }
                System.out.println("[WG-FILL] chunk(" + cx + "," + cz + ") bufNonzero=" + nzBuf
                        + " readback=nonair " + na + "/16 status=" + chunk.getStatus());
            } catch (Throwable t) {
                System.out.println("[WG-FILL] readback threw: " + t);
            }
        }
    }

    // end 维度：min_y=0/height=128（buffer 16*16*128），endHandle 分派（260906-04 end 接管）
    private static final ThreadLocal<int[]> BUF_END =
            ThreadLocal.withInitial(() -> new int[16 * 16 * 128]);

    public static void fillChunkEnd(Chunk chunk) {
        long h = endHandle;  // 本地快照
        if (!endEnabled || h == 0) return;
        int cx = chunk.getPos().x, cz = chunk.getPos().z;
        int[] buf = BUF_END.get();
        int got = 0;
        try {
            got = CppWorldgen.fillBlocks(h, new int[]{cx}, new int[]{cz},
                    new int[][]{buf}, THREADS);
        } catch (Throwable t) {
            System.out.println("[CppBridge] DIAG fillBlocks(end) threw chunk(" + cx + "," + cz + "): " + t);
            return;
        }
        if (got != 1) {
            System.out.println("[CppBridge] DIAG fillBlocks(end) got=" + got + " chunk(" + cx + "," + cz + ")");
            return;
        }
        try {
            writeChunk(chunk, cx, cz, buf, 128);
        } catch (Throwable t) {
            System.out.println("[CppBridge] DIAG write(end) threw chunk(" + cx + "," + cz + "): " + t);
        }
        // 读回自证（end 高度 128 = 8 sections，取 section 2（y 32-47，中心岛顶面附近））
        // 260911-05 A1b：整块门控（nzBuf 扫描与读回只被门控内打印消费）
        // 260911-05 A2：补 `[WG-CONTENT]` 指纹行——overworld/nether 早有、end 缺失
        // （260911-05 A2 首轮实证：end 臂 intercepted=4761 而 contentLines=0），全维行为门要求三维同载体。
        // 260912-01 D3 并入注记：1.21.6 原形态 = nzBuf 全量扫描 + 16 点读回**无条件**执行、仅 println 门控
        // （1.21.6 :486-500，且无 [WG-CONTENT] 行）；共享形态整块移入门控 + 补 [WG-CONTENT]（A2）
        // ⇒ 1.21.6 侧为已批准的行为变更（热路径去诊断成本 + 新增门控内诊断行），共享源内不加分支。
        if (MIXLOG) {
            int nzBuf = 0;
            for (int v : buf) if (v != 0) nzBuf++;
            System.out.println("[WG-CONTENT] chunk(" + cx + "," + cz + ") hash="
                    + Long.toHexString(wgBufHash(buf)) + " nz=" + nzBuf);
            try {
                var s2 = chunk.getSection(2);
                int na = 0;
                for (int x = 0; x < 16; x += 4) {
                    for (int z = 0; z < 16; z += 4) {
                        if (!s2.getBlockState(x, 8, z).isAir()) na++;
                    }
                }
                System.out.println("[WG-FILL] chunk(" + cx + "," + cz + ") bufNonzero=" + nzBuf
                        + " readback=nonair " + na + "/16 status=" + chunk.getStatus());
            } catch (Throwable t) {
                System.out.println("[WG-FILL] readback threw: " + t);
            }
        }
    }

    // 直写 PalettedContainer（跳过 chunk.setBlockState 的 heightmap/blockEntity 开销）
    // 泛化维度（2026-08-30 多世界）：height 参数（overworld 384/24 sections；nether 256/16 sections）。
    // Chunk.getSection(int) 是 0-based 索引（相对维度 bottomY；buf[0] = y=min_y=bottomY）。
    //
    // A1a（260911-05）：**空气不写**——对空气格写 air 是语义 no-op（palette 内容与 nonEmptyBlockCount
    // 均不变），但原实现每 chunk 仍做 98,304 次 setBlockState（含空气）。前提 = 被跳过的格子当前即空气；
    // 接管点 = populateNoise HEAD cancel（NoiseChunkGeneratorMixin:273-283）⇒ 目标 chunk 全新，
    // 前提应恒成立，并由 WBCHECK 自检门控逐格实证（见下）。
    private static final boolean WBCHECK = System.getProperty("coreswap.wbcheck") != null;
    /** A1a 的 A/B 开关（{@code -Dcoreswap.skipair=0} = 旧形态「逐格写含空气」，默认开 = 跳过空气）。
     *  静态 final ⇒ JIT 常量折叠，热路径零分支成本；用途 = 同构建态单变量 A/B（R3 验证三件套之一）。
     *  260912-01 D3：默认值收进分版缝 {@code WgCompat.SKIPAIR_ON}（1.20.1=true 已验证 / 1.21.6=false 待翻转），
     *  property 解析统一走 {@code WgCompat.flag}（未设 = 分版缺省；显式 0/1 两版均可覆盖）。 */
    private static final boolean SKIPAIR = WgCompat.flag("coreswap.skipair", WgCompat.SKIPAIR_ON);
    private static final java.util.concurrent.atomic.AtomicLong WB_AIR_SKIP =
            new java.util.concurrent.atomic.AtomicLong();
    private static final java.util.concurrent.atomic.AtomicLong WB_STALE_NONAIR =
            new java.util.concurrent.atomic.AtomicLong();
    /** C 线 Tier 4：旧逐块路径的**对称**计时（仅 {@code -Dcoreswap.bulkwblog=1} 臂）——
     *  使两臂能在**同一 run / 同一仪器**下比较写回分项（bulk 侧见 BulkWb 的 ns/call(total)）。 */
    private static final boolean WBLOG = System.getProperty("coreswap.bulkwblog") != null;
    private static final java.util.concurrent.atomic.AtomicLong PB_CALLS =
            new java.util.concurrent.atomic.AtomicLong();
    private static final java.util.concurrent.atomic.AtomicLong PB_NS =
            new java.util.concurrent.atomic.AtomicLong();

    /**
     * C 线（260911-05，用户批准 D1）bulk 写回默认开关（260912-01 D3：收进分版缝）。
     * 1.20.1 缺省 true（已验证）/ 1.21.6 缺省 false（D-4(i)：行为不变，待自身 A/B 后单独翻转）。
     * 未设 property 取分版缺省；{@code -Dcoreswap.bulkwb=1}/{@code =0} 两版均可显式覆盖
     * （1.21.6 上 {@code -Dcoreswap.bulkwb=1} 仍可**强制开启**，用于 1.21.6 侧自身 A/B）。
     * <p>注：{@code BulkWb} 内部诊断/自证分支（LOG/WBTEST 等）仍由该文件自身 `ON` 门控
     * （`BulkWb.java` 同批共享化、由另一 worker 同步为同一 WgCompat 表达式）；本类只承担**分派判定**。
     */
    private static final boolean BULKWB = WgCompat.flag("coreswap.bulkwb", WgCompat.BULKWB_ON);

    /**
     * C 线（260911-05，用户批准 D1）分派点：**bulk 整段替换**（默认）↔ 旧逐块路径
     * （{@code -Dcoreswap.bulkwb=0}）。高度图与 Tier 1 指纹门在两形态**共用**，
     * 保证 A/B 唯一变量 = 写回形态（口径对等纪律，见 index.yaml 260910-05 条教训）。
     * 260912-01 D3：分派条件由 {@code BulkWb.ON} 改为本类 {@code BULKWB}（分版缺省经 WgCompat），
     * 1.20.1 语义不变（同为「property 非 "0" 即真 + 缺省 true」）。
     */
    private static void writeChunk(Chunk chunk, int cx, int cz, int[] buf, int height) {
        if (BULKWB) {
            BulkWb.writeSections(chunk, cx, cz, buf, height);
        } else {
            writeChunkPerBlock(chunk, cx, cz, buf, height);
        }
        // 补设高度图（原版 populateNoise 只设 WORLD_SURFACE_WG；buildSurface 被跳过，
        // 需一次性补齐全部，否则 FULL 后的生物生成/寻路/光照依赖错乱）
        long th0 = ChunkTiming.ON ? System.nanoTime() : 0L;   // B2 并入（1.21.6 :540）
        Heightmap.populateHeightmaps(chunk, java.util.Set.of(
                Heightmap.Type.WORLD_SURFACE_WG,
                Heightmap.Type.WORLD_SURFACE,
                Heightmap.Type.OCEAN_FLOOR_WG,
                Heightmap.Type.OCEAN_FLOOR,
                Heightmap.Type.MOTION_BLOCKING,
                Heightmap.Type.MOTION_BLOCKING_NO_LEAVES));
        ChunkTiming.addHmap(System.nanoTime() - th0);   // B2 并入（1.21.6 :548）
        // Tier 1（默认关，-Dcoreswap.wbcontent=1）：写回**之后**的逐 chunk 读回指纹门。
        // 与 [WG-CONTENT]（Rust buf 层）不同层——后者对 Java 侧消费的变更不敏感。
        BulkWb.contentLine(chunk, cx, cz);
    }

    /** 旧逐块路径（A1a 跳空气 + WBCHECK 自检）；保留为 A/B 回退与降级路径。 */
    private static void writeChunkPerBlock(Chunk chunk, int cx, int cz, int[] buf, int height) {
        long tPb0 = WBLOG ? System.nanoTime() : 0L;
        int secCount = height / 16;
        net.minecraft.world.chunk.ChunkSection[] sections = new net.minecraft.world.chunk.ChunkSection[secCount];
        for (int secIdx = 0; secIdx < secCount; secIdx++) sections[secIdx] = chunk.getSection(secIdx);
        for (int by = 0; by < height; by++) {
            net.minecraft.world.chunk.ChunkSection sec = sections[by >> 4];
            int sy = by & 15;
            for (int z = 0; z < 16; z++) {
                int base = by * 256 + z * 16;
                for (int x = 0; x < 16; x++) {
                    int id = buf[base + x];
                    if (id < 0 || id >= MAX_ID)
                        throw new IllegalArgumentException("bad id " + id + " chunk(" + cx + "," + cz + ")");
                    if (id == 0) {   // A1a：raw id 0 = minecraft:air（blocks.json 实证）
                        if (WBCHECK) {
                            WB_AIR_SKIP.incrementAndGet();
                            // judge C5 严格化：跳过的写是「写 minecraft:air 默认态」，故前提必须是
                            // 「当前格恰为 Blocks.AIR」而非泛 isAir()——持 cave_air/void_air 的格子旧形态
                            // 会被改写成 air、新形态不会 ⇒ 用 isAir() 会漏计该类违例（vanilla 参照
                            // Heightmap.java:53 亦用 isOf(Blocks.AIR)）。
                            if (!sec.getBlockState(x, sy, z).isOf(Blocks.AIR)) WB_STALE_NONAIR.incrementAndGet();
                        }
                        if (SKIPAIR) continue;
                    }
                    BlockState st = stateById(id);
                    // 必须用 ChunkSection.setBlockState（内部=container.set + nonEmptyBlockCount 更新）：
                    // 直写 container.set 不更新计数 → isEmpty() 误判 true → 全部读成空气（历史根因）
                    sec.setBlockState(x, sy, z, st);
                }
            }
        }
        if (WBLOG) {
            PB_CALLS.incrementAndGet();
            PB_NS.addAndGet(System.nanoTime() - tPb0);
        }
    }

    /**
     * raw block id → {@link BlockState}（M14 根因修复的唯一实现处；C 线 bulk 路径复用同一份缓存与语义）。
     *
     * <p>M14 根因（2026-09-01）：Rust buf 携带的是 blocks.json 域的 **block 注册表 raw id**
     * （"minecraft:stone": 1），不是全局 **block state id**。正确映射 = {@code Registries.BLOCK.getRawId}
     * → {@code block.getDefaultState()}。6a7337d 改用 {@code Block.STATE_IDS}（state id 域）是反向修复——
     * 只修对了分析/显示层的对照，把写入路径也改错：nether 块 raw id 在 STATE_IDS 域错位解码成
     * oak_leaves×3150（实机「怪异城」= 此处写入的错块）。
     *
     * <p>260912-01 D3 并入注记（scout §2.2 B5）：1.21.6 侧为 `writeChunk` 内联形态（1.21.6 :519-531），
     * 语义与本方法逐字相同（M14 说明同源）；共享形态取本独立方法——`BulkWb` 复用本方法，
     * 依赖其**包私有 `static`** 可见性（两版同包 `wg.bench`）。
     */
    static BlockState stateById(int id) {
        BlockState st = STATE_BY_ID.get(id);
        if (st == null) {
            net.minecraft.block.Block b = net.minecraft.registry.Registries.BLOCK.get(id);
            st = b == null ? AIR : b.getDefaultState();
            if (st == null) st = AIR;
            STATE_BY_ID.set(id, st);  // 幂等 set（并发同 id 同值，无锁安全）
        }
        return st;
    }

    public static void destroy() {
        // 只标记禁用并摘除句柄；真实释放由 shutdown hook 完成
        // （防止「保存并退出」时异步 chunk 生成还在 fillBlocks 里用已释放句柄 → use-after-free）
        // A1a 等价性自检汇总（仅 WBCHECK 臂）：stale_nonair 必须为 0，否则「空气不写」前提被证伪
        if (WBCHECK) {
            System.out.println("[WB-CHECK] air_skipped=" + WB_AIR_SKIP.get()
                    + " stale_nonair=" + WB_STALE_NONAIR.get()
                    + " (stale_nonair=0 ⇒ 跳过空气写与逐格写等价)");
        }
        // C 线 bulk 写回汇总（仅 -Dcoreswap.bulkwblog=1 臂）：section 数 / 跳空气 / ID_LIST 命中 / 分项 ns
        BulkWb.reportSummary();
        // C 线 Tier 4：旧逐块路径对称计时（同 run 可比）
        if (WBLOG) {
            long pc = PB_CALLS.get();
            System.out.println("[WG-PERBLOCK] calls=" + pc + " ns/call(total)="
                    + (pc > 0 ? PB_NS.get() / pc : -1) + " (旧逐块路径写回分项，与 [WG-BULKWB] ns/call(total) 同仪器)");
        }
        enabled = false;
        handle = 0;
        netherEnabled = false;
        netherHandle = 0;
        endEnabled = false;
        endHandle = 0;
    }

    // 分量对照探针：用 vanilla NoiseConfig 的 density function registry 采样指定坐标的分量
    private static volatile boolean compProbed = false;
    public static boolean didCompProbe() { return compProbed; }

    public static void compProbe(NoiseConfig noiseConfig) {
        compProbed = true;
        try {
            int bx = Integer.parseInt(System.getProperty("comp.x"));
            int bz = Integer.parseInt(System.getProperty("comp.z"));
            int by = Integer.parseInt(System.getProperty("comp.y", "31"));
            var router = noiseConfig.getNoiseRouter();
            var names = new String[]{"finalDensity", "depth",
                    "continents", "erosion", "ridges", "initialDensityWithoutJaggedness",
                    "fluidLevelFloodedness", "fluidLevelSpread", "barrier", "lava"};
            var noisePos = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(bx, by, bz);
            for (String n : names) {
                java.lang.reflect.Method m = router.getClass().getMethod(n);
                net.minecraft.world.gen.densityfunction.DensityFunction df =
                        (net.minecraft.world.gen.densityfunction.DensityFunction) m.invoke(router);
                if (df != null) {
                    System.out.println("[COMP] " + n + "(" + bx + "," + by + "," + bz + ")=" + df.sample(noisePos));
                } else {
                    System.out.println("[COMP] " + n + "=<null>");
                }
            }
            // 提取 finalDensity 树里的 InterpolatedNoiseSampler（base_3d_noise 唯一节点）
            final net.minecraft.world.gen.densityfunction.DensityFunction finalDensity = router.finalDensity();
            final Object[] found = new Object[1];
            finalDensity.apply(new net.minecraft.world.gen.densityfunction.DensityFunction.DensityFunctionVisitor() {
                public net.minecraft.world.gen.densityfunction.DensityFunction apply(
                        net.minecraft.world.gen.densityfunction.DensityFunction df) {
                    if (found[0] == null &&
                            df instanceof net.minecraft.util.math.noise.InterpolatedNoiseSampler) {
                        found[0] = df;
                    }
                    return df;
                }
                public net.minecraft.world.gen.densityfunction.DensityFunction.Noise apply(
                        net.minecraft.world.gen.densityfunction.DensityFunction.Noise noise) { return noise; }
            });
            if (found[0] != null) {
                net.minecraft.world.gen.densityfunction.DensityFunction df =
                        (net.minecraft.world.gen.densityfunction.DensityFunction) found[0];
                System.out.println("[COMP] base_3d_noise(" + bx + "," + by + "," + bz + ")=" + df.sample(noisePos));
            } else {
                System.out.println("[COMP] base_3d_noise=<not found>");
            }
        } catch (Throwable t) {
            System.out.println("[COMP] probe error: " + t);
        }
    }

    static {
        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            long h = handle;
            handle = 0;
            long hn = netherHandle;
            netherHandle = 0;
            long he = endHandle;
            endHandle = 0;
            if (h != 0) CppWorldgen.destroy(h);
            if (hn != 0) CppWorldgen.destroy(hn);
            if (he != 0) CppWorldgen.destroy(he);
        }, "coreswap-destroy"));
    }
}
