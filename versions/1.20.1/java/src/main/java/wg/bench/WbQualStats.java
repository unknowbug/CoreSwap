package wg.bench;

import java.util.Locale;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * 写回资格打点通道（260920-04 design-260920-04 §2，idk-K2a）：[WBQ] 逐 chunk 路径归属行 + [WBQ-SUM] 汇总行。
 * 普通 holder 类（非 mixin，禁 mixin 类反射自载 #176；形态 = 1.21.6 BlobProbeStats.java 修复后正确形态）。
 *
 * <p>开关：{@code -Dcoreswap.light.wbqual}（build.gradle -Pwbqual 映射，B6-1 门）。类加载读一次，
 * 默认关；关 = {@link #emit}/{@link #countLegacyEnter} 首行直返，调用点无逻辑分支差异（零行为变化）。
 *
 * <p>行格式（单行自包含，禁跨行状态依赖；输出 System.err，与 stdout 采集流分离按解析器规则合流）：
 * <pre>
 * [WBQ] ev=&lt;ev&gt; cx=&lt;x&gt; cz=&lt;z&gt; dk=&lt;domainKey&gt; x3=&lt;kx&gt; z3=&lt;kz&gt; [extra...]
 * [WBQ-SUM] ENTER=.. INIT0=.. PRECHECK_FAIL=.. COLLECT_FAIL=.. DOMAIN_SUBMIT=.. DEGRADE=..
 *           WB_DOMAIN=.. LEGACY_ENTER=.. LEGACY_WB=.. LEGACY_FB=.. VANILLA_RET=..
 * </pre>
 *
 * <p>dk/x3/z3 自洽轴（design §4 轴②）：dk 公式镜像 Mixin#wgLightDomainKey
 * （((long)floorDiv(cx,3)<<32) | (floorDiv(cz,3)&0xFFFFFFFF)），x3/z3 = cx/cz - 3*floorDiv(·,3)
 * = floorMod(·,3) ∈ {0,1,2}（域内 3×3 槽位，南带判读 = z3==2）。三值同源自 (cx,cz)，机械自洽。
 *
 * <p>汇总挂点：FormProbe.printSummary 同形态（FP-SUM，FormProbe.java:54-61/119-124）——on 时
 * static 块注册 shutdown hook 打一行（无既有 server-stop Fabric 钩子可挂，照 FP-SUM 形态兜底）。
 */
public final class WbQualStats {
    /** 门（类加载读一次，默认关）。P1 armed 行回显此值（[LIGHT-DOMAIN] hook armed wbqual=on/off）。 */
    public static final boolean on = System.getProperty("coreswap.light.wbqual") != null;

    public static final AtomicInteger ENTER = new AtomicInteger();
    public static final AtomicInteger INIT0 = new AtomicInteger();
    public static final AtomicInteger PRECHECK_FAIL = new AtomicInteger();
    public static final AtomicInteger COLLECT_FAIL = new AtomicInteger();
    public static final AtomicInteger DOMAIN_SUBMIT = new AtomicInteger();
    public static final AtomicInteger DEGRADE = new AtomicInteger();
    public static final AtomicInteger WB_DOMAIN = new AtomicInteger();
    public static final AtomicInteger LEGACY_ENTER = new AtomicInteger();
    public static final AtomicInteger LEGACY_WB = new AtomicInteger();
    public static final AtomicInteger LEGACY_FB = new AtomicInteger();
    public static final AtomicInteger VANILLA_RET = new AtomicInteger();
    /** 未知 ev 兜底计数（judge S3，260920-05）：禁归入 ENTER 制造假账；SUM 行随行输出。 */
    public static final AtomicInteger UNKNOWN = new AtomicInteger();

    private static final AtomicBoolean SUM_DONE = new AtomicBoolean(false);

    static {
        if (on) {
            // FormProbe 同形态：shutdown hook 兜底汇总（server stop 时打一行 [WBQ-SUM]）
            Runtime.getRuntime().addShutdownHook(new Thread(WbQualStats::sum, "wbqual-sum"));
        }
    }

    private WbQualStats() {
    }

    /** legacy per-chunk 路入口计数（wgLightLegacyTakeover HEAD；仅计数无事件行，ev 标签集不含 legacy-enter）。 */
    public static void countLegacyEnter() {
        if (!on) return;
        LEGACY_ENTER.incrementAndGet();
    }

    /**
     * 打一行 [WBQ]（gate 关零成本直返）并递增 ev 对应计数器。
     * dk/x3/z3 由 (cx,cz) 同源推导（见类注释自洽轴），extra 为预拼好的 "k=v" 片段（可空）。
     *
     * @param ev enter|init0|precheck-fail|collect-fail|domain-submit|degrade|wb|legacy-wb|legacy-fb|vanilla-ret
     */
    public static void emit(String ev, int cx, int cz, String... extra) {
        if (!on) return;
        counterOf(ev).incrementAndGet();
        StringBuilder sb = new StringBuilder(96);
        sb.append("[WBQ] ev=").append(ev)
                .append(" cx=").append(cx)
                .append(" cz=").append(cz)
                .append(" dk=").append(domainKey(cx, cz))
                .append(" x3=").append(Math.floorMod(cx, 3))
                .append(" z3=").append(Math.floorMod(cz, 3));
        for (String e : extra) {
            if (e != null && !e.isEmpty()) {
                sb.append(' ').append(e);
            }
        }
        // 不吞异常铁律：仅打点失败不影响主流程（stderr 可能关闭等 IO 异常），登记后继续
        try {
            System.err.println(sb);
        } catch (Throwable t) {
            System.out.println("[WBQ] emit fail ev=" + ev + " (" + cx + "," + cz + "): " + t);
        }
    }

    /** 汇总行（shutdown hook / server stop 时调用；幂等）。 */
    public static void sum() {
        if (!on) return;
        if (!SUM_DONE.compareAndSet(false, true)) return;
        System.err.println(String.format(Locale.ROOT,
                "[WBQ-SUM] ENTER=%d INIT0=%d PRECHECK_FAIL=%d COLLECT_FAIL=%d DOMAIN_SUBMIT=%d DEGRADE=%d"
                        + " WB_DOMAIN=%d LEGACY_ENTER=%d LEGACY_WB=%d LEGACY_FB=%d VANILLA_RET=%d UNKNOWN=%d",
                ENTER.get(), INIT0.get(), PRECHECK_FAIL.get(), COLLECT_FAIL.get(), DOMAIN_SUBMIT.get(),
                DEGRADE.get(), WB_DOMAIN.get(), LEGACY_ENTER.get(), LEGACY_WB.get(), LEGACY_FB.get(),
                VANILLA_RET.get(), UNKNOWN.get()));
    }

    /** 域键（Mixin wgLightDomainKey 镜像公式：3×3 网格对齐 floorDiv(cx,3), floorDiv(cz,3)，打包 long）。 */
    private static long domainKey(int cx, int cz) {
        return ((long) Math.floorDiv(cx, 3) << 32) | (Math.floorDiv(cz, 3) & 0xFFFFFFFFL);
    }

    private static AtomicInteger counterOf(String ev) {
        switch (ev) {
            case "enter": return ENTER;
            case "init0": return INIT0;
            case "precheck-fail": return PRECHECK_FAIL;
            case "collect-fail": return COLLECT_FAIL;
            case "domain-submit": return DOMAIN_SUBMIT;
            case "degrade": return DEGRADE;
            case "wb": return WB_DOMAIN;
            case "legacy-wb": return LEGACY_WB;
            case "legacy-fb": return LEGACY_FB;
            case "vanilla-ret": return VANILLA_RET;
            default: return UNKNOWN; // 未知 ev 独立计数（judge S3），不归 ENTER 防假账；行内 ev 字段仍如实打印
        }
    }
}
