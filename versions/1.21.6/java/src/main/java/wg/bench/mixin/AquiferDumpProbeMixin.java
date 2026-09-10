package wg.bench.mixin;

import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.ChunkSectionPos;
import net.minecraft.util.math.MathHelper;
import net.minecraft.util.math.random.Random;
import net.minecraft.util.math.random.RandomSplitter;
import net.minecraft.world.biome.source.util.VanillaBiomeParameters;
import net.minecraft.world.dimension.DimensionType;
import net.minecraft.world.gen.chunk.AquiferSampler;
import net.minecraft.world.gen.chunk.ChunkNoiseSampler;
import net.minecraft.world.gen.densityfunction.DensityFunction;
import org.apache.commons.lang3.mutable.MutableDouble;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.lang.reflect.Field;
import java.util.Locale;

/**
 * CoreSwap 诊断（残留 1830 判别探针，260904 worker 草稿，未编译验证）：
 * hook AquiferSampler.Impl.apply(NoisePos;D)BlockState —— HEAD 复算全链路值（@Shadow 直读
 * vanilla 私有 sampler 字段，无 CellCache 反射——-288 教训），RETURN 捕获 vanilla 真实
 * decision 后统一输出一行。行格式/字段名/顺序与 Rust WG_AQDUMP（aquifer.rs）逐字段一致。
 *
 * 门控：-Daqdump.probe=1（静态 final，门控关时两处注入首行即返回，零热路径成本）。
 * 点集：-Daqdump.points（AquiferDumpProbe.load）；输出：-Daqdump.out（缺省 stdout）。
 *
 * 复算纪律（只读旁观）：
 * - blockPositions/waterLevels 只读；miss 时本地重放 split/链式计算，不写回缓存
 *   （vanilla 随后自填同值，零污染）。
 * - est 走公开方法 chunkNoiseSampler.estimateSurfaceHeight（精确值缓存 surfaceHeightEstimateCache
 *   与 vanilla 自身复用同值；不触碰 CellCache）。
 * - d/e/f/bl/decision 公式按 AquiferSampler.java 静态复刻（已证零偏离，对比目的是输入值）；
 *   decision 以 RETURN 捕获的 vanilla 真实返回值为准。
 */
@Mixin(AquiferSampler.Impl.class)
public abstract class AquiferDumpProbeMixin {

	// ---- @Shadow：vanilla Impl 私有字段直读（无反射）----
	@Shadow @Final private DensityFunction barrierNoise;
	@Shadow @Final private DensityFunction fluidLevelFloodednessNoise;
	@Shadow @Final private DensityFunction fluidLevelSpreadNoise;
	@Shadow @Final private DensityFunction fluidTypeNoise;
	@Shadow @Final private DensityFunction erosionDensityFunction;
	@Shadow @Final private DensityFunction depthDensityFunction;
	@Shadow @Final private RandomSplitter randomDeriver;
	@Shadow @Final private AquiferSampler.FluidLevel[] waterLevels;
	@Shadow @Final private long[] blockPositions;
	@Shadow @Final private ChunkNoiseSampler chunkNoiseSampler;
	@Shadow @Final private int startX;
	@Shadow @Final private int startY;
	@Shadow @Final private int startZ;
	@Shadow @Final private int sizeX;
	@Shadow @Final private int sizeZ;

	@Unique private static final boolean AQ_ON = System.getProperty("aqdump.probe") != null;
	@Unique private static final ThreadLocal<String> WG_STASH = new ThreadLocal<>();
	@Unique private static final int[][] WG_OFFSETS = new int[][]{
			{0, 0}, {-2, -1}, {-1, -1}, {0, -1}, {1, -1}, {-3, 0}, {-2, 0}, {-1, 0}, {1, 0}, {-2, 1}, {-1, 1}, {0, 1}, {1, 1}
	};
	/** AquiferSampler.FluidLevel.y 是包私有 final —— 仅此一处小反射（读 y；state 走 getBlockState(MIN_VALUE) 技巧免反射）。 */
	@Unique private static volatile Field WG_FL_Y;

	@Inject(method = "apply(Lnet/minecraft/world/gen/densityfunction/DensityFunction$NoisePos;D)Lnet/minecraft/block/BlockState;",
			at = @At("HEAD"))
	private void wgAqDumpHead(DensityFunction.NoisePos pos, double density, CallbackInfoReturnable<BlockState> cir) {
		if (!AQ_ON || !wg.bench.AquiferDumpProbe.isHit(pos.blockX(), pos.blockY(), pos.blockZ())) return;
		WG_STASH.set(this.wgBuildLine(pos.blockX(), pos.blockY(), pos.blockZ(), density));
	}

	@Inject(method = "apply(Lnet/minecraft/world/gen/densityfunction/DensityFunction$NoisePos;D)Lnet/minecraft/block/BlockState;",
			at = @At("RETURN"))
	private void wgAqDumpReturn(DensityFunction.NoisePos pos, double density, CallbackInfoReturnable<BlockState> cir) {
		if (!AQ_ON) return;
		String line = WG_STASH.get();
		if (line == null) return;
		WG_STASH.remove();
		BlockState ret = cir.getReturnValue();
		String decision;
		if (ret == null) {
			decision = "null"; // vanilla null → ChainedBlockSource → stone（Rust -1/Rock）
		} else if (ret.isOf(Blocks.WATER)) {
			decision = "BLOCK:1";
		} else if (ret.isOf(Blocks.LAVA)) {
			decision = "BLOCK:2";
		} else if (ret.isAir()) {
			decision = "BLOCK:0";
		} else {
			decision = "BLOCK:other";
		}
		wg.bench.AquiferDumpProbe.write(line + " decision=" + decision);
	}

	// ================= 复算链（AquiferSampler.java L145-450 逐行镜像）=================

	@Unique
	private String wgBuildLine(int i, int j, int k, double density) {
		StringBuilder b = new StringBuilder(640);
		b.append("AQDUMP x=").append(i).append(" y=").append(j).append(" z=").append(k)
				.append(" density=").append(wgF(density));
		if (density > 0.0) {
			// vanilla HEAD 早退（Rust decision=Rock:density>0）；链路值全部 na
			b.append(" floodedRaw=na flooded=na spreadRaw=na spreadRnd=na spreadBase=na erosion=na depth=na gate=na barrier=na bl=na f=na est=na")
					.append(" fl2=na fl3=na fl4=na opq=(na,na,na) r=na s=na t=na")
					.append(" d=na fq=na gpq=na cdE=na e=na cdG=na g=na cdH=na h=na");
			return b.toString();
		}
		wg.bench.AquiferDumpProbe.WgCap cap = new wg.bench.AquiferDumpProbe.WgCap();
		// default fluid（overworld：NoiseChunkGenerator L78-84 语义，Rust aquifer.rs default_level 同）
		int l = Math.floorDiv(i - 5, 16), m = Math.floorDiv(j + 1, 12), n = Math.floorDiv(k - 5, 16);
		int o = Integer.MAX_VALUE, p = Integer.MAX_VALUE, q = Integer.MAX_VALUE;
		long r = 0L, s = 0L, t = 0L;
		for (int u = 0; u <= 1; u++) {
			for (int v = -1; v <= 1; v++) {
				for (int w = 0; w <= 1; w++) {
					int x = l + u, y = m + v, z = n + w;
					int aa = this.wgIndex(x, y, z);
					long ab = this.blockPositions[aa];
					if (ab == Long.MAX_VALUE) {
						// 只读重放：不写回 blockPositions（vanilla 随后自填同值）
						Random random = this.randomDeriver.split(x, y, z);
						ab = BlockPos.asLong(x * 16 + random.nextInt(10), y * 12 + random.nextInt(9), z * 16 + random.nextInt(10));
					}
					int ad = BlockPos.unpackLongX(ab) - i;
					int ae = BlockPos.unpackLongY(ab) - j;
					int af = BlockPos.unpackLongZ(ab) - k;
					int ag = ad * ad + ae * ae + af * af;
					if (o >= ag) { t = s; s = r; r = ab; q = p; p = o; o = ag; }
					else if (p >= ag) { t = s; s = ab; q = p; p = ag; }
					else if (q >= ag) { t = ab; q = ag; }
				}
			}
		}
		// fl2（r 链）带捕获；fl3/fl4 链不捕获（首链记录 wins，与 Rust 一致）
		// 注：water-over-lava 检查（apply L215）的 getFluidLevel 直调不捕获——vanilla 在 fl2 之后才调用，
		     // 首链已定或 fl2 缓存命中（chainRan=false→na），与 Rust suppress 语义一致。
		AquiferSampler.FluidLevel fl2 = this.wgWaterLevelCap(r, cap);
		AquiferSampler.FluidLevel fl3 = this.wgWaterLevelCap(s, null);
		AquiferSampler.FluidLevel fl4 = this.wgWaterLevelCap(t, null);
		double d = 1.0 - Math.abs(p - o) / 25.0;
		MutableDouble md = new MutableDouble(Double.NaN);
		DensityFunction.UnblendedNoisePos bpos = new DensityFunction.UnblendedNoisePos(i, j, k);
		double cdE = this.wgCalcDensity(bpos, md, fl2, fl3, cap);
		double e = d * cdE;
		double fq = 1.0 - Math.abs(q - o) / 25.0;
		double cdG = 0.0, g = 0.0, cdH = 0.0, h = 0.0;
		boolean fPos = fq > 0.0;
		if (fPos) {
			cdG = this.wgCalcDensity(bpos, md, fl2, fl4, cap);
			g = d * fq * cdG;
		}
		double gpq = 1.0 - Math.abs(q - p) / 25.0;
		boolean g2Pos = gpq > 0.0;
		if (g2Pos) {
			cdH = this.wgCalcDensity(bpos, md, fl3, fl4, cap);
			h = d * gpq * cdH;
		}
		// 字段顺序与 Rust print 严格一致
		b.append(" floodedRaw=").append(wgF(cap.floodedRaw)).append(" flooded=").append(wgF(cap.flooded))
				.append(" spreadRaw=").append(wgF(cap.spreadRaw)).append(" spreadRnd=").append(wgI(cap.spreadRnd))
				.append(" spreadBase=").append(wgI(cap.spreadBase))
				.append(" erosion=").append(wgF(cap.erosion)).append(" depth=").append(wgF(cap.depth))
				.append(" gate=").append(cap.gate < 0 ? "na" : String.valueOf(cap.gate))
				.append(" barrier=").append(wgF(cap.barrier))
				.append(" bl=").append(cap.bl < 0 ? "na" : String.valueOf(cap.bl))
				.append(" f=").append(wgF(cap.fw)).append(" est=").append(cap.est == null ? "na" : cap.est)
				.append(" fl2=").append(wgFl(fl2)).append(" fl3=").append(wgFl(fl3)).append(" fl4=").append(wgFl(fl4))
				.append(" opq=(").append(wgI(o)).append(',').append(wgI(p)).append(',').append(wgI(q)).append(')')
				.append(" r=").append(wgBlob(r, fl2)).append(" s=").append(wgBlob(s, fl3)).append(" t=").append(wgBlob(t, fl4))
				.append(" d=").append(wgF(d)).append(" fq=").append(wgF(fq)).append(" gpq=").append(wgF(gpq))
				.append(" cdE=").append(wgF(cdE)).append(" e=").append(wgF(e))
				.append(" cdG=").append(fPos ? wgF(cdG) : "na").append(" g=").append(fPos ? wgF(g) : "na")
				.append(" cdH=").append(g2Pos ? wgF(cdH) : "na").append(" h=").append(g2Pos ? wgF(h) : "na");
		return b.toString();
	}

	@Unique
	private int wgIndex(int x, int y, int z) {
		int i = x - this.startX, j = y - this.startY, k = z - this.startZ;
		return (j * this.sizeZ + k) * this.sizeX + i;
	}

	@Unique
	private AquiferSampler.FluidLevel wgWaterLevelCap(long pos, wg.bench.AquiferDumpProbe.WgCap cap) {
		int i = BlockPos.unpackLongX(pos), j = BlockPos.unpackLongY(pos), k = BlockPos.unpackLongZ(pos);
		int idx = this.wgIndex(Math.floorDiv(i, 16), Math.floorDiv(j, 12), Math.floorDiv(k, 16));
		AquiferSampler.FluidLevel fl = this.waterLevels[idx];
		if (fl != null) return fl; // 缓存命中：cap.chainRan 保持 false → est/bl/f 等 na（与 Rust 一致）
		return this.wgFluidLevelCap(i, j, k, cap); // 只读重放，不写 waterLevels
	}

	@Unique
	private AquiferSampler.FluidLevel wgDefaultLevel(int blockY) {
		return blockY < -54
				? new AquiferSampler.FluidLevel(-54, Blocks.LAVA.getDefaultState())
				: new AquiferSampler.FluidLevel(63, Blocks.WATER.getDefaultState());
	}

	@Unique
	private AquiferSampler.FluidLevel wgFluidLevelCap(int blockX, int blockY, int blockZ, wg.bench.AquiferDumpProbe.WgCap cap) {
		AquiferSampler.FluidLevel defaultFl = this.wgDefaultLevel(blockY);
		int i = Integer.MAX_VALUE;
		int j = blockY + 12, k = blockY - 12;
		boolean bl = false;
		for (int[] is : WG_OFFSETS) {
			int l = blockX + ChunkSectionPos.getBlockCoord(is[0]);
			int m = blockZ + ChunkSectionPos.getBlockCoord(is[1]);
			int n = this.chunkNoiseSampler.estimateSurfaceHeight(l, m); // 公开方法，精确值缓存
			int o = n + 8;
			boolean bl2 = is[0] == 0 && is[1] == 0;
			if (bl2 && k > o) {
				if (cap != null && cap.est == null) cap.est = "early(k>o)";
				return defaultFl;
			}
			boolean bl3 = j > o;
			if (bl3 || bl2) {
				AquiferSampler.FluidLevel fl2 = this.wgDefaultLevel(o);
				if (!fl2.getBlockState(o).isAir()) {
					if (bl2) bl = true;
					if (bl3) return fl2;
				}
			}
			i = Math.min(i, n);
		}
		int py = this.wgFluidBlockY(blockX, blockY, blockZ, defaultFl, i, bl, cap);
		return new AquiferSampler.FluidLevel(py, this.wgFluidBlockState(blockX, blockY, blockZ, defaultFl, py));
	}

	@Unique
	private int wgFluidBlockY(int blockX, int blockY, int blockZ, AquiferSampler.FluidLevel defaultFl,
	                          int surfaceHeightEstimate, boolean bl, wg.bench.AquiferDumpProbe.WgCap cap) {
		DensityFunction.UnblendedNoisePos pos = new DensityFunction.UnblendedNoisePos(blockX, blockY, blockZ);
		// method_43718 镜像 + erosion/depth 捕获（Java && 短路：erosion 条件不满足时 depth 不采样 → depth=na）
		double er = this.erosionDensityFunction.sample(pos);
		double dp = Double.NaN;
		boolean gate;
		if (er < -0.225F) {
			dp = this.depthDensityFunction.sample(pos);
			gate = dp > 0.9F;
		} else {
			gate = false;
		}
		if (cap != null && !cap.chainRan) {
			cap.erosion = er;
			cap.depth = dp;
			cap.gate = gate ? 1 : 0;
		}
		if (gate) {
			if (cap != null && !cap.chainRan) {
				cap.chainRan = true;
				cap.bl = bl ? 1 : 0;
				cap.est = surfaceHeightEstimate == Integer.MAX_VALUE ? "MAX" : String.valueOf(surfaceHeightEstimate);
				// flooded/f/gate 分支未采样 → 保持 NaN（打印 na）
			}
			return wgFluidY(defaultFl); // d=e=-1 → 走 e>0 → default
		}
		int ii = surfaceHeightEstimate + 8 - blockY;
		double fw = bl ? MathHelper.clampedMap((double) ii, 0.0, 64.0, 1.0, 0.0) : 0.0;
		double gRaw = this.fluidLevelFloodednessNoise.sample(pos);
		double g = MathHelper.clamp(gRaw, -1.0, 1.0);
		double h = MathHelper.map(fw, 1.0, 0.0, -0.3, 0.8);
		double kk = MathHelper.map(fw, 1.0, 0.0, -0.8, 0.4);
		double dd = g - kk;
		double ee = g - h;
		if (cap != null && !cap.chainRan) {
			cap.chainRan = true;
			cap.bl = bl ? 1 : 0;
			cap.fw = fw;
			cap.floodedRaw = gRaw;
			cap.flooded = g;
			cap.est = surfaceHeightEstimate == Integer.MAX_VALUE ? "MAX" : String.valueOf(surfaceHeightEstimate);
		}
		if (ee > 0.0) {
			return wgFluidY(defaultFl);
		} else if (dd > 0.0) {
			return this.wgNoiseBasedFluidLevel(blockX, blockY, blockZ, surfaceHeightEstimate, cap);
		} else {
			return -32512; // DimensionType.field_35479 的字面量（worker 预警映射风险，主会话按源码语义替换）
		}
	}

	@Unique
	private int wgNoiseBasedFluidLevel(int blockX, int blockY, int blockZ, int surfaceHeightEstimate, wg.bench.AquiferDumpProbe.WgCap cap) {
		int k = Math.floorDiv(blockX, 16), l = Math.floorDiv(blockY, 40), m = Math.floorDiv(blockZ, 16);
		int base = l * 40 + 20;
		double raw = this.fluidLevelSpreadNoise.sample(new DensityFunction.UnblendedNoisePos(k, l, m));
		double d = raw * 10.0;
		int prnd = MathHelper.roundDownToMultiple(d, 3);
		if (cap != null && Double.isNaN(cap.spreadRaw)) {
			cap.spreadRaw = raw;
			cap.spreadRnd = prnd;
			cap.spreadBase = base;
		}
		// vanilla: q = base + prnd; return Math.min(surfaceHeightEstimate, q)（base 在 min 之前加——F2 修复 260908-11）
		return Math.min(surfaceHeightEstimate, base + prnd);
	}

	@Unique
	private BlockState wgFluidBlockState(int blockX, int blockY, int blockZ, AquiferSampler.FluidLevel defaultFl, int fluidLevel) {
		BlockState blockState = defaultFl.getBlockState(Integer.MIN_VALUE); // 取底层 state（免反射 .state）
		if (fluidLevel <= -10 && fluidLevel != -32512 && !blockState.isOf(Blocks.LAVA)) {
			int k = Math.floorDiv(blockX, 64), l = Math.floorDiv(blockY, 40), m = Math.floorDiv(blockZ, 64);
			double d = this.fluidTypeNoise.sample(new DensityFunction.UnblendedNoisePos(k, l, m));
			if (Math.abs(d) > 0.3) blockState = Blocks.LAVA.getDefaultState();
		}
		return blockState;
	}

	@Unique
	private double wgCalcDensity(DensityFunction.NoisePos pos, MutableDouble md,
	                             AquiferSampler.FluidLevel fl, AquiferSampler.FluidLevel fl2, wg.bench.AquiferDumpProbe.WgCap cap) {
		int i = pos.blockY();
		BlockState bs = fl.getBlockState(i);
		BlockState bs2 = fl2.getBlockState(i);
		boolean lavaWater = (bs.isOf(Blocks.LAVA) && bs2.isOf(Blocks.WATER)) || (bs.isOf(Blocks.WATER) && bs2.isOf(Blocks.LAVA));
		if (!lavaWater) {
			int j = Math.abs(wgFluidY(fl) - wgFluidY(fl2));
			if (j == 0) return 0.0;
			double d = 0.5 * (wgFluidY(fl) + wgFluidY(fl2));
			double e = i + 0.5 - d;
			double f = j / 2.0;
			double o = f - Math.abs(e);
			double q;
			if (e > 0.0) {
				double pp = 0.0 + o;
				q = pp > 0.0 ? pp / 1.5 : pp / 2.5;
			} else {
				double pp = 3.0 + o;
				q = pp > 0.0 ? pp / 3.0 : pp / 10.0;
			}
			double r;
			if (!(q < -2.0) && !(q > 2.0)) {
				double s = md.getValue();
				if (Double.isNaN(s)) {
					double tv = this.barrierNoise.sample(pos);
					md.setValue(tv);
					if (cap != null && Double.isNaN(cap.barrier)) cap.barrier = tv; // 首次 barrier 采样捕获
					r = tv;
				} else {
					r = s;
				}
			} else {
				r = 0.0;
			}
			return 2.0 * (r + q);
		}
		return 2.0;
	}

	// ================= 格式化助手（与 Rust 打印严格对齐）=================

	/** FluidLevel.y 包私有 → 小反射读（仅此一处；失败返回 Integer.MAX_VALUE → 打 MAX 可见，不静默当坐标）。 */
	@Unique
	private static int wgFluidY(AquiferSampler.FluidLevel fl) {
		try {
			Field f = WG_FL_Y;
			if (f == null) {
				f = AquiferSampler.FluidLevel.class.getDeclaredField("y");
				f.setAccessible(true);
				WG_FL_Y = f;
			}
			return f.getInt(fl);
		} catch (Throwable t) {
			return Integer.MAX_VALUE;
		}
	}

	@Unique
	private static String wgF(double v) {
		return Double.isNaN(v) ? "na" : String.format(Locale.ROOT, "%.6f", v);
	}

	@Unique
	private static String wgI(int v) {
		return v == Integer.MAX_VALUE ? "MAX" : String.valueOf(v);
	}

	@Unique
	private static int wgBlockId(AquiferSampler.FluidLevel fl) {
		BlockState bs = fl.getBlockState(Integer.MIN_VALUE);
		if (bs.isOf(Blocks.WATER)) return 1;
		if (bs.isOf(Blocks.LAVA)) return 2;
		if (bs.isAir()) return 0;
		return -1;
	}

	@Unique
	private static String wgFl(AquiferSampler.FluidLevel fl) {
		return "(" + wgFluidY(fl) + "," + wgBlockId(fl) + ")";
	}

	@Unique
	private static String wgBlob(long pos, AquiferSampler.FluidLevel fl) {
		return "(" + BlockPos.unpackLongX(pos) + "," + BlockPos.unpackLongY(pos) + "," + BlockPos.unpackLongZ(pos)
				+ "|" + wgFluidY(fl) + "," + wgBlockId(fl) + ")";
	}

	/** 首链捕获器移至 wg.bench.AquiferDumpProbe.WgCap（mixin 包禁止非 mixin 类，IllegalClassLoadError 修复）。 */
}
