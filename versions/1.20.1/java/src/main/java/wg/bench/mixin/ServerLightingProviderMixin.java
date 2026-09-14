// 未编译验证（260905-04 P2 Java 侧交付，主会话负责编译）
// 依赖 yarn 签名（一手源 .tmp/light-yarn/，行号即该文件行号）：
//   ServerLightingProvider#light(Chunk, boolean) : CompletableFuture<Chunk>（L171-184）
//   ServerLightingProvider#chunkStorage : private final ThreadedAnvilChunkStorage（L32）
//   ServerLightingProvider#enqueueSectionData(LightType, ChunkSectionPos, @Nullable ChunkNibbleArray)（L117，
//     override LightingProvider L121 同签名——⚠️ 第二参数是 ChunkSectionPos 不是 ChunkPos）
//   LightingProvider#propagateLight(ChunkPos)（L97/L79）
//   LightingProvider#world : protected final HeightLimitView（L14）
//   HeightLimitView#getBottomSectionCoord()/getTopSectionCoord()（yarn 标准接口，未含一手源）
//   ChunkNibbleArray#<init>(byte[])（L35，长度必须 2048）/<init>(int defaultValue)（L31）
//   ChunkProvider#getChunk(int, int, ChunkStatus, boolean)（接口标准签名，未含一手源，风险 R2）
//   ChunkStatus.LIGHT（.tmp/light-yarn/ChunkStatus.java L170）
//   Chunk#getPos()/setLightOn(boolean)/getBlockState(BlockPos)/getBottomY()（yarn 标准）
//   WorldChunk#getBlockState(BlockPos)（L182）
//   Registries.BLOCK#getRawId(Block)（yarn 标准）
// 本类不含任何 static nested class（mixin 包铁律）。
package wg.bench.mixin;

import net.minecraft.world.chunk.ChunkStatus;
import net.minecraft.server.world.ServerLightingProvider;
import net.minecraft.server.world.ThreadedAnvilChunkStorage;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.util.math.ChunkSectionPos;
import net.minecraft.world.LightType;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkNibbleArray;
import net.minecraft.world.chunk.ChunkProvider;
import net.minecraft.world.chunk.light.LightingProvider;
import net.minecraft.registry.Registries;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;

/**
 * CoreSwap 光照接管（方案 B，260905-04 P2）：在 ServerLightingProvider.light(Chunk, boolean) HEAD 接管——
 * 收集 3×3 邻域 chunk 方块 raw id → CppWorldgen.lightCompute（Rust 独立重算 3×3 只回中心双通道 nibble）
 * → enqueueSectionData 写回 → 复刻原 light() 尾部语义（chunk.setLightOn(true) + releaseLightTicket）
 * → cancel 原方法（跳过 vanilla propagateLight）。PRE/POST 外壳语义由本实现自行完成。
 * 门控：-Dcoreswap.light.rust 开启（缺省关闭 = 完全 vanilla，零行为影响 = 回退开关）。
 * 数据表：-Dcoreswap.light.data（缺省 E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen/light_data.json）。
 * 回退：Rust 失败/邻 chunk 缺失/世界高度不符 → 不 cancel，走 vanilla 路径；回退事件 chunk 级一次性
 * 打点 + 计数（前 8 次逐条 + 之后每 256 次一条，禁止逐 chunk 刷屏；诊断不在热路径逐点执行）。
 */
@Mixin(ServerLightingProvider.class)
public abstract class ServerLightingProviderMixin extends LightingProvider {

    protected ServerLightingProviderMixin(ChunkProvider chunkProvider, boolean hasBlockLight, boolean hasSkyLight) {
        super(chunkProvider, hasBlockLight, hasSkyLight);
    }

    @Shadow private ThreadedAnvilChunkStorage chunkStorage;

    // ---- feature gate / handle（静态，进程级） ----
    @Unique private static final boolean LIGHT_RUST = System.getProperty("coreswap.light.rust") != null;
    @Unique private static final String LIGHT_DATA = System.getProperty("coreswap.light.data",
            "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen/light_data.json");
    @Unique private static volatile long lightHandle = 0L;
    @Unique private static final Object lightInitLock = new Object();
    @Unique private static final AtomicBoolean lightInitFailed = new AtomicBoolean(false);
    @Unique private static final AtomicBoolean lightNativeDead = new AtomicBoolean(false);

    // ---- 回退打点（chunk 级计数，非热路径逐点日志） ----
    @Unique private static final AtomicInteger wgLightFallbackCount = new AtomicInteger();
    @Unique private static final AtomicInteger wgLightOkCount = new AtomicInteger();

    // ---- 探针轮 260914-04（光照 round3 判据定线；env 门控默认关，chunk 级判断一次，不进每格热路径）----
    // -Dcoreswap.light.timing        ：收集段 + native 段 chunk 级计时，每 200 chunk 汇总一行
    // -Dcoreswap.light.paldump=N     ：前 N chunk 非空节 paletteSize/bits 直方图，N 满一次性汇总
    @Unique private static final boolean LIGHT_TIMING = System.getProperty("coreswap.light.timing") != null;
    @Unique private static final int LIGHT_PALDUMP = Integer.getInteger("coreswap.light.paldump", 0);
    @Unique private static final AtomicLong WG_T_COLLECT = new AtomicLong();
    @Unique private static final AtomicLong WG_T_NATIVE = new AtomicLong();
    @Unique private static final java.util.concurrent.atomic.AtomicInteger WG_T_N = new AtomicInteger();

    // ---- 候选 B（260914-04）：palette 级批量展开收集路 ----
    // 快路径走 PalettedContainer.writePacket 公有序列化面（帧 = byte(bits) + palette 体 +
    // VarInt(n) + longs，一手源 PC.java:383-387 + Singular/Array/BiMap writePacket），
    // 零 mixin accessor / 零反射 / 零 remap 风险；帧自带 bits（消歧 singular/global 形态）。
    // -Dcoreswap.light.blockabi/oldcollect=1 回退旧路（A/B 开关）。
    // （对拍诊断已于 260914-04 验证后移除：4 chunk × 98304 逐元素 ALL-MATCH，#48 惯例）
    @Unique private static final boolean LIGHT_OLD_COLLECT = System.getProperty("coreswap.light.oldcollect") != null;
    @Unique private static final ThreadLocal<io.netty.buffer.ByteBuf> WG_PBUF_TL =
            ThreadLocal.withInitial(() -> io.netty.buffer.Unpooled.buffer(32 * 1024));
    @Unique private static final ThreadLocal<int[]> WG_SEC_TL = ThreadLocal.withInitial(() -> new int[4096]);
    @Unique private static final ThreadLocal<long[]> WG_WORDS_TL = ThreadLocal.withInitial(() -> new long[1024]);

    // ---- 候选 C（260914-04）：packed ABI（writePacket 帧直传，Rust 侧解码）----
    // -Dcoreswap.light.blockabi=1 强制旧 blocks9 ABI（A/B 回退）。
    // 开关矩阵（judge 收尾条件补记）：blockabi=0（默认）→ packed 优先，帧形态意外/global 节
    // （bits≥15，1.20.1 直方图实测 0 例、未证不可能，@anchor.idk 级边界）/rc≠0 → 同 chunk
    // 静默回退 blocks9 ABI（B 快路径）→ 再败 → vanilla fallback；blockabi=1 → 恒 blocks9 ABI；
    // oldcollect=1 → 收集恒旧逐格路（与 blockabi 正交，作用于 blocks9 ABI 内部分支）。
    // （packed 输出层对拍诊断已于 260914-04 验证后移除：4 chunk 三输出逐字节 ALL-MATCH。）
    @Unique private static final boolean LIGHT_BLOCKABI = System.getProperty("coreswap.light.blockabi") != null;
    @Unique private static final int WG_META_LEN = 9 * 24 * 2; // 432
    @Unique private static final ThreadLocal<int[]> WG_META_TL = ThreadLocal.withInitial(() -> new int[WG_META_LEN]);
    @Unique private static final ThreadLocal<int[]> WG_PAL_TL =
            ThreadLocal.withInitial(() -> new int[9 * 24 * 256]); // BiMap psz 上界
    @Unique private static final ThreadLocal<long[]> WG_STO_TL =
            ThreadLocal.withInitial(() -> new long[9 * 24 * 512]); // bits=8 → 512 long/节（global 不进）

    @Unique
    private static long wgLightEnsureInit() {
        long h = lightHandle;
        if (h != 0L) return h;
        if (lightInitFailed.get()) return 0L;
        synchronized (lightInitLock) {
            if (lightHandle != 0L) return lightHandle;
            try {
                h = wg.CppWorldgen.lightInit(LIGHT_DATA);
            } catch (Throwable t) {
                System.out.println("[LightRust] lightInit threw: " + t + " -> fallback vanilla");
                lightInitFailed.set(true);
                return 0L;
            }
            if (h == 0L) {
                System.out.println("[LightRust] lightInit failed (0) path=" + LIGHT_DATA + " -> fallback vanilla");
                lightInitFailed.set(true);
                return 0L;
            }
            lightHandle = h;
            System.out.println("[LightRust] lightInit ok handle=" + h);
            return h;
        }
    }

    @Unique
    private static void wgLightFallback(String reason) {
        int n = wgLightFallbackCount.incrementAndGet();
        if (n <= 8 || (n % 256) == 0) {
            System.out.println("[LightRust] fallback vanilla x" + n + " reason=" + reason
                    + " detail=[" + wgLastFailDetail + "]");
        }
    }

    /**
     * 组装 3×3 邻域 blocks9。返回 false = 邻 chunk 缺失 / 世界高度不符 → 调用方回退 vanilla。
     * chunkIdx = dz*3+dx（dx/dz 0..2 → 邻 (cx-1+dx, cz-1+dz)）；chunk 内 (y+64)*256 + z*16 + x，总长 884736。
     * 260905-04 round2：section 直采（复刻 WorldChunk#getBlockState 一手语义 L199-207：
     * sectionArray[idx].isEmpty() → AIR；否则 section.getBlockState(x&15,y&15,z&15)），
     * 免 BlockPos/世界坐标换算；空节 Arrays.fill(0)（= AIR raw id 0 + luminance 0，与 vanilla 返回值等价）。
     */
    @Unique
    private static volatile String wgLastFailDetail = "";

    // ThreadLocal 缓冲复用（light 线程池每线程一份，~7MB/线程；lambda 不产生嵌套类）
    @Unique private static final ThreadLocal<int[]> WG_BLOCKS9_TL =
            ThreadLocal.withInitial(() -> new int[884736]);
    @Unique private static final ThreadLocal<byte[]> WG_OUT_BLOCK_TL =
            ThreadLocal.withInitial(() -> new byte[24 * 2048]);
    @Unique private static final ThreadLocal<byte[]> WG_OUT_SKY_TL =
            ThreadLocal.withInitial(() -> new byte[24 * 2048]);
    @Unique private static final ThreadLocal<byte[]> WG_OUT_FLAGS_TL =
            ThreadLocal.withInitial(() -> new byte[48]);

    @Unique
    private boolean wgLightCollectBlocks(Chunk center, int[] blocks9) {
        ChunkPos cpos = center.getPos();
        if (center.getBottomY() != -64 || this.world.getBottomSectionCoord() != -4
                || this.world.getTopSectionCoord() - this.world.getBottomSectionCoord() != 24) {
            wgLastFailDetail = "height bottomY=" + center.getBottomY() + " botSec=" + this.world.getBottomSectionCoord()
                    + " span=" + (this.world.getTopSectionCoord() - this.world.getBottomSectionCoord());
            return false; // 非 1.20.1 主世界高度参数，blocks9 布局不适用
        }
        int k = 0;
        for (int dz = 0; dz < 3; dz++) {
            for (int dx = 0; dx < 3; dx++) {
                ChunkProvider cp = ((ChunkLightProviderAccessor) (Object)
                        ((LightingProviderAccessor) (Object) this).wgGetBlockLightProvider()).wgGetChunkProvider();
                // yarn 一手源 ChunkProvider.java L10：getChunk(int,int) -> LightSourceView（TACS 传入实为 Chunk）
                Chunk nc = cp == null ? null : (Chunk) cp.getChunk(cpos.x - 1 + dx, cpos.z - 1 + dz);
                // LIGHT 任务执行时邻 chunk 保证到 INITIALIZE_LIGHT（range=1）；D4 只需 blocks（FEATURES 起就绪）
                if (nc == null) {
                    wgLastFailDetail = "null dx=" + dx + " dz=" + dz;
                    return false;
                }
                if (nc.getStatus() == null || !nc.getStatus().isAtLeast(ChunkStatus.FEATURES)) {
                    wgLastFailDetail = "status dx=" + dx + " dz=" + dz + " st=" + nc.getStatus();
                    return false;
                }
                // 临时诊断（judge 条件 MUST，260905-04 round2）：前 4 chunk 双采集 hash 对拍后移除
                // （放在本 chunk 98304 项写完之后；k 已前移，对拍切片 [k-98304, k)）
                net.minecraft.world.chunk.ChunkSection[] secs = nc.getSectionArray();
                if (secs == null || secs.length < 24) {
                    wgLastFailDetail = "sections dx=" + dx + " dz=" + dz + " n=" + (secs == null ? -1 : secs.length);
                    return false;
                }
                for (int s = 0; s < 24; s++) {
                    net.minecraft.world.chunk.ChunkSection sec = secs[s];
                    if (sec == null || sec.isEmpty()) {
                        // 空节 = 全 AIR：raw id 0 + luminance 0（vanilla getBlockState 空节返回 AIR 默认态，等价）
                        java.util.Arrays.fill(blocks9, k, k + 4096, 0);
                        k += 4096;
                        continue;
                    }
                    // 探针轮 260914-04：paldump 直方图（Data 私有 record 走 LightPalDump 缓存反射桥接，
                    // dev 探针专用；chunk 级门控在 wgLightRustTakeover 的 ok 点记账）
                    if (LIGHT_PALDUMP > 0 && wg.bench.LightPalDump.ready()) {
                        wg.bench.LightPalDump.section(sec);
                    }
                    // 候选 B：快路径（-Dcoreswap.light.oldcollect=1 强制旧路；对拍诊断已于
                    // 260914-04 验证后移除——4 chunk ALL-MATCH，#48 惯例）
                    if (LIGHT_OLD_COLLECT) {
                        if (!wgLightFillSectionOld(sec, blocks9, k)) {
                            wgLastFailDetail = "old-fill " + ((dx - 1)) + "," + ((dz - 1)) + " s=" + s;
                            return false;
                        }
                    } else if (!wgLightFillSectionFast(sec, blocks9, k)
                            && !wgLightFillSectionOld(sec, blocks9, k)) {
                        wgLastFailDetail = "section-fill-both " + ((dx - 1)) + "," + ((dz - 1)) + " s=" + s;
                        return false;
                    }
                    k += 4096;
                }
            }
        }
        return k == 884736;
    }

    /** 邻 chunk 提取（collect blocks / packed 共用）：null/状态不足记 detail 返 null。 */
    @Unique
    private Chunk wgLightNeighbor(ChunkPos cpos, int dx, int dz) {
        ChunkProvider cp = ((ChunkLightProviderAccessor) (Object)
                ((LightingProviderAccessor) (Object) this).wgGetBlockLightProvider()).wgGetChunkProvider();
        // yarn 一手源 ChunkProvider.java L10：getChunk(int,int) -> LightSourceView（TACS 传入实为 Chunk）
        Chunk nc = cp == null ? null : (Chunk) cp.getChunk(cpos.x - 1 + dx, cpos.z - 1 + dz);
        // LIGHT 任务执行时邻 chunk 保证到 INITIALIZE_LIGHT（range=1）；D4 只需 blocks（FEATURES 起就绪）
        if (nc == null) {
            wgLastFailDetail = "null dx=" + dx + " dz=" + dz;
            return null;
        }
        if (nc.getStatus() == null || !nc.getStatus().isAtLeast(ChunkStatus.FEATURES)) {
            wgLastFailDetail = "status dx=" + dx + " dz=" + dz + " st=" + nc.getStatus();
            return null;
        }
        return nc;
    }

    /**
     * 候选 C packed 收集：writePacket 帧拆包 → sectionMeta/paletteData/storage 三数组
     * （Rust 侧解码）。返回 null = 本 chunk 回退 blocks9 ABI（global 节 / 帧形态意外 / 全空）。
     * 返回 int[2] = {paletteLen, storageLen}（JNI 显式长度参数，免拷贝截断）。
     */
    @Unique
    private int[] wgLightCollectPacked(Chunk center) {
        if (center.getBottomY() != -64 || this.world.getBottomSectionCoord() != -4
                || this.world.getTopSectionCoord() - this.world.getBottomSectionCoord() != 24) {
            wgLastFailDetail = "packed-height";
            return null;
        }
        ChunkPos cpos = center.getPos();
        int[] meta = WG_META_TL.get();
        int[] pal = WG_PAL_TL.get();
        long[] sto = WG_STO_TL.get();
        int palCur = 0, stoCur = 0;
        io.netty.buffer.ByteBuf bb = WG_PBUF_TL.get();
        net.minecraft.network.PacketByteBuf pbuf = new net.minecraft.network.PacketByteBuf(bb);
        int si = 0;
        for (int dz = 0; dz < 3; dz++) {
            for (int dx = 0; dx < 3; dx++) {
                Chunk nc = wgLightNeighbor(cpos, dx, dz);
                if (nc == null) return null;
                net.minecraft.world.chunk.ChunkSection[] secs = nc.getSectionArray();
                if (secs == null || secs.length < 24) {
                    wgLastFailDetail = "packed-sections dx=" + dx + " dz=" + dz;
                    return null;
                }
                for (int s = 0; s < 24; s++, si += 2) {
                    net.minecraft.world.chunk.ChunkSection sec = secs[s];
                    if (sec == null || sec.isEmpty()) {
                        meta[si] = 0;
                        meta[si + 1] = 0;
                        continue;
                    }
                    bb.clear();
                    try {
                        sec.getBlockStateContainer().writePacket(pbuf);
                    } catch (Throwable t) {
                        wgLastFailDetail = "packed-writePacket " + t;
                        return null;
                    }
                    if (pbuf.readableBytes() < 1) return null;
                    int bits = pbuf.readUnsignedByte();
                    if (bits == 0) {
                        // singular：裸 VarInt(entryId) + 空 longs（VarInt(0)）
                        int enc = wgLightEncState(pbuf.readVarInt());
                        if (enc == Integer.MIN_VALUE || pbuf.readVarInt() != 0 || palCur >= pal.length) return null;
                        meta[si] = 0;
                        meta[si + 1] = 1;
                        pal[palCur++] = enc;
                        continue;
                    }
                    if (bits < 4 || bits > 14) return null; // global（ID_LIST）/ 未知 → 整 chunk 回退
                    int psz = pbuf.readVarInt();
                    if (psz < 1 || psz > 256 || palCur + psz > pal.length) return null;
                    for (int i = 0; i < psz; i++) {
                        int enc = wgLightEncState(pbuf.readVarInt());
                        if (enc == Integer.MIN_VALUE) return null;
                        pal[palCur++] = enc;
                    }
                    int epl = 64 / bits;
                    int nLen = pbuf.readVarInt(); // writeLongArray 的 VarInt 长度前缀（B 路同款；漏读 = 全流错位）
                    int n = (4096 + epl - 1) / epl;
                    if (nLen != n || stoCur + n > sto.length) return null;
                    for (int i = 0; i < n; i++) sto[stoCur++] = pbuf.readLong();
                    meta[si] = bits;
                    meta[si + 1] = psz;
                }
            }
        }
        if (si != WG_META_LEN || palCur == 0 || stoCur == 0) return null;
        return new int[] { palCur, stoCur };
    }

    /** 旧收集路（round2 section 直采）——回退分支 + 对拍基准，逻辑与历史版逐行同构。 */
    @Unique
    private static boolean wgLightFillSectionOld(net.minecraft.world.chunk.ChunkSection sec, int[] blocks9, int k) {
        try {
            for (int ly = 0; ly < 16; ly++) {
                for (int z = 0; z < 16; z++) {
                    for (int x = 0; x < 16; x++) {
                        net.minecraft.block.BlockState st = sec.getBlockState(x, ly, z);
                        // ABI：低 24 位 = raw id，高 8 位 = vanilla 真值 luminance（state 级）
                        blocks9[k + (ly << 8 | z << 4 | x)] = Registries.BLOCK.getRawId(st.getBlock())
                                | (st.getLuminance() << 24);
                    }
                }
            }
            return true;
        } catch (Throwable t) {
            return false;
        }
    }

    /** state id（Block.STATE_IDS 域）→ blocks9 ABI 编码（rawId | lum<<24）。未知 id 返 MIN_VALUE。 */
    @Unique
    private static int wgLightEncState(int stateId) {
        net.minecraft.block.BlockState st = net.minecraft.block.Block.STATE_IDS.get(stateId);
        if (st == null) {
            return Integer.MIN_VALUE;
        }
        return Registries.BLOCK.getRawId(st.getBlock()) | (st.getLuminance() << 24);
    }

    /**
     * 候选 B 快路径：writePacket 公有帧（PC.java:383-387）→ palette 小表 + PackedIntegerArray
     * 位流解码（LSB-first、元素不跨 long，PIA.java:261/307-313；索引序 = computeIndex
     * y<<8|z<<4|x，PC.java:465-467，与 blocks9 节内序逐元素一致——b2 §1 静态闭环）。
     * 任何帧形态意外返 false（调用方回旧路），防御面：bits 域外 / palette 越界 / n 不符。
     */
    @Unique
    private static boolean wgLightFillSectionFast(net.minecraft.world.chunk.ChunkSection sec, int[] blocks9, int k) {
        io.netty.buffer.ByteBuf bb = WG_PBUF_TL.get();
        bb.clear();
        net.minecraft.network.PacketByteBuf pbuf = new net.minecraft.network.PacketByteBuf(bb);
        try {
            sec.getBlockStateContainer().writePacket(pbuf);
        } catch (Throwable t) {
            return false; // 未初始化 palette 等 → 旧路
        }
        if (pbuf.readableBytes() < 1) return false;
        int bits = pbuf.readUnsignedByte(); // Data.writePacket 首字节 = storage.getElementBits()
        if (bits == 0) {
            // singular：SingularPalette.writePacket = 裸 VarInt(entryId)（无 size 前缀）+ 空 longs（VarInt(0)）
            int enc = wgLightEncState(pbuf.readVarInt());
            if (enc == Integer.MIN_VALUE) return false;
            if (pbuf.readVarInt() != 0) return false;
            java.util.Arrays.fill(blocks9, k, k + 4096, enc);
            return true;
        }
        if (bits < 4 || bits > 16) return false; // BLOCK_STATE provider 域 = 4..8 / 15..16（防御，judge MUST）
        int[] tab = WG_SEC_TL.get();
        int psz;
        if (bits >= 15) {
            psz = -1; // global（ID_LIST）：无 palette 体，位流索引即 state id
        } else {
            psz = pbuf.readVarInt();
            if (psz < 1 || psz > 256) return false;
            for (int i = 0; i < psz; i++) {
                int enc = wgLightEncState(pbuf.readVarInt());
                if (enc == Integer.MIN_VALUE) return false;
                tab[i] = enc;
            }
        }
        int n = pbuf.readVarInt();
        int epl = 64 / bits;
        if (n != (4096 + epl - 1) / epl || n > 1024) return false; // PackedIntegerArray 容量恒等式（PIA.java:261 构造）
        long[] words = WG_WORDS_TL.get();
        for (int i = 0; i < n; i++) {
            words[i] = pbuf.readLong();
        }
        long mask = (1L << bits) - 1;
        if (psz >= 1) {
            for (int idx = 0; idx < 4096; idx++) {
                int w = idx / epl; // 常数除（JIT 乘移化），语义 = getStorageIndex（b2 §1.3 免魔数论证）
                int v = (int) (words[w] >>> ((idx - w * epl) * bits) & mask);
                if (v >= psz) return false; // 索引越界 = 帧解译错位，一票回旧路（防静默错值）
                blocks9[k + idx] = tab[v];
            }
        } else {
            for (int idx = 0; idx < 4096; idx++) {
                int w = idx / epl;
                int sid = (int) (words[w] >>> ((idx - w * epl) * bits) & mask);
                int enc = wgLightEncState(sid);
                if (enc == Integer.MIN_VALUE) return false;
                blocks9[k + idx] = enc;
            }
        }
        return true;
    }

    @Inject(method = "light(Lnet/minecraft/world/chunk/Chunk;Z)Ljava/util/concurrent/CompletableFuture;",
            at = @At("HEAD"), cancellable = true, require = 1)
    private void wgLightRustTakeover(Chunk chunk, boolean excludeBlocks, CallbackInfoReturnable<CompletableFuture<Chunk>> cir) {
        if (!LIGHT_RUST) return; // 开关关闭：完全 vanilla，零行为影响
        long handle = wgLightEnsureInit();
        if (handle == 0L) return; // init 失败（已打点一次）→ vanilla

        ChunkPos chunkPos = chunk.getPos();
        // ThreadLocal 缓冲复用（out 缓冲在 wgLightNibble 中已拷出，方法返回后无保留引用）
        int[] blocks9 = WG_BLOCKS9_TL.get();
        byte[] outBlock = WG_OUT_BLOCK_TL.get();
        byte[] outSky = WG_OUT_SKY_TL.get();
        byte[] outFlags = WG_OUT_FLAGS_TL.get();
        int rc;
        long _t0 = LIGHT_TIMING ? System.nanoTime() : 0L;
        try {
            // 候选 C 分流：packed ABI 优先（失败/global 节 → blocks9 ABI 同 chunk 重算）
            int[] packedLens = null;
            if (!LIGHT_BLOCKABI) {
                packedLens = wgLightCollectPacked(chunk);
            }
            long _t1;
            if (packedLens != null) {
                _t1 = LIGHT_TIMING ? System.nanoTime() : 0L;
                rc = wg.CppWorldgen.lightComputePacked(handle, WG_META_TL.get(), WG_PAL_TL.get(),
                        WG_STO_TL.get(), packedLens[0], packedLens[1], outBlock, outSky, outFlags);
                if (rc != 0) {
                    // packed 解码/长度错（rc=-2）→ blocks9 ABI 同 chunk 兜底重算（正确性优先）
                    if (!wgLightCollectBlocks(chunk, blocks9)) {
                        wgLightFallback("packed-rc" + rc + "-then-collect " + chunkPos);
                        return;
                    }
                    rc = wg.CppWorldgen.lightCompute(handle, blocks9, outBlock, outSky, outFlags);
                }
            } else {
                if (!wgLightCollectBlocks(chunk, blocks9)) {
                    wgLightFallback("neighbor-missing-or-height " + chunkPos);
                    return;
                }
                _t1 = LIGHT_TIMING ? System.nanoTime() : 0L;
                rc = wg.CppWorldgen.lightCompute(handle, blocks9, outBlock, outSky, outFlags);
            }
            if (LIGHT_TIMING) {
                WG_T_COLLECT.addAndGet(_t1 - _t0);
                WG_T_NATIVE.addAndGet(System.nanoTime() - _t1);
                int n = WG_T_N.incrementAndGet();
                if (n % 200 == 0) {
                    System.out.println("[LIGHTPROBE-T] chunks=" + n
                            + " collectAvgMs=" + String.format("%.3f", WG_T_COLLECT.get() / 1e6 / n)
                            + " nativeAvgMs=" + String.format("%.3f", WG_T_NATIVE.get() / 1e6 / n));
                }
            }
        } catch (Throwable t) {
            if (lightNativeDead.compareAndSet(false, true)) {
                System.out.println("[LightRust] lightCompute threw: " + t + " -> fallback vanilla permanently");
            }
            wgLightFallback("native-throw " + chunkPos);
            return;
        }
        if (rc != 0) {
            wgLightFallback("lightCompute rc=" + rc + " " + chunkPos);
            return;
        }

        int bottomSection = this.world.getBottomSectionCoord();
        for (int s = 0; s < 24; s++) {
            ChunkSectionPos sp = ChunkSectionPos.from(chunkPos, bottomSection + s);
            this.enqueueSectionData(LightType.BLOCK, sp, wgLightNibble(outBlock, s * 2048, outFlags[2 * s]));
            this.enqueueSectionData(LightType.SKY, sp, wgLightNibble(outSky, s * 2048, outFlags[2 * s + 1]));
        }

        // 复刻原 light() 尾部语义（yarn L179-183 POST 阶段）：setLightOn(true) + releaseLightTicket
        chunk.setLightOn(true);
        ((ThreadedAnvilChunkStorageAccessor) this.chunkStorage).wgReleaseLightTicket(chunkPos);
        wgLightOkCount.incrementAndGet();
        // 探针轮 260914-04：paldump 每 chunk（= 一次接管调用，含 9 邻 216 节）记账 + N 满汇总
        if (LIGHT_PALDUMP > 0) {
            wg.bench.LightPalDump.chunkDone(LIGHT_PALDUMP);
        }
        cir.setReturnValue(CompletableFuture.completedFuture(chunk));
    }

    /** flag：0=数据有效（拷贝 2048B），1=全 0，2=全 15（均质用 defaultValue 构造，不传 null）。 */
    @Unique
    private static ChunkNibbleArray wgLightNibble(byte[] data, int off, byte flag) {
        if (flag == 1) return new ChunkNibbleArray(0);
        if (flag == 2) return new ChunkNibbleArray(15);
        byte[] sec = new byte[2048];
        System.arraycopy(data, off, sec, 0, 2048);
        return new ChunkNibbleArray(sec);
    }
}
