package wg.bench;

/** 分版兼容缝（260912-01 D3 共享 Java 适配核）：共享文件不得出现版本分支/映射差异，全部收进本类。 */
public final class WgCompat {
    private WgCompat() {}

    /** bulk section 写回（C 线）默认开关。1.20.1 = true（C 线已验证）；1.21.6 = false（待自身 A/B 后翻转）。 */
    public static final boolean BULKWB_ON = true;

    /** A1a 写回跳空气默认开关。1.20.1 = true（A1a 已验证）；1.21.6 = false（待翻转）。 */
    public static final boolean SKIPAIR_ON = true;

    /** property 未设则取分版缺省；设了则「非 "0" 即真」（两版行为差异只在这两个常量）。 */
    public static boolean flag(String property, boolean fallback) {
        String v = System.getProperty(property);
        return v == null ? fallback : !"0".equals(v);
    }

    /**
     * storage 长数组帧写出（1.20.1 契约，260912-02 F1 分版缝）。
     * <p>1.20.1：{@code writeLongArray} = **VarInt 长度前缀 + longs**，与同版
     * {@code PalettedContainer.readPacket} 的 {@code readLongArray(long[])} 配对。
     * 共享 {@code BulkWb} 只调本缝，自身不含版本分支。
     */
    public static void writeStorageLongs(net.minecraft.network.PacketByteBuf pb, long[] data) {
        pb.writeLongArray(data);
    }
}
