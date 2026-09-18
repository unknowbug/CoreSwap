package wg.bench;

/**
 * CoreSwap 诊断（260918-03）：BlobProbeMixin 的 CALLS/WRITTEN 计数 holder。
 * 修复背景：此前计数器放在 mixin 类本体的 @Unique 静态字段上，BlobProbe 经
 * Class.forName 反射读取时触发 mixin 类本体类加载 → transformer 自变换失败
 * （RuntimeException: Mixin transformation of wg.bench.mixin.BlobProbeMixin failed），
 * stats 自 1.20.1 时代首跑起恒为哨兵初值 -1。计数器移到普通类后免反射直读。
 * 注意：mixin 织入目标类不加载 mixin 类本体，此失败只影响 stats 读取面，不影响注入。
 */
public final class BlobProbeStats {
    public static volatile int CALLS = 0;
    public static volatile int WRITTEN = 0;

    private BlobProbeStats() {
    }
}
