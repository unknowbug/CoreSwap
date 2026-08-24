// worldgen_api.h — CoreSwap worldgen 纯 C 接口（JNI 无关，便于任何语言桥接）
#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// 创建 worldgen 句柄：一次 seed 初始化（构建全部 noise samplers + density 树）
// worldgenDir: vanilla worldgen JSON 数据目录（含 data/minecraft/worldgen/...）
// 失败返回 NULL
void* wg_create(int64_t seed, const char* worldgenDir, const char* settingsName = nullptr,
                const char* biomeParamsFile = nullptr, int worldHeight = 0);
double wg_sample_density(void* handle, int x, int y, int z);
double wg_sample_named(void* handle, const char* name, int x, int y, int z);
double wg_sample_noise(void* handle, const char* name, double x, double y, double z);

// 诊断/探针：直接采样第 which 个 SplineDF（绕过 min/interpolated/blend 等 wrapper 链）→ 隔离 ② wrapper 链贡献。
// which 越界 → 返回 0.0。返回已构建的 SplineDF 总数。
int wg_spline_count(void* handle);
int wg_spline_nodes(void* handle, int which);
double wg_sample_spline(void* handle, int which, int x, int y, int z);

// M3 interp-only 探针：直接采样 finalDensity 顶层 InterpolatedDF#1（预建 grid 后只测 grid 命中）。
double wg_sample_interp(void* handle, int x, int y, int z);

// 释放句柄
void wg_destroy(void* handle);

// 密度场批量求值：region = size×size chunks（minChunkX..minChunkX+size-1）
// out: 调用方分配，至少 size*size*pointsPerChunk 个 double
// 布局：chunk-major（cz 外循环 → cx 内循环），chunk 内 y→z→x
// 密度网格：每 chunk sx×sy×sz = (16/xzInterval)×(height/yInterval)×(16/xzInterval)
// 返回每个 chunk 的点数（pointsPerChunk），失败返回 0
int wg_fill_density(void* handle, int minChunkX, int minChunkZ, int size,
                    double* out);

// 密度网格参数（与 density 文件 header 一致）
int wg_density_xz_interval(void* handle);
int wg_density_y_interval(void* handle);
int wg_min_y(void* handle);
int wg_height(void* handle);
int wg_density_points_per_chunk(void* handle);

// 完整区块生成：density → aquifer → surface rules
// out: int32_t[16*16*384]（vanilla block raw id）
// 返回写入的方块数（16*16*384），失败返回 0
int wg_fill_blocks(void* handle, int chunkX, int chunkZ, int32_t* out);

// 多 chunk 并行生成（结果与串行逐位一致）：count 个 (chunkX, chunkZ, out) 三元组。
// threads: 并行线程数；0 或负 = 自适应 min(CPU 逻辑线程数, count)（探测失败兜底 1）。
// 返回 count。
int wg_fill_blocks_multi(void* handle, const int* chunkXs, const int* chunkZs,
                         int32_t* const* outs, int count, int threads);

// 两阶段 FEATURE（跨 chunk 球体，Java 语义）：
//   phase 1：surface+carvers 全部生成并存 regionCols（阶段 1，可并行）
//   phase 2：features 重跑（跨 chunk 写 regionCols 邻域；顺序敏感，内部强制串行）
// 需先调用 phase 1 再 phase 2（同一 handle）。
int wg_fill_blocks_multi_phase(void* handle, const int* chunkXs, const int* chunkZs,
                               int32_t* const* outs, int count, int threads, int phase);

// 设置指定 chunk 的 Beardifier（StructureWeightSampler 结构密度修正）输入。
// 数据为 vanilla createStructureWeightSampler 产出的 piece/junction 列表：
//   pieces:   每 8 个 int = {minX, minY, minZ, maxX, maxY, maxZ, terrain, groundLevelDelta}
//             terrain 序数：0=NONE 1=BURY 2=BEARD_THIN 3=BEARD_BOX
//   junctions:每 3 个 int = {sourceX, sourceGroundY, sourceZ}
// 未设置 = 无结构（Beardifier=0，行为与现状一致）。线程安全：调用须在 fill 之前。
void wg_set_beardifier(void* handle, int chunkX, int chunkZ,
                       const int* pieces, int pieceCount,
                       const int* junctions, int junctionCount);
// 清空全部 chunk 的 Beardifier 输入（可选，destroy 前调用）
void wg_clear_beardifier(void* handle);

// 剖析统计输出（WG_PROFILE=1 时启用，运行结束后调用打印到 stderr；无 Profile 时为空操作）
void wg_profile_dump(void);
void wg_sample_biome(void* handle, int x, int y, int z, char* out, int outLen);
double wg_router_sample(void* handle, const char* name, int x, int y, int z);
#ifdef __cplusplus
}
#endif
