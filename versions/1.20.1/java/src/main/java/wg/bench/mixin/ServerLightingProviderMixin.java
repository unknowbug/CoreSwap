// 未编译验证（260917-05 P-β/P-path 探针打点追加；主会话负责编译/运行）。判据 = 
// .investigations/pbeta-260917-05/criteria-260917-05.md（同批预登记）。探针 env 门控默认关
// （-Dcoreswap.light.betaprobe），缺省路径零变化；打点全在 chunk 级，不进每格热路径。
// 历史依赖 yarn 签名（一手源 .tmp/light-yarn/，行号即该文件行号）：
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
//   Util#getMainWorkerExecutor() : ExecutorService（1.20.1，无 .named()——#113 API 形态注记）
// 本类不含任何 static nested class（mixin 包铁律）。
//
// CP-1（260916-01，.b1 域批中心化）：-Dcoreswap.light.domainbatch 开启后 light() HEAD 改为
// 「域批登记 + cancel」（默认关 = 现役 per-chunk 内联路径零变化 = 同构建态单变量 A/B 开关）。
// 域 = 3×3 网格对齐块（9 中心）；封板后任务投递 Util.getMainWorkerExecutor()，一次收集 5×5
// blocks25 + 一次 JNI lightComputeDomain 产 9 中心光照，逐 chunk 写回 + setLightOn + 摘票
//（POST 语义严格在对应 chunk 计算完成后，.b1 §3.3-2）。首版仅 blocks25 ABI（packed 批形态
// 收窄声明，.b1 §1.2c）。登记/去重/封板/宽限在 wg.bench.LightDomainBatch（#12：非 mixin 类
// 禁入 mixin 包，设计案原文「mixin 同文件包内」就此勘误）。
// 自证硬门数据面（#118）：正证据 = [LIGHT-DOMAIN] task 行 + LightDomainBatch.SEALED；
// 负证据 = LIGHT_DOMAIN 开臂下 WG_DOMAIN_INLINE（降级重放/未入域计数）。
package wg.bench.mixin;

import net.minecraft.world.chunk.ChunkStatus;
import net.minecraft.server.world.ServerLightingProvider;
import net.minecraft.server.world.ThreadedAnvilChunkStorage;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.util.math.ChunkSectionPos;
import net.minecraft.world.HeightLimitView;
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
 * 域批：-Dcoreswap.light.domainbatch 开启（CP-1 .b1；缺省关 = 现役内联路径）。
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
    // CP-1 .b1：域批形态开关（同构建态单变量 A/B；缺省关）
    @Unique private static final boolean LIGHT_DOMAIN = System.getProperty("coreswap.light.domainbatch") != null;
    @Unique private static volatile boolean LIGHT_DOMAIN_HOOK_READY = false;
    @Unique private static final Object LIGHT_DOMAIN_HOOK_LOCK = new Object();
    @Unique private static final String LIGHT_DATA = System.getProperty("coreswap.light.data",
            "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen/light_data.json");
    @Unique private static volatile long lightHandle = 0L;
    @Unique private static final Object lightInitLock = new Object();
    @Unique private static final AtomicBoolean lightInitFailed = new AtomicBoolean(false);
    @Unique private static final AtomicBoolean lightNativeDead = new AtomicBoolean(false);

    // ---- 回退打点（chunk 级计数，非热路径逐点日志） ----
    @Unique private static final AtomicInteger wgLightFallbackCount = new AtomicInteger();
    @Unique private static final AtomicInteger wgLightOkCount = new AtomicInteger();

    // ---- CP-1 .b1 自证计数（#118 负证据面）：域臂下走内联/降级重放的 chunk 数 ----
    @Unique private static final AtomicInteger WG_DOMAIN_INLINE = new AtomicInteger();
    // 域批 ABI 常量（与 jni_bridge.rs / light_compute_domain 契约一致）
    @Unique private static final int WG_BLOCKS9_LEN = 9 * 16 * 16 * 384; // 884736
    @Unique private static final int WG_BLOCKS25_LEN = 25 * 16 * 16 * 384; // 2457600
    @Unique private static final int WG_DOMAIN_SEG = 2 * 24 * 2048 + 48; // 98352
    @Unique private static final int WG_DOMAIN_OUT_LEN = 9 * WG_DOMAIN_SEG; // 885168

    // ---- 探针轮 260914-04（光照 round3 判据定线；env 门控默认关，chunk 级判断一次，不进每格热路径）----
    // -Dcoreswap.light.timing        ：收集段 + native 段 chunk 级计时，每 200 chunk 汇总一行
    // -Dcoreswap.light.paldump=N     ：前 N chunk 非空节 paletteSize/bits 直方图，N 满一次性汇总
    @Unique private static final boolean LIGHT_TIMING = System.getProperty("coreswap.light.timing") != null;
    @Unique private static final int LIGHT_PALDUMP = Integer.getInteger("coreswap.light.paldump", 0);
    @Unique private static final AtomicLong WG_T_COLLECT = new AtomicLong();
    @Unique private static final AtomicLong WG_T_NATIVE = new AtomicLong();
    @Unique private static final java.util.concurrent.atomic.AtomicInteger WG_T_N = new AtomicInteger();

    // ---- P-β / P-path 探针（260917-05，env 门控默认关，chunk 级一次，不进每格热路径）----
    // -Dcoreswap.light.betaprobe：① 每 chunk 收集完成打一行 [LIGHT-BETA]（输入快照 hash + 空节计数）
    //   ② light() HEAD 打 [LIGHT-PATH] enter；outcome 打 path=domain|legacy-rust|legacy-fallback|vanilla-init0。
    //   判别（criteria-260917-05）：run 间输入 hash 差集 ∩ snap changed 集 → β（输入通道）；
    //   输入恒定而输出变 → F3/output 侧通道。
    @Unique private static final boolean LIGHT_BETAPROBE = System.getProperty("coreswap.light.betaprobe") != null;
    @Unique private static final AtomicInteger WG_BETA_SEC = new AtomicInteger();
    @Unique private static final AtomicInteger WG_BETA_SEC_CUR = new AtomicInteger();
    @Unique private static final AtomicInteger WG_PATH_N = new AtomicInteger();

    // ---- 候选 B（260914-04）：palette 级批量展开收集路 ----
    @Unique private static final boolean LIGHT_OLD_COLLECT = System.getProperty("coreswap.light.oldcollect") != null;
    @Unique private static final ThreadLocal<io.netty.buffer.ByteBuf> WG_PBUF_TL =
            ThreadLocal.withInitial(() -> io.netty.buffer.Unpooled.buffer(32 * 1024));
    @Unique private static final ThreadLocal<int[]> WG_SEC_TL = ThreadLocal.withInitial(() -> new int[4096]);
    @Unique private static final ThreadLocal<long[]> WG_WORDS_TL = ThreadLocal.withInitial(() -> new long[1024]);

    // ---- 候选 C（260914-04）：packed ABI（writePacket 帧直传，Rust 侧解码）----
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
        if (wg.bench.FormProbe.ON) {
            wg.bench.FormProbe.lightFall("x" + n + " " + reason);
        }
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
     * CP-1 重构：实例 → 静态（provider/world 显式传参；域批任务线程复用同构逻辑）。
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
    private static boolean wgLightHeightOk(HeightLimitView world) {
        return world.getBottomSectionCoord() == -4
                && world.getTopSectionCoord() - world.getBottomSectionCoord() == 24;
    }

    /** FNV-1a over int[] 前缀（P-β 输入快照指纹；seed 链式拼接 meta/pal/sto）。 */
    @Unique
    private static int wgBetaHash(int seed, int[] a, int len) {
        int h = seed;
        for (int i = 0; i < len; i++) {
            h ^= a[i];
            h *= 0x01000193;
        }
        return h;
    }

    /** FNV-1a over long[] 前缀（packed storage 段；高低 32 位合入混合）。 */
    @Unique
    private static int wgBetaHash(int seed, long[] a, int len) {
        int h = seed;
        for (int i = 0; i < len; i++) {
            h ^= (int) a[i];
            h *= 0x01000193;
            h ^= (int) (a[i] >>> 32);
            h *= 0x01000193;
        }
        return h;
    }

    @Unique
    private static boolean wgLightCollectBlocks(ServerLightingProvider provider, HeightLimitView world,
                                                Chunk center, int[] blocks9) {
        ChunkPos cpos = center.getPos();
        if (center.getBottomY() != -64 || !wgLightHeightOk(world)) {
            wgLastFailDetail = "height bottomY=" + center.getBottomY() + " botSec=" + world.getBottomSectionCoord()
                    + " span=" + (world.getTopSectionCoord() - world.getBottomSectionCoord());
            return false; // 非 1.20.1 主世界高度参数，blocks9 布局不适用
        }
        int k = 0;
        for (int dz = 0; dz < 3; dz++) {
            for (int dx = 0; dx < 3; dx++) {
                Chunk nc = wgLightNeighbor(provider, cpos, dx, dz);
                if (nc == null) return false;
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
                        if (LIGHT_BETAPROBE) WG_BETA_SEC_CUR.incrementAndGet(); // P-β：空节瞬态读计数
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
    private static Chunk wgLightNeighbor(ServerLightingProvider provider, ChunkPos cpos, int dx, int dz) {
        ChunkProvider cp = ((ChunkLightProviderAccessor) (Object)
                ((LightingProviderAccessor) (Object) provider).wgGetBlockLightProvider()).wgGetChunkProvider();
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
    private static int[] wgLightCollectPacked(ServerLightingProvider provider, HeightLimitView world, Chunk center) {
        if (center.getBottomY() != -64 || !wgLightHeightOk(world)) {
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
                Chunk nc = wgLightNeighbor(provider, cpos, dx, dz);
                if (nc == null) return null;
                net.minecraft.world.chunk.ChunkSection[] secs = nc.getSectionArray();
                if (secs == null || secs.length < 24) {
                    wgLastFailDetail = "packed-sections dx=" + dx + " dz=" + dz;
                    return null;
                }
                for (int s = 0; s < 24; s++, si += 2) {
                    net.minecraft.world.chunk.ChunkSection sec = secs[s];
                    if (sec == null || sec.isEmpty()) {
                        if (LIGHT_BETAPROBE) WG_BETA_SEC_CUR.incrementAndGet(); // P-β：空节瞬态读计数
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

    // ==================== CP-1 .b1 域批路径 ====================

    /** 域批任务回调注册（幂等；首次域提交时调用）。 */
    @Unique
    private static void wgLightDomainEnsureHook() {
        if (LIGHT_DOMAIN_HOOK_READY) return;
        synchronized (LIGHT_DOMAIN_HOOK_LOCK) {
            if (LIGHT_DOMAIN_HOOK_READY) return;
            wg.bench.LightDomainBatch.taskFactory = st ->
                    CompletableFuture.runAsync(() -> wgLightDomainTaskRun(st),
                            net.minecraft.util.Util.getMainWorkerExecutor());
            LIGHT_DOMAIN_HOOK_READY = true;
            System.out.println("[LIGHT-DOMAIN] hook armed executor=Util.getMainWorkerExecutor graceMs="
                    + wg.bench.LightDomainBatch.GRACE_MS);
        }
    }

    /** 域键：3×3 网格对齐域（floorDiv(cx,3), floorDiv(cz,3)）。 */
    @Unique
    private static long wgLightDomainKey(ChunkPos p) {
        return ((long) Math.floorDiv(p.x, 3) << 32) | (Math.floorDiv(p.z, 3) & 0xFFFFFFFFL);
    }

    /** 提交前廉价预检（.b1 §3.3-2/边界语义）：高度参数 + 3×3 邻域全部到 FEATURES。 */
    @Unique
    private static boolean wgLightDomainReady(ServerLightingProvider provider, HeightLimitView world, ChunkPos p) {
        if (!wgLightHeightOk(world)) return false;
        for (int dz = 0; dz < 3; dz++) {
            for (int dx = 0; dx < 3; dx++) {
                if (wgLightNeighbor(provider, p, dx, dz) == null) return false;
            }
        }
        return true;
    }

    /**
     * 域批提交（实例上下文调用）。返回 false = 预检未过 → 调用方走现役内联路径
     *（vanilla 回退语义原样保留）。返回 true = 已登记并 cancel 原方法。
     */
    @Unique
    private boolean wgLightDomainSubmit(Chunk chunk, CallbackInfoReturnable<CompletableFuture<Chunk>> cir) {
        ChunkPos p = chunk.getPos();
        if (!wgLightDomainReady((ServerLightingProvider) (Object) this, this.world, p)) {
            return false; // 计数由调用方统一记（防双计）
        }
        // 260916-01 崩溃修复：blocks9 收集留在 light 调用线程（与 legacy 同线程同构，历史碰撞面
        // 不变）；域任务线程零 PalettedContainer 访问（writePacket lock 检测器防跨线程并发读）。
        int[] b9 = WG_BLOCKS9_TL.get();
        if (!wgLightCollectBlocks((ServerLightingProvider) (Object) this, this.world, chunk, b9)) {
            WG_DOMAIN_INLINE.incrementAndGet();
            return false;
        }
        int[] snap = java.util.Arrays.copyOf(b9, WG_BLOCKS9_LEN);
        wgLightDomainEnsureHook();
        Object[] ctx = { this, this.chunkStorage, this.world, this.world.getBottomSectionCoord() };
        CompletableFuture<Object> fut = wg.bench.LightDomainBatch.submit(
                p.toLong(), wgLightDomainKey(p), ctx, chunk, snap);
        @SuppressWarnings("unchecked")
        CompletableFuture<Chunk> cf = (CompletableFuture<Chunk>) (CompletableFuture<?>) fut;
        cir.setReturnValue(cf);
        return true;
    }

    /** CP-1 域批收集（已废弃，260916-01 崩溃修复移除——容器读回提交线程；方法体保留会引入
     *  未用代码，整体删除。历史实现见 git 提交。） */
    // （wgLightCollectBlocks25 / wgLightNeighborAt 已删除）

    /** 单中心写回 + POST 语义（严格在对应 chunk 计算完成之后，.b1 §3.3-2）。 */
    @Unique
    private static void wgLightDomainWriteBack(ServerLightingProvider provider, ThreadedAnvilChunkStorage tacs,
                                               int bottomSection, Chunk chunk, byte[] out, int segOff) {
        ChunkPos chunkPos = chunk.getPos();
        int flagOff = segOff + 2 * 24 * 2048;
        for (int s = 0; s < 24; s++) {
            ChunkSectionPos sp = ChunkSectionPos.from(chunkPos, bottomSection + s);
            provider.enqueueSectionData(LightType.BLOCK, sp, wgLightNibble(out, segOff + s * 2048, out[flagOff + 2 * s]));
            provider.enqueueSectionData(LightType.SKY, sp, wgLightNibble(out, segOff + 24 * 2048 + s * 2048, out[flagOff + 2 * s + 1]));
        }
        // 复刻原 light() 尾部语义（yarn L179-183 POST 阶段）：setLightOn(true) + releaseLightTicket
        chunk.setLightOn(true);
        ((ThreadedAnvilChunkStorageAccessor) tacs).wgReleaseLightTicket(chunkPos);
    }

    /** 域批任务体（Util.getMainWorkerExecutor 线程）：拼帧 → 一次 JNI → 逐中心写回/降级。
     *  零 PalettedContainer 访问（收集已在提交线程完成，260916-01 崩溃修复）。 */
    @Unique
    private static void wgLightDomainTaskRun(wg.bench.LightDomainBatch.State st) {
        long[] centers = st.futures.keySet().stream().mapToLong(Long::longValue).sorted().toArray();
        if (centers.length == 0) return;
        long t0 = System.nanoTime();
        Object[] ctx = (Object[]) st.ctx;
        ServerLightingProvider provider = (ServerLightingProvider) ctx[0];
        ThreadedAnvilChunkStorage tacs = (ThreadedAnvilChunkStorage) ctx[1];
        HeightLimitView world = (HeightLimitView) ctx[2]; // 仅降级重放路径使用（loud fail 面）
        int bottomSection = (Integer) ctx[3];
        int minX = ChunkPos.getPackedX(centers[0]) - 1;
        int minZ = ChunkPos.getPackedZ(centers[0]) - 1;
        long handle = wgLightEnsureInit();
        byte[] out = null;
        String fail = null;
        if (handle == 0L) {
            fail = "init0";
        } else {
            int[] blocks25 = new int[WG_BLOCKS25_LEN]; // 域批频度 ≈ per-chunk/9，一次性分配（.b1 §1.2c 假设②）
            boolean assembled = true;
            assemble:
            for (int k = 0; k < centers.length; k++) {
                int[] b9 = st.blocks9s.get(centers[k]);
                if (b9 == null) { // 提交必有快照；缺 = 状态机破坏，整批降级（loud）
                    fail = "missing-snapshot @" + new ChunkPos(centers[k]);
                    assembled = false;
                    break assemble;
                }
                int kx = k % 3, kz = k / 3;
                for (int dz9 = 0; dz9 < 3; dz9++) {
                    for (int dx9 = 0; dx9 < 3; dx9++) {
                        int src = (dz9 * 3 + dx9) * 98304;
                        int dst = ((kz + dz9) * 5 + (kx + dx9)) * 98304;
                        System.arraycopy(b9, src, blocks25, dst, 98304);
                    }
                }
            }
            // 未覆盖的 5×5 边缘槽（不属于任何已提交中心的窗口）保持 0——不进任何中心输出，
            // 内核导出只读各中心 3×3 子窗（窗并集 = 已提交中心快照覆盖面）
            if (assembled) {
                out = new byte[WG_DOMAIN_OUT_LEN];
                int rc = wg.CppWorldgen.lightComputeDomain(handle, blocks25, out);
                if (rc != 0) {
                    fail = "domain rc=" + rc;
                    out = null;
                }
            }
        }
        int ok = 0, degraded = 0;
        for (int k = 0; k < centers.length; k++) {
            long pos = centers[k];
            Chunk ch = (Chunk) st.chunks.get(pos);
            CompletableFuture<Object> f = st.futures.get(pos);
            if (ch == null || f == null) continue;
            if (out != null) {
                wgLightDomainWriteBack(provider, tacs, bottomSection, ch, out, k * WG_DOMAIN_SEG);
                f.complete(ch);
                ok++;
                if (wg.bench.FormProbe.ON) {
                    wg.bench.FormProbe.lightCall(ChunkPos.getPackedX(pos), ChunkPos.getPackedZ(pos),
                            (System.nanoTime() - t0) / centers.length);
                }
            } else {
                // 降级重放（.b1 §1.2c fallback；预检过但任务期失败 = 状态回退，不可达路径，loud fail）
                wg.bench.LightDomainBatch.DEGRADED.incrementAndGet();
                degraded++;
                boolean handled = wgLightLegacyTakeover(provider, tacs, world, bottomSection, ch, false);
                if (handled) {
                    f.complete(ch);
                } else {
                    f.completeExceptionally(new IllegalStateException(
                            "[LightDomain] legacy replay declined @" + new ChunkPos(pos) + " fail=" + fail));
                }
            }
        }
        System.out.println("[LIGHT-DOMAIN] task centers=" + centers.length + " ok=" + ok
                + " degraded=" + degraded + " timedOut=" + st.timedOut
                + " avgMs=" + String.format("%.3f", (System.nanoTime() - t0) / 1e6 / centers.length)
                + " " + wg.bench.LightDomainBatch.selfProof()
                + (fail != null ? " fail=" + fail : ""));
    }

    /**
     * 现役 per-chunk 内联接管路径（CP-1 前的原实现整体迁移，逻辑逐行保留；实例 → 静态化）。
     * 返回 true = 接管完成（调用方 setReturnValue completedFuture）；false = 回退 vanilla。
     */
    @Unique
    private static boolean wgLightLegacyTakeover(ServerLightingProvider provider, ThreadedAnvilChunkStorage tacs,
                                                 HeightLimitView world, int bottomSection,
                                                 Chunk chunk, boolean excludeBlocks) {
        ChunkPos chunkPos = chunk.getPos();
        // 形态审计探针（260915-03）：light 接管调用计时（独立于 LIGHT_TIMING 聚合门）
        final long fpT0 = wg.bench.FormProbe.ON ? System.nanoTime() : 0L;
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
            if (LIGHT_BETAPROBE) {
                WG_BETA_SEC_CUR.set(0);
            }
            if (!LIGHT_BLOCKABI) {
                packedLens = wgLightCollectPacked(provider, world, chunk);
            }
            long _t1;
            if (packedLens != null) {
                _t1 = LIGHT_TIMING ? System.nanoTime() : 0L;
                if (LIGHT_BETAPROBE) {
                    int h = wgBetaHash(0x811c9dc5, WG_META_TL.get(), WG_META_LEN);
                    h = wgBetaHash(h, WG_PAL_TL.get(), packedLens[0]);
                    h = wgBetaHash(h, WG_STO_TL.get(), packedLens[1]);
                    int cur = WG_BETA_SEC_CUR.getAndSet(0);
                    WG_BETA_SEC.addAndGet(cur);
                    System.out.println("[LIGHT-BETA] chunk(" + chunkPos.x + "," + chunkPos.z
                            + ") abi=packed hash=" + h + " emptySec=" + cur);
                }
                rc = wg.CppWorldgen.lightComputePacked(wgLightEnsureInit(), WG_META_TL.get(), WG_PAL_TL.get(),
                        WG_STO_TL.get(), packedLens[0], packedLens[1], outBlock, outSky, outFlags);
                if (rc != 0) {
                    // packed 解码/长度错（rc=-2）→ blocks9 ABI 同 chunk 兜底重算（正确性优先）
                    if (!wgLightCollectBlocks(provider, world, chunk, blocks9)) {
                        wgLightFallback("packed-rc" + rc + "-then-collect " + chunkPos);
                        return false;
                    }
                    rc = wg.CppWorldgen.lightCompute(wgLightEnsureInit(), blocks9, outBlock, outSky, outFlags);
                }
            } else {
                if (!wgLightCollectBlocks(provider, world, chunk, blocks9)) {
                    wgLightFallback("neighbor-missing-or-height " + chunkPos);
                    return false;
                }
                if (LIGHT_BETAPROBE) {
                    int h = wgBetaHash(0x811c9dc5, blocks9, WG_BLOCKS9_LEN);
                    int cur = WG_BETA_SEC_CUR.getAndSet(0);
                    WG_BETA_SEC.addAndGet(cur);
                    System.out.println("[LIGHT-BETA] chunk(" + chunkPos.x + "," + chunkPos.z
                            + ") abi=blocks9 hash=" + h + " emptySec=" + cur);
                }
                _t1 = LIGHT_TIMING ? System.nanoTime() : 0L;
                rc = wg.CppWorldgen.lightCompute(wgLightEnsureInit(), blocks9, outBlock, outSky, outFlags);
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
            if (rc != 0) {
                wgLightFallback("lightCompute rc=" + rc + " " + chunkPos);
                return false;
            }

            for (int s = 0; s < 24; s++) {
                ChunkSectionPos sp = ChunkSectionPos.from(chunkPos, bottomSection + s);
                provider.enqueueSectionData(LightType.BLOCK, sp, wgLightNibble(outBlock, s * 2048, outFlags[2 * s]));
                provider.enqueueSectionData(LightType.SKY, sp, wgLightNibble(outSky, s * 2048, outFlags[2 * s + 1]));
            }

            // 复刻原 light() 尾部语义（yarn L179-183 POST 阶段）：setLightOn(true) + releaseLightTicket
            chunk.setLightOn(true);
            ((ThreadedAnvilChunkStorageAccessor) tacs).wgReleaseLightTicket(chunkPos);
            wgLightOkCount.incrementAndGet();
            if (wg.bench.FormProbe.ON && fpT0 != 0L) {
                wg.bench.FormProbe.lightCall(chunkPos.x, chunkPos.z, System.nanoTime() - fpT0);
            }
            // 探针轮 260914-04：paldump 每 chunk（= 一次接管调用，含 9 邻 216 节）记账 + N 满汇总
            if (LIGHT_PALDUMP > 0) {
                wg.bench.LightPalDump.chunkDone(LIGHT_PALDUMP);
            }
            return true;
        } catch (Throwable t) {
            if (lightNativeDead.compareAndSet(false, true)) {
                System.out.println("[LightRust] lightCompute threw: " + t + " -> fallback vanilla permanently");
            }
            wgLightFallback("native-throw " + chunkPos);
            return false;
        }
    }

    @Inject(method = "light(Lnet/minecraft/world/chunk/Chunk;Z)Ljava/util/concurrent/CompletableFuture;",
            at = @At("HEAD"), cancellable = true, require = 1)
    private void wgLightRustTakeover(Chunk chunk, boolean excludeBlocks, CallbackInfoReturnable<CompletableFuture<Chunk>> cir) {
        if (!LIGHT_RUST) return; // 开关关闭：完全 vanilla，零行为影响
        if (LIGHT_BETAPROBE) {
            int n = WG_PATH_N.incrementAndGet();
            if (n == 1) System.out.println("[LIGHT-BETA] probe armed betaprobe=1");
            System.out.println("[LIGHT-PATH] enter (" + chunk.getPos().x + "," + chunk.getPos().z + ")");
        }
        long handle = wgLightEnsureInit();
        if (handle == 0L) { // init 失败（已打点一次）→ vanilla
            if (LIGHT_BETAPROBE) System.out.println("[LIGHT-PATH] path=vanilla-init0 ("
                    + chunk.getPos().x + "," + chunk.getPos().z + ")");
            return;
        }

        // CP-1 .b1：域批分流（预检未过 → 现役内联路径，vanilla 回退语义原样保留）
        if (LIGHT_DOMAIN) {
            if (wgLightDomainSubmit(chunk, cir)) {
                if (LIGHT_BETAPROBE) System.out.println("[LIGHT-PATH] path=domain ("
                        + chunk.getPos().x + "," + chunk.getPos().z + ")");
                return;
            }
            WG_DOMAIN_INLINE.incrementAndGet();
        }

        boolean handled = wgLightLegacyTakeover((ServerLightingProvider) (Object) this, this.chunkStorage,
                this.world, this.world.getBottomSectionCoord(), chunk, excludeBlocks);
        if (LIGHT_BETAPROBE) {
            System.out.println("[LIGHT-PATH] path=" + (handled ? "legacy-rust" : "legacy-fallback")
                    + " (" + chunk.getPos().x + "," + chunk.getPos().z + ")");
        }
        if (handled) {
            cir.setReturnValue(CompletableFuture.completedFuture(chunk));
        }
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
