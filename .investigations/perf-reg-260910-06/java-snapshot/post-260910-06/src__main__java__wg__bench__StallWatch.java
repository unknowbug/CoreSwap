package wg.bench;

/**
 * 停滞看门狗（260910-06，**诊断专用、默认完全关闭**）：{@code -Dcoreswap.stallwatch=<秒>} 打开后
 * 起一个守护线程，每 N 秒把**全部线程栈**打到 stdout。
 *
 * <p>存在理由：本机沙箱**拒访 JVM attach**（{@code jstack}/{@code jcmd} 均 exit 1），
 * 而「Chunky 任务 Processed 恒 0 + 服务器 CPU ≈ 0」这类停滞必须看线程栈才能定性（死锁？泄漏的锁？
 * 还是环境问题）。看门狗把线程栈变成**日志内证据**，不需要 attach。
 *
 * <p>纪律：默认关（生产零成本，只读 prop 一次）；间隔由调用方给出，避免每次进入 dump 的开销；
 * dump 内容进 stdout → gradle 日志，可直接 grep {@code [STALLWATCH]}。
 */
public final class StallWatch {
    private static volatile boolean started = false;

    private StallWatch() {}

    /** 幂等启动（多处调用只起一个线程）；未设 prop 时不做任何事。 */
    public static void start() {
        String prop = System.getProperty("coreswap.stallwatch");
        if (prop == null) return;
        long intervalSec;
        try { intervalSec = Long.parseLong(prop.trim()); } catch (Exception e) { intervalSec = 60L; }
        if (intervalSec <= 0) intervalSec = 60L;
        synchronized (StallWatch.class) {
            if (started) return;
            started = true;
        }
        final long iv = intervalSec * 1000L;
        Thread t = new Thread(() -> {
            while (true) {
                try {
                    Thread.sleep(iv);
                } catch (InterruptedException e) {
                    return;
                }
                try {
                    System.out.println("[STALLWATCH] === thread dump @ " + new java.util.Date()
                            + " (interval=" + (iv / 1000) + "s) ===");
                    for (java.util.Map.Entry<Thread, StackTraceElement[]> e : Thread.getAllStackTraces().entrySet()) {
                        Thread th = e.getKey();
                        System.out.println("[STALLWATCH] \"" + th.getName() + "\" id=" + th.getId()
                                + " state=" + th.getState() + " daemon=" + th.isDaemon());
                        for (StackTraceElement el : e.getValue()) {
                            System.out.println("[STALLWATCH]     at " + el);
                        }
                    }
                    System.out.println("[STALLWATCH] === end ===");
                } catch (Throwable ignored) {
                    // 看门狗自身绝不打断主流程
                }
            }
        }, "coreswap-stallwatch");
        t.setDaemon(true);
        t.start();
        System.out.println("[STALLWATCH] enabled interval=" + intervalSec + "s");
    }
}
