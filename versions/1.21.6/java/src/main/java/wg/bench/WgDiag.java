package wg.bench;

/**
 * 260905-12 采集提速第一件（NEXT_SESSION 采集提速三件套 #1）：
 * 诊断 mixin 的 chunk 级过滤支撑。population 入口行（ChunkRandomSeedLogMixin
 * wg$logPop，每 chunk 恒一条）写入线程当前 chunk；各诊断 mixin 打点前查
 * 目标 chunk 过滤（env WG_DIAGCHUNK / sysprop wg.diagchunk，格式 "x,z"）。
 *
 * 设计约束：
 * - ThreadLocal：vanilla chunk 生成多线程，static 单值会串 chunk（ décorator 与
 *   population 同线程，ThreadLocal 语义安全）。
 * - 位于 wg.bench 而非 wg.bench.mixin：mixin 包禁止非 mixin 类（compiler-idioms #12）。
 * - 未设目标 chunk 时行为与旧版完全一致（全量输出）；设了则只输出目标 chunk 行，
 *   SEEDLOG population 行不滤（每 chunk 一条的锚点，非噪声源）。
 */
public final class WgDiag {
    /** ThreadLocal<{cx, cz}>；Long.MIN_VALUE 占位 = 未设置。 */
    private static final ThreadLocal<long[]> CHUNK =
            ThreadLocal.withInitial(() -> new long[]{Long.MIN_VALUE, Long.MIN_VALUE});

    private static final int NO_TARGET = Integer.MIN_VALUE;
    private static final int TARGET_X;
    private static final int TARGET_Z;
    private static final boolean TARGET_SET;
    private static boolean bannerPrinted = false;

    static {
        String spec = System.getProperty("wg.diagchunk");
        if (spec == null) spec = System.getenv("WG_DIAGCHUNK");
        int tx = NO_TARGET, tz = 0;
        if (spec != null) {
            String[] parts = spec.split(",");
            if (parts.length == 2) {
                try {
                    tx = Integer.parseInt(parts[0].trim());
                    tz = Integer.parseInt(parts[1].trim());
                } catch (NumberFormatException e) {
                    tx = NO_TARGET;
                }
            }
        }
        TARGET_X = tx;
        TARGET_Z = tz;
        TARGET_SET = tx != NO_TARGET;
    }

    private WgDiag() {}

    /** population 入口行调用：记录本线程当前正在做 feature 的 chunk。 */
    public static void setCurrent(int cx, int cz) {
        long[] a = CHUNK.get();
        a[0] = cx;
        a[1] = cz;
    }

    /**
     * 无坐标上下文的打点（THJ/BEE 等）：目标未设 → 放行（旧行为）；
     * 目标已设 → 仅当本线程当前 chunk == 目标时放行。
     * ⚠️ 260905-13 E5 修正：setCurrent 实际收到的是 setPopulationSeed 的 chunk 起始
     * block 坐标（实测 population 行 "chunk 464 -256" = chunk 29,-16），此处统一 >>4。
     */
    public static boolean curAllowed() {
        if (!TARGET_SET) return true;
        long[] a = CHUNK.get();
        return (a[0] >> 4) == TARGET_X && (a[1] >> 4) == TARGET_Z;
    }

    /** 有 BlockPos 的打点（CNT/SQ 等）：按 pos 所在 chunk 对目标过滤。 */
    public static boolean posAllowed(int x, int z) {
        if (!TARGET_SET) return true;
        return (x >> 4) == TARGET_X && (z >> 4) == TARGET_Z;
    }

    /** 目标过滤激活时打一次性 banner，确认过滤生效（防死参数假判别，workflow-patterns #20）。 */
    public static synchronized void banner() {
        if (TARGET_SET && !bannerPrinted) {
            bannerPrinted = true;
            System.out.println("[DIAG] chunk filter active: " + TARGET_X + "," + TARGET_Z);
        }
    }
}
