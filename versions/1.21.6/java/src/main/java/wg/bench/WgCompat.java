package wg.bench;

/** 分版兼容缝（260912-01 D3 共享 Java 适配核）：共享文件不得出现版本分支/映射差异，全部收进本类。 */
public final class WgCompat {
    private WgCompat() {}

    /** bulk section 写回（C 线）默认开关。两版均已验证 ⇒ 均为 true。1.21.6 于 **260913-01** 翻转：
     *  自身 A/B 三臂（默认 / {@code bulkwb=0} / {@code bulkwb=1}）× 三维（overworld/nether/end）全部等价
     *  （两层指纹整文件 sha256 同一，逐 chunk 多重集 18/18 全等；nether/end 首次有载体）。
     *  {@code -Dcoreswap.bulkwb=0} 可即时回退到逐块路径。 */
    public static final boolean BULKWB_ON = true;

    /** A1a 写回跳空气默认开关。1.20.1 = true（A1a 已验证）；1.21.6 = false（1.21.6 侧尚未验证 ⇒ 缺省关；
     *  {@code -Dcoreswap.skipair=1} 可强制开启做自身 A/B）。 */
    public static final boolean SKIPAIR_ON = false;

    /** property 未设则取分版缺省；设了则「非 "0" 即真」（两版行为差异只在这两个常量）。 */
    public static boolean flag(String property, boolean fallback) {
        String v = System.getProperty(property);
        return v == null ? fallback : !"0".equals(v);
    }

    /**
     * storage 长数组帧写出（1.21.6 契约，260912-02 F1 分版缝）。
     * <p>1.21.6 把 storage 段从「VarInt 长度前缀 + longs」改为「**定长、无前缀**」：
     * {@code PalettedContainer.readPacket} 调 {@code readFixedLengthLongArray(long[])}
     * （1.20.1 同位置为 {@code readLongArray(long[])}），故写端必须随之走本缝。
     * <p>证据：{@code .investigations/shared-java-core-260912-02/evidence/A1-*.txt}
     * （两版 {@code readPacket} 指令偏移逐条相同，唯一差异 = 第 39 条调用目标）；
     * 运行级判别 {@code evidence/C1-log-1216-probe-b2a-WBTEST.log}（前缀帧 × 定长读端 ⇒
     * {@code [WG-BULKWB-TEST] FAIL branch distinct=3} + {@code EntryMissingException} 级联）。
     */
    public static void writeStorageLongs(net.minecraft.network.PacketByteBuf pb, long[] data) {
        pb.writeFixedLengthLongArray(data);
    }
}
