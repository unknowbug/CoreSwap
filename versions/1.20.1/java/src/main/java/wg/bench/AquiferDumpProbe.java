package wg.bench;

import net.minecraft.server.MinecraftServer;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.chunk.ChunkStatus;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.HashSet;
import java.util.Set;

/**
 * CoreSwap 诊断（残留 1830 水洞群判别探针，260904 worker 草稿，未编译验证）：
 * 预生成覆盖点集的 chunk region（FULL，触发完整 NOISE→SURFACE→CARVERS 管线），配合
 * AquiferDumpProbeMixin（hook AquiferSampler.Impl.apply HEAD/RETURN）对命中点 dump
 * aquifer 全链路值。行格式与 Rust 侧 WG_AQDUMP（WorldgenRust/src/aquifer.rs）完全一致，
 * 逐字段 diff → 判别「单向漂移 vs 边界震荡」（scout-aquifer-map.md §3 P-D1/P-D2/P-D4）。
 *
 * 用法（systemProperty，仿 EstDumpProbe）：
 *   -Daqdump.probe=1 -Daqdump.points=&lt;点文件路径&gt; [-Daqdump.out=&lt;输出文件，缺省 stdout&gt;]
 * 点文件行格式：`x y z`（# 注释）——与 Rust WG_AQDUMP 点文件同格式。
 * 建议叠加 -Vanilla（cpp.vanilla）确保 CoreSwap 原生 worldgen 关闭（aqdump.probe 已在
 * BenchMod anyProbe 列表 → replace 自动关，显式 -Vanilla 双保险）。
 * 输出头：AQDUMP seed=&lt;worldSeed&gt;（三查铁律：对比前先核两侧 seed 一致）。
 */
public class AquiferDumpProbe {
	/** 命中点集（pack 成 long，供 mixin 线程安全只读访问；mixin 先 load 后遍历期间不再变更）。 */
	static final Set<Long> POINTS = new HashSet<>();

	private static volatile String outPath;

	public static void run(MinecraftServer server) {
		String pointsFile = System.getProperty("aqdump.points");
		if (pointsFile == null) {
			System.out.println("[AQDUMP] missing -Daqdump.points, abort");
			server.stop(false);
			return;
		}
		outPath = System.getProperty("aqdump.out");
		try {
			for (String line : Files.readAllLines(Path.of(pointsFile))) {
				line = line.trim();
				if (line.isEmpty() || line.startsWith("#")) continue;
				String[] p = line.split("\\s+");
				if (p.length >= 3) {
					POINTS.add(BlockPos.asLong(Integer.parseInt(p[0]), Integer.parseInt(p[1]), Integer.parseInt(p[2])));
				}
			}
		} catch (Exception e) {
			System.out.println("[AQDUMP] failed to read points file " + pointsFile + ": " + e);
			server.stop(false);
			return;
		}
		int minCx = Integer.MAX_VALUE, maxCx = Integer.MIN_VALUE;
		int minCz = Integer.MAX_VALUE, maxCz = Integer.MIN_VALUE;
		for (long pt : POINTS) {
			int cx = BlockPos.unpackLongX(pt) >> 4;
			int cz = BlockPos.unpackLongZ(pt) >> 4;
			minCx = Math.min(minCx, cx); maxCx = Math.max(maxCx, cx);
			minCz = Math.min(minCz, cz); maxCz = Math.max(maxCz, cz);
		}
		var world = server.getOverworld();
		// 输出头（Rust 侧由 aqdump_seed 打同一行；三查：与 WG_AQDUMP 侧 seed 必须一致）
		System.out.println("AQDUMP seed=" + world.getSeed());
		System.out.println("[AQDUMP] points=" + POINTS.size() + " out=" + (outPath == null ? "stdout" : outPath)
				+ " region cx[" + minCx + "," + maxCx + "] cz[" + minCz + "," + maxCz + "]");
		for (int cz = minCz; cz <= maxCz; cz++) {
			for (int cx = minCx; cx <= maxCx; cx++) {
				world.getChunk(cx, cz, ChunkStatus.FULL, true);
			}
		}
		System.out.println("[AQDUMP] done");
		server.stop(false);
	}

	/** mixin 调用：点集命中判断。 */
	public static boolean isHit(int x, int y, int z) {
		return POINTS.contains(BlockPos.asLong(x, y, z));
	}

	/** mixin 调用：单行输出（stdout 或追加文件；同步防 worldgen worker 线程交错）。 */
	public static void write(String line) {
		String out = outPath;
		if (out == null) {
			System.out.println(line);
			return;
		}
		try {
			Path p = Path.of(out).toAbsolutePath().normalize();
			if (p.getParent() != null) Files.createDirectories(p.getParent());
			synchronized (AquiferDumpProbe.class) {
				Files.write(p, (line + "\n").getBytes(StandardCharsets.UTF_8),
						StandardOpenOption.CREATE, StandardOpenOption.APPEND);
			}
		} catch (Throwable t) {
			System.out.println("[AQDUMP] write failed: " + t);
		}
	}

	/** 首链捕获器（NaN/MAX/-1 = na；与 Rust AqDump 字段一一对应）。 */
	public static final class WgCap {
		public double floodedRaw = Double.NaN, flooded = Double.NaN;
		public double spreadRaw = Double.NaN, erosion = Double.NaN, depth = Double.NaN, barrier = Double.NaN, fw = Double.NaN;
		public int spreadRnd = Integer.MAX_VALUE, spreadBase = Integer.MAX_VALUE;
		public int gate = -1, bl = -1;
		public String est = null;
		public boolean chainRan = false;
	}
}
