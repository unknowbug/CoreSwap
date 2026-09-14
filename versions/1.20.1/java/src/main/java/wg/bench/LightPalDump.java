// 探针轮 260914-04（光照 round3 判据定线）：非空节 palette 规模/bits 直方图采集器。
// 为什么走反射而非 mixin @Accessor：PalettedContainer.Data 是**私有内部 record**（一手源
// .tmp/scout-260905-08/mcsrc/net/minecraft/world/chunk/PalettedContainer.java:361），
// 编译期签名不可引用；而 Palette / PaletteStorage 是公有接口，反射拿到 Data 实例后正常转型调用。
//
// ⚠️ 边界声明（#55 反射重映射家族）：本类只在 dev loom（yarn 名域）下由探针 env 门控调用
// （-Dcoreswap.light.paldump=N），生产路径零引用、dev 之外的命名环境不运行——字符串字段名
// "data"/"palette"/"storage" 不参与 remap，这是有意的探针专用边界，不是可复用生产模式。
//
// 线程安全：直方图累加在 light 线程池并发（计数近似容差可接受——直方图是量级判读非精确判据）；
// chunk 完成记账与打印走原子 + CAS 单次打印。
package wg.bench;

import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.chunk.Palette;
import net.minecraft.world.chunk.PalettedContainer;
import net.minecraft.util.collection.PaletteStorage;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicBoolean;

public final class LightPalDump {

    private static final long[] SZ = new long[17];   // [0..15]=paletteSize 命中，[16]=16+（ID_LIST 域）
    private static final long[] BITS = new long[17]; // 下标=bits（0=singular/EmptyPaletteStorage，4..15+）
    private static final AtomicInteger CHUNKS = new AtomicInteger();
    private static final AtomicBoolean PRINTED = new AtomicBoolean();
    private static final long[] FAIL = new long[1];

    private static final Field DATA_F;
    private static final Method PAL_M;
    private static final Method STO_M;
    static {
        Field f = null; Method p = null; Method s = null;
        try {
            f = PalettedContainer.class.getDeclaredField("data");
            f.setAccessible(true);
            // Data record 的公有组件访问器（record 声明在私有作用域内，方法本身 public）
            for (Class<?> c : PalettedContainer.class.getDeclaredClasses()) {
                if (c.getSimpleName().equals("Data")) {
                    p = c.getMethod("palette");
                    s = c.getMethod("storage");
                    p.setAccessible(true);
                    s.setAccessible(true);
                    break;
                }
            }
        } catch (Throwable t) {
            System.out.println("[LIGHTPROBE-PAL] reflect init failed: " + t);
        }
        DATA_F = f; PAL_M = p; STO_M = s;
    }

    private LightPalDump() {}

    /** 探针是否可采（初始化成功 + 未打印过）。 */
    public static boolean ready() {
        return DATA_F != null && PAL_M != null && STO_M != null && !PRINTED.get();
    }

    /** 单个非空节采样（异常单节只计数不中断）。 */
    public static void section(ChunkSection sec) {
        try {
            PalettedContainer<?> container = sec.getBlockStateContainer();
            Object d = DATA_F.get(container);
            Palette<?> pal = (Palette<?>) PAL_M.invoke(d);
            PaletteStorage sto = (PaletteStorage) STO_M.invoke(d);
            int psz = pal.getSize();
            int bits = sto.getElementBits();
            int si = Math.min(psz, 16);
            int bi = Math.min(bits, 16);
            synchronized (SZ) { SZ[si]++; BITS[bi]++; }
        } catch (Throwable t) {
            synchronized (SZ) { FAIL[0]++; }
            if (FAIL[0] == 1) System.out.println("[LIGHTPROBE-PAL] section failed: " + t);
        }
    }

    /** 一次接管调用（9 邻 216 节）完成记账；达到 limit 时一次性汇总打印。 */
    public static void chunkDone(int limit) {
        if (PRINTED.get() || CHUNKS.incrementAndGet() < limit) return;
        if (!PRINTED.compareAndSet(false, true)) return;
        StringBuilder sz = new StringBuilder(), bt = new StringBuilder();
        long total = 0;
        synchronized (SZ) {
            for (int i = 0; i <= 16; i++) {
                if (SZ[i] > 0) sz.append(i == 16 ? "16+:" : i + ":").append(SZ[i]).append(' ');
                if (BITS[i] > 0) bt.append(i == 16 ? "16+:" : i + ":").append(BITS[i]).append(' ');
            }
            for (long c : SZ) total += c;
            System.out.println("[LIGHTPROBE-PAL] nonEmptySections=" + total + " fail=" + FAIL[0]);
            System.out.println("[LIGHTPROBE-PAL] paletteSizeHist: " + sz);
            System.out.println("[LIGHTPROBE-PAL] bitsHist: " + bt);
        }
    }
}
