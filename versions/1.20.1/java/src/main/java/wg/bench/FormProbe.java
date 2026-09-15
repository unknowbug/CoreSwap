package wg.bench;

import java.util.Comparator;
import java.util.Locale;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicLong;

import net.minecraft.server.world.ChunkTicketManager;
import net.minecraft.server.world.ChunkTicketType;
import net.minecraft.server.world.ServerChunkManager;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.world.World;
import wg.bench.mixin.ServerChunkManagerAccessor;

/**
 * 形态审计预验证探针（260915-03，诊断专用，默认全关 = 零行为影响）。
 *
 * <p>开关（gradle -P 映射见 build.gradle）：
 * <ul>
 *   <li>{@code -Dcoreswap.formprobe=1} — 事件行日志（fill sub/sta/end、light call/fall、SUM）</li>
 *   <li>{@code -Dcoreswap.formdrive=1} — 虚拟玩家票据驱动（隐含要求 formprobe 同开）</li>
 * </ul>
 *
 * <p>自证行（#118/#141 硬门禁）：开启后首行打 {@code [FP-ON]}（含 commonPool parallelism /
 * availableProcessors / drive 状态）；驱动臂每个位置跳转打 {@code [FP-DRV] ev=move}。
 *
 * <p>产物行格式（离线解析器按字段读，字段名勿改）：
 * <pre>
 * [FP-FILL] ev=sub t=... cx= cz= th=...        （提交到 fill 执行器）
 * [FP-FILL] ev=sta t=... cx= cz= th=...        （开始执行，CS 接管臂独有）
 * [FP-FILL] ev=end t=... cx= cz= th=... durMs=... [callerRuns=1]
 * [FP-LIGHT] ev=call t=... cx= cz= th=... durMs=... gapMs=...
 * [FP-LIGHT] ev=fall t=... reason=...
 * [FP-DRV] ev=move seq=... cx=... cz=... t=...
 * [FP-SUM] fillN=... fillBusyMs=... lightN=... lightBusyMs=... wallMs=... poolWidth=... commonPoolParallelism=...
 * </pre>
 *
 * <p>判据预登记：`.investigations/form-audit-260915-03/probe-criteria.md`（先于采集定稿，禁改判据口径）。
 */
public final class FormProbe {
    public static final boolean ON = System.getProperty("coreswap.formprobe") != null;
    public static final boolean DRIVE = ON && System.getProperty("coreswap.formdrive") != null;

    private static final long T0 = System.nanoTime();
    private static final AtomicLong FILL_N = new AtomicLong();
    private static final AtomicLong FILL_BUSY_NS = new AtomicLong();
    private static final AtomicLong LIGHT_N = new AtomicLong();
    private static final AtomicLong LIGHT_BUSY_NS = new AtomicLong();
    private static final AtomicLong CALLERRUNS_N = new AtomicLong();
    /** 同线程上一次 light 调用结束时刻（ns），用于 gapMs（.b2 粘线判据）。 */
    private static final ConcurrentHashMap<String, Long> LIGHT_LAST_NS = new ConcurrentHashMap<>();

    static {
        if (ON) {
            System.out.println("[FP-ON] formprobe=1 drive=" + DRIVE
                    + " commonPoolParallelism=" + java.util.concurrent.ForkJoinPool.commonPool().getParallelism()
                    + " availableProcessors=" + Runtime.getRuntime().availableProcessors());
            Runtime.getRuntime().addShutdownHook(new Thread(FormProbe::printSummary, "formprobe-sum"));
        }
    }

    private FormProbe() {}

    private static double ms(long nsSinceT0) {
        return nsSinceT0 / 1e6;
    }

    private static long now() {
        return System.nanoTime() - T0;
    }

    public static void fillSub(int cx, int cz, String dim) {
        if (!ON) return;
        FILL_N.incrementAndGet();
        System.out.println(String.format(Locale.ROOT,
                "[FP-FILL] ev=sub t=%.1f dim=%s cx=%d cz=%d th=%s", ms(now()), dim, cx, cz, Thread.currentThread().getName()));
    }

    public static void fillSta(int cx, int cz, String dim) {
        if (!ON) return;
        System.out.println(String.format(Locale.ROOT,
                "[FP-FILL] ev=sta t=%.1f dim=%s cx=%d cz=%d th=%s", ms(now()), dim, cx, cz, Thread.currentThread().getName()));
    }

    /** durNs = start→end busy 时长（dispatch wrapper / 原路完成回调提供）。 */
    public static void fillEnd(int cx, int cz, String dim, long durNs, boolean callerRuns) {
        if (!ON) return;
        if (durNs > 0) FILL_BUSY_NS.addAndGet(durNs);
        if (callerRuns) CALLERRUNS_N.incrementAndGet();
        System.out.println(String.format(Locale.ROOT,
                "[FP-FILL] ev=end t=%.1f dim=%s cx=%d cz=%d th=%s durMs=%.3f%s",
                ms(now()), dim, cx, cz, Thread.currentThread().getName(), ns2ms(durNs), callerRuns ? " callerRuns=1" : ""));
    }

    private static double ns2ms(long ns) {
        return ns / 1e6;
    }

    /** light 接管成功调用（不含 vanilla 回退路径——回退走 lightFall）。 */
    public static void lightCall(int cx, int cz, long durNs) {
        if (!ON) return;
        LIGHT_N.incrementAndGet();
        LIGHT_BUSY_NS.addAndGet(durNs);
        String th = Thread.currentThread().getName();
        long t = now();
        Long last = LIGHT_LAST_NS.put(th, t);
        double gapMs = last == null ? -1.0 : ms(t - last);
        System.out.println(String.format(Locale.ROOT,
                "[FP-LIGHT] ev=call t=%.1f cx=%d cz=%d th=%s durMs=%.3f gapMs=%.1f",
                ms(t), cx, cz, th, ns2ms(durNs), gapMs));
    }

    public static void lightFall(String reason) {
        if (!ON) return;
        System.out.println(String.format(Locale.ROOT, "[FP-LIGHT] ev=fall t=%.1f reason=%s", ms(now()), reason));
    }

    private static void printSummary() {
        System.out.println(String.format(Locale.ROOT,
                "[FP-SUM] fillN=%d fillBusyMs=%.1f lightN=%d lightBusyMs=%.1f callerRunsN=%d wallMs=%.1f",
                FILL_N.get(), ns2ms(FILL_BUSY_NS.get()), LIGHT_N.get(), ns2ms(LIGHT_BUSY_NS.get()),
                CALLERRUNS_N.get(), ms(now())));
    }

    // ---------------- 虚拟玩家票据驱动 ----------------

    /** 自建票据类型（一手源：ChunkTicketType.create 公有静态，ChunkTicketType.java:40）。 */
    public static final ChunkTicketType<ChunkPos> FORM_TYPE =
            ChunkTicketType.create("formprobe", Comparator.comparingLong(ChunkPos::toLong));

    /** 虚拟玩家加载等级 = NEARBY_PLAYER_TICKET_LEVEL 同源（entity-ticking，22）。 */
    private static final int DRIVE_LEVEL = 22;
    /** 首个窗口前的稳定延时与窗口时长（ns）：25s 起跳、每 25s 一格、6 格。 */
    private static final long SETTLE_NS = 25_000_000_000L;
    private static final long DWELL_NS = 25_000_000_000L;
    private static final int HOPS = 6;
    private static final int BASE_CX = 200;
    private static final int BASE_CZ = 200;
    private static final int HOP = 24;

    private static volatile long firstTickNanos = 0L;
    private static volatile int driveSeq = -1;
    private static ChunkPos curPos = null;
    private static boolean curActive = false;

    /** 由 ServerWorldFormProbeMixin 每 tick 调用（server 线程）。 */
    public static void serverTick(ServerWorld world) {
        if (!DRIVE) return;
        if (world.getRegistryKey() != World.OVERWORLD) return;
        long t = System.nanoTime();
        if (firstTickNanos == 0L) {
            firstTickNanos = t;
            System.out.println("[FP-DRV] ev=init t=" + ms(now()));
            return;
        }
        long elapsed = t - firstTickNanos;
        if (elapsed < SETTLE_NS) return;
        int seq = (int) ((elapsed - SETTLE_NS) / DWELL_NS);
        if (seq < 0 || seq >= HOPS) return;
        if (seq == driveSeq) return;
        moveTo(world, seq);
    }

    private static void moveTo(ServerWorld world, int seq) {
        try {
            ServerChunkManager cm = (ServerChunkManager) world.getChunkManager();
            ChunkTicketManager tm = ((ServerChunkManagerAccessor) cm).wgTicketManager();
            if (curActive && curPos != null) {
                tm.removeTicketWithLevel(FORM_TYPE, curPos, DRIVE_LEVEL, curPos);
            }
            curPos = new ChunkPos(BASE_CX + HOP * seq, BASE_CZ);
            tm.addTicketWithLevel(FORM_TYPE, curPos, DRIVE_LEVEL, curPos);
            curActive = true;
            driveSeq = seq;
            System.out.println(String.format(Locale.ROOT, "[FP-DRV] ev=move seq=%d cx=%d cz=%d t=%.1f",
                    seq, curPos.x, curPos.z, ms(now())));
        } catch (Throwable t) {
            // 不吞异常（崩溃日志铁律）：打印后原样上抛，由 MC 状态机处理
            System.out.println("[FP-DRV] move FAILED seq=" + seq + ": " + t);
            if (t instanceof RuntimeException rt) throw rt;
            if (t instanceof Error err) throw err;
            throw new RuntimeException(t);
        }
    }
}
