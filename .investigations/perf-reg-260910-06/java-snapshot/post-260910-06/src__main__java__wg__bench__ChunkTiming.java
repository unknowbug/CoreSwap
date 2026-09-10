package wg.bench;

/**
 * 260910-06 chunk 级接管段计时（1.20.1 版；口径净化门控：{@code -Dcoreswap.chunktime=1}，默认完全关闭）。
 *
 * <p>用途：把「每 chunk 墙钟」拆成「mixin 接管段内部耗时」与「段间 gap」，并给出**同时刻在飞接管段数**
 * 峰值 —— 后者是「重活是否被 worldgen 单车道串行化」的**直接证据**（1.21.6 R3 的判据，
 * 见 knowledge workflow-patterns #107；本类 = 1.21.6 同款仪器的 1.20.1 精简移植）。
 *
 * <p>精简说明（与 1.21.6 版的差异，**声明**）：1.20.1 侧 CppBridge 未挂 JNI/write/hmap/scan/carve/feat
 * 分项计时钩子，故本报告只报 mixin 总段 + gap + inflight + n；1.21.6 版的分项列在此**不出现**
 * （不打印恒 0 的假分项，防「看着像测了」）。
 *
 * <p>纪律：每 chunk 数次 {@code System.nanoTime()}（~20ns 级），**非逐点**；门控默认关
 * （对齐「测量/探针污染铁律」与 #11「诊断门控鸡生蛋」——门控读取与数据初始化分离）。
 * 关闭时 {@link #ON} 为 false，调用点均为一次布尔判断。
 */
public final class ChunkTiming {
    public static final boolean ON = System.getProperty("coreswap.chunktime") != null;

    /** mixin 接管段累计（ns）与同线程两次接管之间的 gap 累计（ns）。 */
    private static final java.util.concurrent.atomic.LongAdder MIXIN = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.LongAdder GAP = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.AtomicLong N = new java.util.concurrent.atomic.AtomicLong();

    /** 直接证伪仪器：同时刻在飞的接管段数 + 历史峰值（「单车道 ⇒ 并发恒为 1」的直接检验）。 */
    private static final java.util.concurrent.atomic.AtomicInteger INFLIGHT = new java.util.concurrent.atomic.AtomicInteger();
    private static final java.util.concurrent.atomic.AtomicInteger MAX_INFLIGHT = new java.util.concurrent.atomic.AtomicInteger();

    /** 每线程上一次 mixin 退出时刻（测「同一 Worker 线程两次接管之间的间隔」）。 */
    private static final ThreadLocal<long[]> LAST = ThreadLocal.withInitial(() -> new long[]{0L});

    private static final int REPORT_EVERY = 256;

    private ChunkTiming() {}

    public static void inflightEnter() {
        if (!ON) return;
        int cur = INFLIGHT.incrementAndGet();
        MAX_INFLIGHT.accumulateAndGet(cur, Math::max);
    }

    public static void inflightExit() {
        if (!ON) return;
        INFLIGHT.decrementAndGet();
    }

    /** mixin 接管段入口：累计与上一次接管退出的间隔（同线程）。 */
    public static void enter(long tEntry) {
        if (!ON) return;
        long[] l = LAST.get();
        if (l[0] != 0L) GAP.add(tEntry - l[0]);
    }

    /** mixin 接管段出口。 */
    public static void exit(long tExit, long mixinNs) {
        if (!ON) return;
        LAST.get()[0] = tExit;
        MIXIN.add(mixinNs);
        long n = N.incrementAndGet();
        if (n % REPORT_EVERY == 0) report(n);
    }

    private static void report(long n) {
        double per = (double) n;
        System.out.println(String.format(java.util.Locale.ROOT,
                "[CHUNKTIME] n=%d perChunk(ms): mixin=%.2f gap=%.2f | sumMixin=%.1fs sumGap=%.1fs | inflight max=%d now=%d",
                n, MIXIN.sum() / 1e6 / per, GAP.sum() / 1e6 / per,
                MIXIN.sum() / 1e9, GAP.sum() / 1e9,
                MAX_INFLIGHT.get(), INFLIGHT.get()));
    }
}
