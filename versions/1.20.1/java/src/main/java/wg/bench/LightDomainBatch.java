// LightDomainBatch — CP-1（260916-01，.b1 域批中心化）Java 侧域批登记器。
// 职责（.b1 §1.2a）：按 3×3 网格对齐域（domainKey = floorDiv(cx,3),floorDiv(cz,3)）收拢
// center chunk 提交；9 中心到齐即封板投递（taskFactory）；宽限期超时按当前已到齐中心封板
//（未到齐中心 = 未提交者无 future，不产出；已提交者由任务侧按现役 per-chunk 路径降级重放）。
// 在飞去重：同一 centerPos 重复提交返回既有 future（去重对象 = 收拢工作，非计算结果缓存）。
// ⚠️ 放在 wg.bench 而非 wg.bench.mixin：mixin 包禁止任何非 mixin 类含 static nested
//（compiler-idioms #12，IllegalClassLoadError 直接判据）——.b1 设计案原文「mixin 同文件包内」
// 在此勘误，实体类移出 mixin 包。
// 状态：未编译验证（主会话负责 gradle compile）。
package wg.bench;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

public final class LightDomainBatch {
    private LightDomainBatch() {}

    /** 宽限期上限（R1）：超时按当前已到齐中心封板；负/0 = 无宽限（不推荐）。 */
    public static final int GRACE_MS = Integer.getInteger("coreswap.light.domainbatch.gracems", 10_000);

    /** 自证计数（#118 硬门数据源）：封板数 / 超时封板数 / 重复提交数 / 降级重放 chunk 数。 */
    public static final AtomicInteger SEALED = new AtomicInteger();
    public static final AtomicInteger TIMEDOUT = new AtomicInteger();
    public static final AtomicInteger DUP = new AtomicInteger();
    public static final AtomicInteger DEGRADED = new AtomicInteger();

    /** 封板回调：由 mixin 注册（投递 Util.getMainWorkerExecutor 执行真正的收集+JNI+写回）。 */
    public interface Task {
        void run(State st);
    }

    public static volatile Task taskFactory;

    /** 一个域批的在飞状态。ctx = 提交方捕获的执行上下文（provider/tacs/world/bottomSection）。 */
    public static final class State {
        public final long key;
        public volatile Object ctx;
        public volatile boolean timedOut;
        public final ConcurrentHashMap<Long, CompletableFuture<Object>> futures = new ConcurrentHashMap<>();
        public final ConcurrentHashMap<Long, Object> chunks = new ConcurrentHashMap<>();
        /** 提交线程侧收集的 blocks9 快照（CP-1 260916-01 崩溃修复：容器读只在 light 调用线程，
         *  域任务零 PalettedContainer 访问——writePacket lock 检测器与 5×5 窗重叠并发读相撞）。 */
        public final ConcurrentHashMap<Long, int[]> blocks9s = new ConcurrentHashMap<>();
        // 260919-03（.b2 C-3）：packed 帧快照（同线程同构收集；packedMeta=432、
        // packedLens=[palLen,stoLen]、pal/sto 为实长副本）。packed 提交者四 map 同 key 在位。
        public final ConcurrentHashMap<Long, int[]> packedMetas = new ConcurrentHashMap<>();
        public final ConcurrentHashMap<Long, int[]> packedPals = new ConcurrentHashMap<>();
        public final ConcurrentHashMap<Long, long[]> packedStos = new ConcurrentHashMap<>();
        public final ConcurrentHashMap<Long, int[]> packedLens = new ConcurrentHashMap<>();

        State(long key) {
            this.key = key;
        }
    }

    private static final ConcurrentHashMap<Long, State> DOMAINS = new ConcurrentHashMap<>();

    /**
     * centerPos 提交进其域批。返回该 chunk 的完成 future（重复提交返回既有 future，DUP 计数）。
     * 9 中心到齐立即封板；否则等宽限期超时封板。
     */
    public static CompletableFuture<Object> submit(long centerPos, long domainKey, Object ctx, Object chunk, int[] blocks9Snapshot) {
        State st = DOMAINS.compute(domainKey, (k, cur) -> {
            State s = (cur != null) ? cur : new State(k);
            if (cur == null && GRACE_MS > 0) {
                CompletableFuture.delayedExecutor(GRACE_MS, TimeUnit.MILLISECONDS).execute(() -> seal(s, true));
            }
            s.ctx = ctx; // 同域同世界，重复赋值幂等
            return s;
        });
        CompletableFuture<Object> fut = st.futures.putIfAbsent(centerPos, new CompletableFuture<>());
        if (fut == null) {
            fut = st.futures.get(centerPos);
            st.chunks.put(centerPos, chunk);
            st.blocks9s.put(centerPos, blocks9Snapshot);
        } else {
            DUP.incrementAndGet();
        }
        if (st.futures.size() >= 9) {
            seal(st, false);
        }
        return fut;
    }

    /** packed 提交（260919-03 .b2 C-3）：同 submit，快照 = packed 帧四件套（blocks9s 不落）。 */
    public static CompletableFuture<Object> submitPacked(long centerPos, long domainKey, Object ctx, Object chunk,
                                                          int[] meta, int[] pal, long[] sto, int[] lens) {
        State st = DOMAINS.compute(domainKey, (k, cur) -> {
            State s = (cur != null) ? cur : new State(k);
            if (cur == null && GRACE_MS > 0) {
                CompletableFuture.delayedExecutor(GRACE_MS, TimeUnit.MILLISECONDS).execute(() -> seal(s, true));
            }
            s.ctx = ctx;
            return s;
        });
        CompletableFuture<Object> fut = st.futures.putIfAbsent(centerPos, new CompletableFuture<>());
        if (fut == null) {
            fut = st.futures.get(centerPos);
            st.chunks.put(centerPos, chunk);
            st.packedMetas.put(centerPos, meta);
            st.packedPals.put(centerPos, pal);
            st.packedStos.put(centerPos, sto);
            st.packedLens.put(centerPos, lens);
        } else {
            DUP.incrementAndGet();
        }
        if (st.futures.size() >= 9) {
            seal(st, false);
        }
        return fut;
    }

    /** 封板（幂等：remove(key, st) 保证只执行一次）。 */
    static void seal(State st, boolean timedOut) {
        if (DOMAINS.remove(st.key, st)) {
            st.timedOut = timedOut;
            (timedOut ? TIMEDOUT : SEALED).incrementAndGet();
            Task t = taskFactory;
            if (t != null) {
                t.run(st);
            } else {
                // 未注册回调 = 接线缺陷：loud fail（不吞），全部 future 异常完成
                st.futures.forEach((pos, f) -> f.completeExceptionally(new IllegalStateException("LightDomainBatch sealed without taskFactory")));
            }
        }
    }

    /** 自证行（#118：正/负成对证据的数据面）。 */
    public static String selfProof() {
        return "sealed=" + SEALED.get() + " timeout=" + TIMEDOUT.get() + " dup=" + DUP.get();
    }
}
