package wg.bench;

import java.lang.reflect.Method;
import java.util.Comparator;
import java.util.Locale;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicLong;

import net.minecraft.server.world.ChunkHolder;
import net.minecraft.server.world.ChunkTicketManager;
import net.minecraft.server.world.ChunkTicketType;
import net.minecraft.server.world.ServerChunkManager;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.world.World;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkStatus;
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
    /** 全覆盖网格驱动（260920-05，design-260920-05 §2）：隐含 formprobe 同开；与走廊 DRIVE 互斥，grid 优先。 */
    public static final boolean GRID = ON && System.getProperty("coreswap.formdrivegrid") != null;
    /**
     * level 边界对照臂驱动（260920-06，design-260920-06 §2）：单 run 双相自对照——
     * 先 addTicketWithLevel(lvlN=34，目标 INITIALIZE_LIGHT 停 light() 之前)，再升档 lvlP=33（目标 FULL）。
     * 隐含 formprobe 同开；与 DRIVE/GRID 互斥（EDGE 优先，混开属采集台污染，由解析器 Q4/Q5 VOID）。
     */
    public static final boolean EDGE = ON && System.getProperty("coreswap.formedge") != null;

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
            System.out.println("[FP-ON] formprobe=1 drive=" + DRIVE + " grid=" + GRID + " edge=" + EDGE
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
        if (!EDGE && !DRIVE && !GRID) return;
        if (world.getRegistryKey() != World.OVERWORLD) return;
        if (EDGE) {
            edgeTick(world);
            return;
        }
        if (GRID) {
            gridTick(world);
            return;
        }
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

    // ---------------- 全覆盖网格驱动（260920-05，design-260920-05 §2） ----------------

    /** face bbox 实测 [13,-25]..[332,212]（verify_design.py 复算）；网格起点取 (14,-24) 间距 20。 */
    private static final int GRID_X0 = 14;
    private static final int GRID_Z0 = -24;
    private static final int GRID_STEP = 20;
    private static final int GRID_XN = 17;
    private static final int GRID_ZN = 13;
    /** W = 221；覆盖完整性机械验证：face 全部 chunk 到最近 waypoint Chebyshev ≤10（.tmp/k2a-260920-05/verify_design.py）。 */
    public static final int GRID_W = GRID_XN * GRID_ZN;
    /** dwell 下界 = GRACE_MS(10s, LightDomainBatch.java:21 直读) + 裕量（design §1.3，禁 <12s）。 */
    private static final long GRID_SETTLE_NS = 25_000_000_000L;
    private static final long GRID_DWELL_NS = 15_000_000_000L;
    /**
     * D-Lag 变体（260920-05，design §2.1/§6 R1）：hold-all 实测 OOM（fullcov01，seq≈168/221 崩），
     * 改摘票滞留 L=2 waypoint（≈45s+2×GRACE 余量）。判读须带「含卸载竞态混杂」标注。
     */
    private static final int GRID_LAG = 2;
    private static final java.util.ArrayDeque<ChunkPos> GRID_LIVE = new java.util.ArrayDeque<>();

    private static volatile boolean gridInited = false;

    private static void gridTick(ServerWorld world) {
        long t = System.nanoTime();
        if (!gridInited) {
            gridInited = true;
            firstTickNanos = t;
            System.out.println("[FP-DRV] ev=init grid=1 W=" + GRID_W + " dwell_s=15 lag=" + GRID_LAG + " t=" + ms(now()));
            return;
        }
        long elapsed = t - firstTickNanos;
        if (elapsed < GRID_SETTLE_NS) return;
        int seq = (int) ((elapsed - GRID_SETTLE_NS) / GRID_DWELL_NS);
        if (seq < 0 || seq >= GRID_W) return;
        if (seq == driveSeq) return;
        gridMoveTo(world, seq);
    }

    private static void gridMoveTo(ServerWorld world, int seq) {
        try {
            ServerChunkManager cm = (ServerChunkManager) world.getChunkManager();
            ChunkTicketManager tm = ((ServerChunkManagerAccessor) cm).wgTicketManager();
            // 蛇形 row-major：z 行内 x 正反交替（生成请求局部性；hold-all 下不影响最终集）
            int row = seq / GRID_XN;
            int col = seq % GRID_XN;
            if (row % 2 == 1) col = GRID_XN - 1 - col;
            int cx = GRID_X0 + GRID_STEP * col;
            int cz = GRID_Z0 + GRID_STEP * row;
            ChunkPos pos = new ChunkPos(cx, cz);
            // D-Lag：滞留 L=2 后摘最旧票（design §2.1 变体；fullcov01 hold-all OOM 的修正形态）
            GRID_LIVE.addLast(pos);
            while (GRID_LIVE.size() > GRID_LAG) {
                ChunkPos old = GRID_LIVE.removeFirst();
                tm.removeTicketWithLevel(FORM_TYPE, old, DRIVE_LEVEL, old);
            }
            tm.addTicketWithLevel(FORM_TYPE, pos, DRIVE_LEVEL, pos);
            driveSeq = seq;
            System.out.println(String.format(Locale.ROOT, "[FP-DRV] ev=move seq=%d cx=%d cz=%d t=%.1f",
                    seq, cx, cz, ms(now())));
        } catch (Throwable t) {
            System.out.println("[FP-DRV] grid move FAILED seq=" + seq + ": " + t);
            if (t instanceof RuntimeException rt) throw rt;
            if (t instanceof Error err) throw err;
            throw new RuntimeException(t);
        }
    }

    // ---------------- level 边界对照臂驱动（260920-06，design-260920-06 §2/§3） ----------------

    /** 负臂档（默认 34 = byDistanceFromFull(1) = INITIALIZE_LIGHT，恰停 light() 之前）。 */
    private static final int EDGE_LVL_N = edgeLvl(0, 34);
    /** 正臂档（默认 33 = byDistanceFromFull(0) = FULL）。 */
    private static final int EDGE_LVL_P = edgeLvl(1, 33);
    private static final int EDGE_CX = edgeAt(0, 160);
    private static final int EDGE_CZ = edgeAt(1, 96);
    /** SETTLE 25s 同走廊/grid；每相 dwell 上限 30s（design §6 R3 预算 900s）。 */
    private static final long EDGE_SETTLE_NS = 25_000_000_000L;
    private static final long EDGE_PHASE_NS = 30_000_000_000L;

    private static int edgeLvl(int idx, int dflt) {
        String s = System.getProperty("coreswap.formedge.lvl");
        if (s == null) return dflt;
        String[] parts = s.split(":");
        return idx < parts.length ? Integer.parseInt(parts[idx].trim()) : dflt;
    }

    private static int edgeAt(int idx, int dflt) {
        String s = System.getProperty("coreswap.formedge.at");
        if (s == null) return dflt;
        String[] parts = s.split(":");
        return idx < parts.length ? Integer.parseInt(parts[idx].trim()) : dflt;
    }

    /** 相位：0=待 SETTLE（arm）→1=N 相待 probe→2=P 相待 probe→3=done。 */
    private static volatile int edgePhase = 0;
    private static volatile long edgePhaseStart = 0L;

    private static void edgeTick(ServerWorld world) {
        long t = System.nanoTime();
        switch (edgePhase) {
            case 0 -> {
                firstTickNanos = t;
                edgePhase = 1;
                edgePhaseStart = t;
                System.out.println(String.format(Locale.ROOT,
                        "[FP-EDGE] ev=init t=%.1f cx=%d cz=%d lvln=%d lvlp=%d", ms(now()), EDGE_CX, EDGE_CZ, EDGE_LVL_N, EDGE_LVL_P));
            }
            case 1 -> {
                if (t - firstTickNanos < EDGE_SETTLE_NS) return;
                try {
                    ServerChunkManager cm = (ServerChunkManager) world.getChunkManager();
                    ChunkTicketManager tm = ((ServerChunkManagerAccessor) cm).wgTicketManager();
                    ChunkPos pos = new ChunkPos(EDGE_CX, EDGE_CZ);
                    // BASE 基线 probe（design §7.5）：加票前直读运行时档位/状态，兜底 ambient 票据污染
                    edgeProbe(world, tm, pos, "BASE");
                    tm.addTicketWithLevel(FORM_TYPE, pos, EDGE_LVL_N, pos);
                    System.out.println(String.format(Locale.ROOT, "[FP-EDGE] ev=arm t=%.1f cx=%d cz=%d lvl=%d",
                            ms(now()), EDGE_CX, EDGE_CZ, EDGE_LVL_N));
                    edgePhase = 2;
                    edgePhaseStart = t;
                } catch (Throwable th) {
                    edgeFail("arm", th);
                }
            }
            case 2 -> {
                if (!edgePhaseDue(world, t, "N", EDGE_LVL_N, ChunkStatus.INITIALIZE_LIGHT)) return;
                try {
                    ServerChunkManager cm = (ServerChunkManager) world.getChunkManager();
                    ChunkTicketManager tm = ((ServerChunkManagerAccessor) cm).wgTicketManager();
                    ChunkPos pos = new ChunkPos(EDGE_CX, EDGE_CZ);
                    tm.removeTicketWithLevel(FORM_TYPE, pos, EDGE_LVL_N, pos);
                    tm.addTicketWithLevel(FORM_TYPE, pos, EDGE_LVL_P, pos);
                    edgePhase = 3;
                    edgePhaseStart = t;
                } catch (Throwable th) {
                    edgeFail("switch", th);
                }
            }
            case 3 -> {
                edgePhaseDue(world, t, "P", EDGE_LVL_P, ChunkStatus.FULL);
                if (edgePhase != 3) return;
                // P 相到期/到位后收尾（probe 已打）
                edgePhase = 4;
                System.out.println(String.format(Locale.ROOT, "[FP-EDGE] ev=done t=%.1f", ms(now())));
            }
            default -> { }
        }
    }

    /** 到期（30s 上限）或到位（required status 可得）时打 probe 行并返回 true（每相只打一次：由相位推进保证）。 */
    private static boolean edgePhaseDue(ServerWorld world, long t, String phase, int lvl, ChunkStatus required) {
        if (t - edgePhaseStart < EDGE_PHASE_NS && !edgeReady(world, required)) return false;
        try {
            ServerChunkManager cm = (ServerChunkManager) world.getChunkManager();
            ChunkTicketManager tm = ((ServerChunkManagerAccessor) cm).wgTicketManager();
            edgeProbe(world, tm, new ChunkPos(EDGE_CX, EDGE_CZ), phase);
        } catch (Throwable th) {
            edgeFail("probe-" + phase, th);
        }
        return true;
    }

    /** R5 到位确认：required status 的 chunk 可非阻塞取得（null = 未到位）。 */
    private static boolean edgeReady(ServerWorld world, ChunkStatus required) {
        try {
            ServerChunkManager cm = (ServerChunkManager) world.getChunkManager();
            return cm.getChunk(EDGE_CX, EDGE_CZ, required, false) != null;
        } catch (Throwable th) {
            return false;
        }
    }

    /** #118 硬门读数：lvl = 运行时 holder level（反射直读，非参数回显）；status = chunk.getStatus().getId()。 */
    private static void edgeProbe(ServerWorld world, ChunkTicketManager tm, ChunkPos pos, String phase) {
        int lvl = -1;
        String status = "absent";
        try {
            Method m = tm.getClass().getDeclaredMethod("getChunkHolder", long.class);
            m.setAccessible(true);
            Object holder = m.invoke(tm, pos.toLong());
            if (holder instanceof ChunkHolder ch) lvl = ch.getLevel();
        } catch (Throwable th) {
            System.out.println("[FP-EDGE] ev=holder-reflect-FAILED phase=" + phase + ": " + th);
        }
        try {
            ServerChunkManager cm = (ServerChunkManager) world.getChunkManager();
            Chunk c = cm.getChunk(pos.x, pos.z, ChunkStatus.EMPTY, false);
            if (c != null) status = c.getStatus().toString(); // ChunkStatus.toString = Registries.CHUNK_STATUS.getId（ChunkStatus.java:407-409）
        } catch (Throwable th) {
            status = "read-failed:" + th.getClass().getSimpleName();
        }
        System.out.println(String.format(Locale.ROOT, "[FP-EDGE] ev=probe phase=%s t=%.1f cx=%d cz=%d lvl=%d status=%s%s",
                phase, ms(now()), pos.x, pos.z, lvl, status, "BASE".equals(phase) ? " base=1" : ""));
    }

    private static void edgeFail(String where, Throwable t) {
        System.out.println("[FP-EDGE] " + where + " FAILED: " + t);
        if (t instanceof RuntimeException rt) throw rt;
        if (t instanceof Error err) throw err;
        throw new RuntimeException(t);
    }
}
