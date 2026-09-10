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
                    // 非空节：局部坐标直采，ly/z/x 行主序与 blocks9 布局 (y+64)*256+z*16+x 对齐
                    for (int ly = 0; ly < 16; ly++) {
                        for (int z = 0; z < 16; z++) {
                            for (int x = 0; x < 16; x++) {
                                net.minecraft.block.BlockState st = sec.getBlockState(x, ly, z);
                                // ABI：低 24 位 = raw id，高 8 位 = vanilla 真值 luminance（state 级）
                                blocks9[k++] = Registries.BLOCK.getRawId(st.getBlock())
                                        | (st.getLuminance() << 24);
                            }
                        }
                    }
                }
            }
        }
        return k == 884736;
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
        try {
            if (!wgLightCollectBlocks(chunk, blocks9)) {
                wgLightFallback("neighbor-missing-or-height " + chunkPos);
                return;
            }
            rc = wg.CppWorldgen.lightCompute(handle, blocks9, outBlock, outSky, outFlags);
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
