package wg;

public class WorldGen {
    static {
        System.loadLibrary("worldgen");
    }

    public static native long nativeProbe(long seed, int x, int z);

    public static void main(String[] args) {
        long seed = Long.parseLong(args.length > 0 ? args[0] : "123456789");
        int x = args.length > 1 ? Integer.parseInt(args[1]) : 42;
        int z = args.length > 2 ? Integer.parseInt(args[2]) : -17;
        long got = nativeProbe(seed, x, z);
        System.out.println("seed=" + seed + " x=" + x + " z=" + z + " => " + got);
    }
}
