package wg.bench;

/**
 * 260910-04 chunk 级分段计时（口径净化门控：-Dcoreswap.chunktime=1，默认完全关闭）。
 *
 * <p>目的：把「每 chunk 墙钟 ≈1.4s 而 CPU 仅 ~0.4-1.7 核」拆开——判断时间花在
 * mixin 接管段内部（JNI/Rust/写回）还是段外（MC chunk 管线等待）。
 *
 * <p>纪律：每 chunk 数次 {@code System.nanoTime()}（~20ns 级），**非逐点**、门控默认关
 * （对齐「测量/探针污染铁律」与 workflow-patterns #11「诊断门控鸡生蛋」——门控读取
 * 与数据初始化分离）。关闭时 {@link #ON} 为 false，调用点均为一次布尔判断。
 */
public final class ChunkTiming {
    public static final boolean ON = System.getProperty("coreswap.chunktime") != null;

    private static final java.util.concurrent.atomic.LongAdder MIXIN = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.LongAdder JNI = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.LongAdder WRITE = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.LongAdder HMAP = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.LongAdder SCAN = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.LongAdder BEARD = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.LongAdder CARVE = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.LongAdder FEAT = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.LongAdder GAP = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.AtomicLong N = new java.util.concurrent.atomic.AtomicLong();

    /** 直接证伪仪器（260910-04）：同时刻在飞的接管段数 + 历史峰值（b1「单车道 ⇒ 并发恒为 1」的直接检验）。 */
    private static final java.util.concurrent.atomic.AtomicInteger INFLIGHT = new java.util.concurrent.atomic.AtomicInteger();
    private static final java.util.concurrent.atomic.AtomicInteger MAX_INFLIGHT = new java.util.concurrent.atomic.AtomicInteger();

    public static void inflightEnter() {
        if (!ON) return;
        int cur = INFLIGHT.incrementAndGet();
        MAX_INFLIGHT.accumulateAndGet(cur, Math::max);
    }

    public static void inflightExit() {
        if (!ON) return;
        INFLIGHT.decrementAndGet();
    }

    /** 每线程上一次 mixin 退出时刻（测「同一 Worker 线程两次接管之间的间隔」）。 */
    private static final ThreadLocal<long[]> LAST = ThreadLocal.withInitial(() -> new long[]{0L});

    /** 臂无关每 chunk 间隔（在 generateFeatures HEAD 打点）：vanilla / coreswap 同尺子可比。 */
    private static final ThreadLocal<long[]> FEAT_LAST = ThreadLocal.withInitial(() -> new long[]{0L});
    private static final java.util.concurrent.atomic.LongAdder FEAT_GAP = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.AtomicLong FEAT_N = new java.util.concurrent.atomic.AtomicLong();

    /** generateFeatures 入口调用（两臂都走）：累计同线程两次调用间隔 = 该臂每 chunk 线程时间。 */
    public static void featTick(long t) {
        if (!ON) return;
        long[] l = FEAT_LAST.get();
        if (l[0] != 0L) FEAT_GAP.add(t - l[0]);
        l[0] = t;
        long fn = FEAT_N.incrementAndGet();
        // vanilla 臂无 mixin 出口 → 本处驱动报告（coreswap 臂两处都会触发，无妨）
        if (fn % REPORT_EVERY == 0) report(fn);
    }
    private static final int REPORT_EVERY = 256;

    private ChunkTiming() {}

    public static void addJni(long ns) { if (ON) JNI.add(ns); }
    public static void addWrite(long ns) { if (ON) WRITE.add(ns); }
    public static void addHmap(long ns) { if (ON) HMAP.add(ns); }
    public static void addScan(long ns) { if (ON) SCAN.add(ns); }
    public static void addBeard(long ns) { if (ON) BEARD.add(ns); }
    public static void addCarve(long ns) { if (ON) CARVE.add(ns); }
    public static void addFeat(long ns) { if (ON) FEAT.add(ns); }

    /** mixin 接管段入口：累计与上一次接管退出的间隔。 */
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
                "[CHUNKTIME] n=%d perChunk(ms): mixin=%.2f [jni=%.2f write=%.2f hmap=%.2f scan=%.2f beard=%.2f] "
                        + "gap=%.2f | stage carve=%.2f feat=%.2f | armAgnostic featInterval=%.2fms(n=%d) "
                        + "| sumMixin=%.1fs sumGap=%.1fs sumCarve=%.1fs sumFeat=%.1fs | inflight max=%d now=%d",
                n, MIXIN.sum() / 1e6 / per, JNI.sum() / 1e6 / per,
                Math.max(WRITE.sum() - HMAP.sum(), 0) / 1e6 / per, HMAP.sum() / 1e6 / per,
                SCAN.sum() / 1e6 / per, BEARD.sum() / 1e6 / per,
                GAP.sum() / 1e6 / per,
                CARVE.sum() / 1e6 / per, FEAT.sum() / 1e6 / per,
                (FEAT_N.get() > 0 ? FEAT_GAP.sum() / 1e6 / FEAT_N.get() : 0.0), FEAT_N.get(),
                MIXIN.sum() / 1e9, GAP.sum() / 1e9, CARVE.sum() / 1e9, FEAT.sum() / 1e9,
                MAX_INFLIGHT.get(), INFLIGHT.get()));
    }
}
