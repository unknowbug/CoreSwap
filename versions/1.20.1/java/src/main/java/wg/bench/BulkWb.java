package wg.bench;

import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.network.PacketByteBuf;
import net.minecraft.util.collection.PackedIntegerArray;
import net.minecraft.util.math.MathHelper;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.chunk.PalettedContainer;
import wg.bench.mixin.ChunkSectionAccessor;

/**
 * C 线（260911-05，用户批准 D1）：**bulk section 写回**——把 Rust 填好的扁平
 * {@code int[]}（y-major raw block id）一次性转成 section 级 {@link PalettedContainer}，
 * 取代「每 chunk 98,304 次 {@code ChunkSection.setBlockState}」。
 *
 * <h2>为什么这样写（全部经一手源码核对，见 .investigations/bulk-writeback-260911-05/api-probe-260911-05.md）</h2>
 * <ul>
 *   <li><b>不能用 5 参 {@code PalettedContainer} 构造器</b>：其第 3 参类型
 *       {@code PalettedContainer.DataProvider} 是**包私有 record**（{@code PalettedContainer.java:398}），
 *       包外无法命名 ⇒ javac 拒。改用**公开的 {@code readPacket(PacketByteBuf)}**（{@code :203-215}），
 *       我们自己构造 buffer（storage longs 整段打包），反而**连 per-position 容器写都省掉**。</li>
 *   <li><b>buffer 契约</b>（{@code readPacket} 逐字节）：{@code byte 请求位宽} →
 *       palette 段（0：{@code VarInt rawStateId}；1..8：{@code VarInt size} + {@code size×VarInt rawStateId}；
 *       ≥9：**无字节**）→ {@code writeLongArray(storage.getData())}（带 VarInt 长度前缀）。</li>
 *   <li><b>storage 值域/位宽</b>（{@code BLOCK_STATE.createDataProvider} 的 switch，{@code :420-430}）：
 *       0→无；1..4→**恒 4 位**、**局部索引**；5..8→请求位宽、**局部索引**；
 *       ≥9→{@code ceilLog2(Block.STATE_IDS.size())}、**全局 state id**。Java 必须自复刻这 6 行 switch
 *       （provider 类型不可命名）。</li>
 *   <li><b>位布局不自实现</b>：{@code PackedIntegerArray}（ctor/{@code set}/{@code getData} 全 public）
 *       自己打包；其线性索引 = {@code (y<<8)|(z<<4)|x}，与 buf 的 section 内布局**逐位相同**。</li>
 *   <li><b>零布局转换</b>：buf 是 y-major ⇒ section s 恰是连续切片 {@code buf[s*4096, (s+1)*4096)}，
 *       切片内部布局 = section 内部布局。</li>
 *   <li><b>section 数组</b>：{@code Chunk.getSectionArray()} 返回**活引用**（{@code Chunk.java:163-165}），
 *       且 {@code ProtoChunk}/{@code WorldChunk} **均未覆写**它（只把数组交给 {@code super}）⇒
 *       生成期 ProtoChunk 同样有效。本实现进一步**原地改旧 section 对象**（不新建、不换数组元素）。</li>
 * </ul>
 *
 * <h2>派生计数：为什么必须绕过 {@code calculateCounts()}（本块实测发现）</h2>
 * {@code ChunkSection(pc, biome)} 构造器会调 {@code calculateCounts()}（{@code :27-31}），
 * 而它与增量 {@code setBlockState}（{@code :60-93}）**语义不一致**：
 * <ul>
 *   <li>{@code calculateCounts} 把「非空气且流体非空」的方块**重复**计入 {@code nonEmptyBlockCount}；</li>
 *   <li>{@code calculateCounts} 的 {@code nonEmptyFluidCount} 只计「流体自身 {@code hasRandomTicks()}」的格
 *       （静水 {@code WaterFluid.Still.hasRandomTicks()=false} ⇒ 静水 section 该计数 = 0，而增量路径 = 4096）。</li>
 * </ul>
 * vanilla {@code populateNoise} 走增量路径 ⇒ **增量语义才是生成期 vanilla 语义**（也是老路径语义）。
 * 故本实现复刻增量语义（去重遍顺带算，O(distinct)），并经 {@link ChunkSectionAccessor} 直接写三个私有计数；
 * 副作用 = 省掉 {@code calculateCounts} 的 4096 次哈希表操作（实测 72.5µs/section → 237ns/section）。
 * <p>注：三个计数**不入存档**（{@code ChunkSerializer:310-311} 只写 {@code block_states}/{@code biomes}），
 * 故 region 对拍（Tier 3）看不到它们；唯一可见面 = 生成期到下次存读之间的内存态
 * （{@code hasRandomFluidTicks()} → 流体随机刻调度；{@code nonEmptyBlockCount} → {@code isEmpty()}/客户端包）。
 *
 * <h2>开关与诊断（默认关，生产零成本）</h2>
 * <ul>
 *   <li>{@code -Dcoreswap.bulkwb=0}：回退 {@code CppBridge.writeChunkPerBlock} 旧路径（同构建态单变量 A/B + 即时降级）。</li>
 *   <li>{@code -Dcoreswap.bulkwblog=1}：首跑打印 state id 域位宽；destroy 汇总（分项 ns + 旧路径对称计时）。</li>
 *   <li>{@code -Dcoreswap.wbcontent=1}：**Tier 1/2 写回后逐 chunk 指纹门**（{@code [WG-CONTENT-WB]}）——
 *       {@code hash=} 方块状态层、{@code dh=} 派生字段层。⚠️ 与 {@code [WG-CONTENT]}（Rust **buf** 层指纹）
 *       **不是同一层**：后者对「Java 侧消费」的变更不敏感（buf 不变则 hash 必相同）。</li>
 *   <li>{@code -Dcoreswap.bulkwbtest=1}：首跑**自检**——用合成 buf 覆盖 SINGULAR/ARRAY/BI_MAP/**ID_LIST** 四支
 *       （自然生成 max_distinct=7 ⇒ ID_LIST 支实测不可达，必须刻意压测，否则「全绿但未覆盖」）。</li>
 * </ul>
 */
public final class BulkWb {

    /** 默认开；{@code -Dcoreswap.bulkwb=0} 回退旧逐块路径（A/B 单变量开关）。 */
    static final boolean ON = !"0".equals(System.getProperty("coreswap.bulkwb"));
    private static final boolean LOG = System.getProperty("coreswap.bulkwblog") != null;
    /** Tier 1/2 写回后指纹门（重载，仅验证运行开启）。 */
    private static final boolean WBCONTENT = System.getProperty("coreswap.wbcontent") != null;
    /** 首跑合成自检（覆盖 ID_LIST 支）。 */
    private static final boolean WBTEST = System.getProperty("coreswap.bulkwbtest") != null;

    /** 与 CppBridge 同域：Rust buf 携带的是 block 注册表 raw id，域上界 4096。 */
    private static final int MAX_ID = 4096;
    /** 单 section 体积（16³）。 */
    private static final int SEC_SIZE = 4096;

    private BulkWb() {}

    // ---- 诊断计数器（仅 LOG/WBCONTENT 臂有意义，默认路径只做一次静态 final 判断） ----
    private static final java.util.concurrent.atomic.AtomicLong N_SECTIONS =
            new java.util.concurrent.atomic.AtomicLong();
    /** writeSections 调用次数（T_TOTAL_NS 的正确分母——按 section 除会把每调用总时长算小）。 */
    private static final java.util.concurrent.atomic.AtomicLong N_CALLS =
            new java.util.concurrent.atomic.AtomicLong();
    private static final java.util.concurrent.atomic.AtomicLong N_AIR_SKIP =
            new java.util.concurrent.atomic.AtomicLong();
    private static final java.util.concurrent.atomic.AtomicLong N_IDLIST =
            new java.util.concurrent.atomic.AtomicLong();
    private static final java.util.concurrent.atomic.AtomicLong T_PACK_NS =
            new java.util.concurrent.atomic.AtomicLong();
    private static final java.util.concurrent.atomic.AtomicLong T_READ_NS =
            new java.util.concurrent.atomic.AtomicLong();
    private static final java.util.concurrent.atomic.AtomicLong T_COUNTS_NS =
            new java.util.concurrent.atomic.AtomicLong();
    private static final java.util.concurrent.atomic.AtomicLong T_TOTAL_NS =
            new java.util.concurrent.atomic.AtomicLong();
    private static final java.util.concurrent.atomic.AtomicBoolean REPORTED =
            new java.util.concurrent.atomic.AtomicBoolean();
    private static final java.util.concurrent.atomic.AtomicBoolean TESTED =
            new java.util.concurrent.atomic.AtomicBoolean();
    private static final java.util.concurrent.atomic.AtomicInteger MAX_DISTINCT =
            new java.util.concurrent.atomic.AtomicInteger();

    /** per-thread 复用状态（避免每 section 分配；scratch/stamp 用 epoch 免清零）。 */
    private static final class TL {
        final int[] scratch = new int[MAX_ID];      // raw block id -> 本 section 的局部索引
        final int[] stamp = new int[MAX_ID];        // == epoch 表示 scratch 项本 section 有效
        final int[] localIdx = new int[SEC_SIZE];   // 每格局部索引
        final int[] cnt = new int[SEC_SIZE];        // 每个局部索引的格数（计数复刻用，用后复位）
        final int[] palRaw = new int[SEC_SIZE];     // 局部索引 -> raw block id
        final int[] palStateIds = new int[SEC_SIZE];// 局部索引 -> 全局 block state id
        final int[] out = new int[4];               // {distinct, nec, rtc, nfc}
        final ByteBuf bb = Unpooled.buffer(8192);
        int epoch = 0;
    }

    private static final ThreadLocal<TL> TLS = ThreadLocal.withInitial(TL::new);

    /** bits==0（SINGULAR/EmptyPaletteStorage）时 storage 段 = 0 个 long。 */
    private static final long[] EMPTY_LONGS = new long[0];

    /** raw block id → 全局 block state id（唯一映射处 = CppBridge.stateById，M14 语义）。 */
    private static int stateRawId(int blockRawId) {
        return Block.STATE_IDS.getRawId(CppBridge.stateById(blockRawId));
    }

    /**
     * 把 buf（y-major raw block id，长度 = 16*16*height）整段写回 chunk 的 section。
     * 调用点与旧逐块路径相同（populateNoise HEAD cancel 之后、高度图之前）。
     */
    static void writeSections(Chunk chunk, int cx, int cz, int[] buf, int height) {
        if (height % 16 != 0) throw new IllegalArgumentException("height not multiple of 16: " + height);
        if (WBTEST) selfTest();
        int secCount = height / 16;
        ChunkSection[] sections = chunk.getSectionArray();
        TL tl = TLS.get();

        long tTotal0 = LOG ? System.nanoTime() : 0L;
        if (LOG) N_CALLS.incrementAndGet();

        for (int s = 0; s < secCount; s++) {
            int off = s * SEC_SIZE;
            ChunkSection old = sections[s];

            // ---- ① 全空气快速短路（O(4096) 扫描，命中即跳过打包） ----
            // 切片全 0（raw id 0 = minecraft:air，blocks.json 实证）且旧 section 已空
            // （nonEmptyBlockCount==0）⇒ 目标态与现状相同，无需替换。
            // 旧 section 非空则**不短路**（安全：前提被证伪时走完整替换路径）。
            boolean allAir = true;
            for (int i = 0; i < SEC_SIZE; i++) {
                if (buf[off + i] != 0) { allAir = false; break; }
            }
            if (allAir && old.isEmpty()) {
                if (LOG) N_AIR_SKIP.incrementAndGet();
                continue;
            }

            // ---- ② 构造容器（去重 + 计数复刻 + 打包 + readPacket） ----
            PalettedContainer<BlockState> pc = buildContainer(buf, off, tl, true);

            // ---- ③ 原地换容器 + 写三个计数（不新建 section、不调 calculateCounts） ----
            long tCounts0 = LOG ? System.nanoTime() : 0L;
            ChunkSectionAccessor acc = (ChunkSectionAccessor) (Object) old;
            acc.wgSetBlockStateContainer(pc);
            acc.wgSetNonEmptyBlockCount((short) tl.out[1]);
            acc.wgSetRandomTickableBlockCount((short) tl.out[2]);
            acc.wgSetNonEmptyFluidCount((short) tl.out[3]);
            // sections[s] 元素不变（原地改 = 生物群系容器原样保留、无分配）
            if (LOG) {
                T_COUNTS_NS.addAndGet(System.nanoTime() - tCounts0);
                N_SECTIONS.incrementAndGet();
                int d = tl.out[0];
                int md = MAX_DISTINCT.get();
                if (d > md) MAX_DISTINCT.compareAndSet(md, d);
            }
        }

        if (LOG) T_TOTAL_NS.addAndGet(System.nanoTime() - tTotal0);
        reportOnce();
    }

    /**
     * 由 {@code buf} 的连续切片构造 block state 容器（全流程：去重 → 计数 → 打包 → {@code readPacket}）。
     * 结果经 {@code tl.out} 回传：{@code {distinct, nonEmptyBlockCount, randomTickableBlockCount, nonEmptyFluidCount}}。
     * 抽成独立函数 = 让合成自检能直接压测 ID_LIST 支（自然生成实测不可达）。
     */
    private static PalettedContainer<BlockState> buildContainer(int[] src, int off, TL tl, boolean count) {
        long tBuild0 = LOG ? System.nanoTime() : 0L;
        // ---- ① 去重（一遍）：raw id -> 局部索引；顺带每格计数 ----
        int epoch = nextEpoch(tl);
        int distinct = 0;
        for (int i = 0; i < SEC_SIZE; i++) {
            int id = src[off + i];
            if (id < 0 || id >= MAX_ID)
                throw new IllegalArgumentException("bad id " + id + " at sec offset " + off);
            int li;
            if (tl.stamp[id] == epoch) {
                li = tl.scratch[id];
            } else {
                li = distinct;
                tl.stamp[id] = epoch;
                tl.scratch[id] = li;
                tl.palRaw[distinct] = id;
                distinct++;
            }
            tl.localIdx[i] = li;
            tl.cnt[li]++;
        }

        // ---- ② 复刻**增量**计数（ChunkSection.setBlockState :70-90 的净增量；起始态 = 全新全空气 section）----
        //   nonEmptyBlockCount      += (非空气)
        //   randomTickableBlockCount+= (非空气 ∧ hasRandomTicks)
        //   nonEmptyFluidCount      += (流体非空)   ← 与 calculateCounts 的「流体有随机刻」**不同**
        int nec = 0, rtc = 0, nfc = 0;
        for (int k = 0; k < distinct; k++) {
            int c = tl.cnt[k];
            tl.cnt[k] = 0;                                  // 复位供下个 section 复用
            BlockState st = CppBridge.stateById(tl.palRaw[k]);
            if (!st.isAir()) {
                nec += c;
                if (st.hasRandomTicks()) rtc += c;
            }
            if (!st.getFluidState().isEmpty()) nfc += c;
        }

        // ---- ③ 位宽与 storage 值域（复刻 BLOCK_STATE.createDataProvider 的 switch） ----
        int bits = (distinct <= 1) ? 0 : MathHelper.ceilLog2(distinct);
        boolean idList = bits >= 9;

        // ---- ④⑤ 打包 + 自建 buffer（契约见类注释） ----
        // ⚠️ bits==0（单态 section）时 provider = SINGULAR、storage = EmptyPaletteStorage：
        //    **不得构造 PackedIntegerArray**（elementBits=0 触发 Validate 1..32 抛 IAE，260911-05 首跑
        //    539 次异常即此因），palette 段 = 单个 VarInt state id，storage = 0 个 long。
        tl.bb.clear();
        PacketByteBuf pb = new PacketByteBuf(tl.bb);
        pb.writeByte(bits);
        if (bits == 0) {
            pb.writeVarInt(stateRawId(tl.palRaw[0]));
            pb.writeLongArray(EMPTY_LONGS);
        } else {
            // ARRAY 恒 4 位（:425）；BI_MAP 用请求位宽（:426）；ID_LIST 用全局 state id 域位宽（:427）
            int storageBits = idList ? MathHelper.ceilLog2(Block.STATE_IDS.size())
                                     : (bits <= 4 ? 4 : bits);
            PackedIntegerArray pa = new PackedIntegerArray(storageBits, SEC_SIZE);
            if (idList) {
                for (int k = 0; k < distinct; k++) tl.palStateIds[k] = stateRawId(tl.palRaw[k]);
                for (int i = 0; i < SEC_SIZE; i++) pa.set(i, tl.palStateIds[tl.localIdx[i]]);
                // ID_LIST：palette 段为空（IdListPalette.readPacket 空实现，:45-46）
            } else {
                for (int i = 0; i < SEC_SIZE; i++) pa.set(i, tl.localIdx[i]);
                pb.writeVarInt(distinct);
                for (int k = 0; k < distinct; k++) pb.writeVarInt(stateRawId(tl.palRaw[k]));
            }
            pb.writeLongArray(pa.getData());
        }

        long tRead0 = LOG ? System.nanoTime() : 0L;
        PalettedContainer<BlockState> pc = new PalettedContainer<>(
                Block.STATE_IDS, Blocks.AIR.getDefaultState(), PalettedContainer.PaletteProvider.BLOCK_STATE);
        pc.readPacket(pb);
        if (LOG) T_READ_NS.addAndGet(System.nanoTime() - tRead0);

        if (idList && count) N_IDLIST.incrementAndGet();
        tl.out[0] = distinct; tl.out[1] = nec; tl.out[2] = rtc; tl.out[3] = nfc;
        if (LOG) {
            T_PACK_NS.addAndGet(System.nanoTime() - tBuild0);   // 含 readPacket（见汇总标注）
        }
        return pc;
    }

    private static int nextEpoch(TL tl) {
        int e = tl.epoch + 1;
        if (e <= 0) {                      // 回绕：清 stamp 后从 1 重来
            java.util.Arrays.fill(tl.stamp, 0);
            e = 1;
        }
        tl.epoch = e;
        return e;
    }

    /**
     * 合成自检（{@code -Dcoreswap.bulkwbtest=1}，首跑一次）：覆盖四支编码分支并逐位读回验证。
     * 用 raw id 1..N（1 = stone，blocks.json 域），读回须等于 {@code CppBridge.stateById(id)}。
     */
    private static void selfTest() {
        if (!TESTED.compareAndSet(false, true)) return;
        int[][] cases = {
                {1},        // SINGULAR（bits=0，EmptyPaletteStorage）
                {1, 2, 3},  // ARRAY（bits=2 → storage 4 位、局部索引）
                {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20},  // BI_MAP（bits=5）
                new int[300],  // ID_LIST（bits=9 > 8 ⇒ 全局 state id 域）
        };
        for (int i = 0; i < 300; i++) cases[3][i] = i + 1;
        TL tl = new TL();
        for (int[] ids : cases) {
            int[] src = new int[SEC_SIZE];
            for (int i = 0; i < SEC_SIZE; i++) src[i] = ids[i % ids.length];
            PalettedContainer<BlockState> pc = buildContainer(src, 0, tl, false);
            int distinct = tl.out[0];
            for (int i = 0; i < SEC_SIZE; i++) {
                int x = i & 15, z = (i >> 4) & 15, y = i >> 8;
                BlockState got = pc.get(x, y, z);
                BlockState want = CppBridge.stateById(src[i]);
                if (got != want) {
                    throw new AssertionError("[WG-BULKWB-TEST] FAIL branch distinct=" + ids.length
                            + " i=" + i + " got=" + got + " want=" + want);
                }
            }
            System.out.println("[WG-BULKWB-TEST] PASS distinct=" + ids.length + " (readback " + SEC_SIZE
                    + " positions; " + (distinct > 256 ? "ID_LIST" : distinct > 16 ? "BI_MAP"
                                       : distinct > 1 ? "ARRAY" : "SINGULAR") + " branch)");
        }
        System.out.println("[WG-BULKWB-TEST] all branches PASS (SINGULAR/ARRAY/BI_MAP/ID_LIST)");
    }

    /** 首跑一次性自证：state id 域位宽（ID_LIST 分支的实际位宽来源）。 */
    private static void reportOnce() {
        if (!LOG) return;
        if (REPORTED.compareAndSet(false, true)) {
            System.out.println("[WG-BULKWB] state_ids_size=" + Block.STATE_IDS.size()
                    + " idlist_bits=" + MathHelper.ceilLog2(Block.STATE_IDS.size())
                    + " (bulk section writeback active; ARRAY<=4bit, BI_MAP 5-8bit, ID_LIST=above)");
        }
    }

    /**
     * Tier 1：**写回后**逐 chunk 读回指纹（FNV-1a 64，over 全部 section 的 state raw id）。
     * 判据用法：A/B 两臂逐 chunk hash **全等**（零差，非「差异小」）。
     */
    static long readbackHash(Chunk chunk) {
        long h = 0xcbf29ce484222325L;
        for (ChunkSection sec : chunk.getSectionArray()) {
            if (sec == null) continue;
            for (int y = 0; y < 16; y++) {
                for (int z = 0; z < 16; z++) {
                    for (int x = 0; x < 16; x++) {
                        int sid = Block.STATE_IDS.getRawId(sec.getBlockState(x, y, z));
                        for (int b = 0; b < 4; b++) {
                            h ^= (sid >>> (b * 8)) & 0xFF;
                            h *= 0x100000001b3L;
                        }
                    }
                }
            }
        }
        return h;
    }

    static void contentLine(Chunk chunk, int cx, int cz) {
        if (!WBCONTENT) return;
        int nz = 0;
        for (ChunkSection sec : chunk.getSectionArray()) {
            if (sec == null || sec.isEmpty()) continue;
            for (int y = 0; y < 16; y++)
                for (int z = 0; z < 16; z++)
                    for (int x = 0; x < 16; x++)
                        if (!sec.getBlockState(x, y, z).isOf(Blocks.AIR)) nz++;
        }
        // Tier 2：派生字段指纹（三个计数的逐 section 序列）——与方块状态指纹**分开**报，
        // 便于判定「状态层等价但派生层不等价」这类偏差（本块曾实测到该形态：calculateCounts 路线）。
        long dh = 0xcbf29ce484222325L;
        for (ChunkSection sec : chunk.getSectionArray()) {
            if (sec == null) continue;
            ChunkSectionAccessor a = (ChunkSectionAccessor) (Object) sec;
            int[] vals = {a.wgGetNonEmptyBlockCount(), a.wgGetRandomTickableBlockCount(),
                          a.wgGetNonEmptyFluidCount()};
            for (int v : vals) {
                dh ^= v & 0xFFFF;
                dh *= 0x100000001b3L;
            }
        }
        System.out.println("[WG-CONTENT-WB] chunk(" + cx + "," + cz + ") hash="
                + Long.toHexString(readbackHash(chunk)) + " dh=" + Long.toHexString(dh) + " nz=" + nz);
    }

    /** destroy 汇总（与 CppBridge.destroy 同点调用）。 */
    static void reportSummary() {
        if (!LOG) return;
        long n = N_SECTIONS.get();
        long calls = N_CALLS.get();
        System.out.println("[WG-BULKWB] calls=" + calls
                + " sections_replaced=" + n
                + " air_sections_skipped=" + N_AIR_SKIP.get()
                + " idlist_hits=" + N_IDLIST.get()
                + " max_distinct=" + MAX_DISTINCT.get()
                + (n > 0 ? (" ns/section: build=" + (T_PACK_NS.get() / n)
                            + " (incl. readPacket=" + (T_READ_NS.get() / n) + ")"
                            + " counts_set=" + (T_COUNTS_NS.get() / n))
                            + " ns/call(total)=" + (calls > 0 ? T_TOTAL_NS.get() / calls : -1)
                          : "")
                + " (per-section ns; compare against per-block setBlockState path)");
    }
}
