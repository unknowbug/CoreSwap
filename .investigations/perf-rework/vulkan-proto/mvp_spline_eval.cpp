// mvp_spline_eval.cpp — DFC C++ 移植 MVP：验证 spline_eval 显式栈算法正确 + 性能
// 路径 B（独立对拍，先证明 DFC 直排假设值得投入）
// 用 CpuBackend (vulkan-proto/cpu_backend.h) 的 spline 表数据（vanilla spline）。
// 对比：递归 spline（虚调用/递归，= production SplineDF 形态）vs 显式栈 spline_eval（= DFC GLSL 形态）。
// 核心验证：① 两算法结果一致（DFC 显式栈正确）② 显式栈性能（并发/串行）。
// 注：spline_coord 用简化版（先验算法，不依赖完整 normal_noise/split 链；spline_coord 真实性后续接）。
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <cmath>
#include <vector>
#include <chrono>
#include <thread>
#include <atomic>
#include <numeric>
#include <algorithm>

// ---- 内联 CpuBackend spline 表（vanilla spline，dfc_gen 导出）----
// nodePack 每 5 个一组 {coordType, n, locBegin, derBegin, valBegin}
static const int SPLINE_NODES = 56;
static const int NP[280] = {
  2,2,17,17,0,2,2,19,19,2,2,6,21,21,4,2,5,27,27,10,2,5,32,32,15,2,5,37,37,20,2,5,42,42,25,1,7,10,10,30,2,5,54,54,37,2,5,59,59,42,2,5,64,64,47,2,5,69,69,52,1,7,47,47,57,2,3,85,85,64,2,3,88,88,67,2,3,91,91,70,2,5,94,94,73,2,5,99,99,78,2,3,104,104,83,1,11,74,74,86,2,3,118,118,97,2,3,121,121,100,2,5,124,124,103,2,5,129,129,108,2,5,134,134,113,2,3,139,139,118,2,5,142,142,121,1,11,107,107,126,0,10,0,0,137,3,2,157,157,147,2,3,154,154,149,3,2,162,162,152,2,3,159,159,154,1,4,150,150,157,2,3,168,168,161,1,4,164,164,164,0,3,147,147,168,3,2,186,186,171,3,2,188,188,173,3,2,190,190,175,3,2,194,194,177,2,2,192,192,179,1,10,176,176,181,3,2,206,206,191,3,2,210,210,193,2,2,208,208,195,1,10,196,196,197,3,2,222,222,207,3,2,226,226,209,2,2,224,224,211,1,10,212,212,213,3,2,239,239,223,2,2,241,241,225,2,2,243,243,227,1,11,228,228,229,0,5,171,171,240
};
static const int SPLINE_VALNODE[245] = {
  -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 8, 9, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10, -1, 13, 14, 15, 16, 17, 10, 10, 18, 18, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 24, -1, -1, -1, -1, -1, -1, 20, 21, 21, 22, 23, 24, 24, 25, 25, 24, 26, -1, -1, -1, -1, -1, 7, 7, 12, 19, 27, -1, -1, -1, -1, 29, -1, -1, -1, -1, 31, 30, 32, 32, -1, -1, 29, 29, 34, 30, 30, -1, -1, 33, 35, -1, -1, -1, -1, -1, -1, -1, -1, -1, 40, 37, 38, 37, 37, 39, 37, -1, 41, 41, -1, -1, -1, -1, -1, -1, 44, 43, 38, 43, 43, 39, 43, -1, 45, 45, -1, -1, -1, -1, -1, -1, 48, 47, 38, 47, 47, 39, 47, -1, 49, 49, -1, -1, -1, 51, -1, 51, -1, 51, 38, 51, 51, 39, 51, 52, 52, 53, 53, -1, -1, 42, 46, 50, 54
};

static const float SPLINE_LOCS[245] = {
  -1.1000000000000001f, -1.02f, -0.51000000000000001f, -0.44f, -0.17999999999999999f, -0.16f, -0.14999999999999999f, -0.10000000000000001f, 0.25f, 1.0f, -0.84999999999999998f, -0.69999999999999996f,
  -0.40000000000000002f, -0.34999999999999998f, -0.10000000000000001f, 0.20000000000000001f, 0.69999999999999996f, -1.0f, 1.0f, -1.0f, 1.0f, -1.0f, -0.75f, -0.65000000000000002f,
  0.5954547f, 0.60545470000000001f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f,
  1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -0.84999999999999998f,
  -0.69999999999999996f, -0.40000000000000002f, -0.34999999999999998f, -0.10000000000000001f, 0.20000000000000001f, 0.69999999999999996f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f,
  -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f,
  0.40000000000000002f, 1.0f, -0.84999999999999998f, -0.69999999999999996f, -0.40000000000000002f, -0.34999999999999998f, -0.10000000000000001f, 0.20000000000000001f, 0.40000000000000002f, 0.45000000000000001f, 0.55000000000000004f, 0.57999999999999996f,
  0.69999999999999996f, -1.0f, 0.0f, 1.0f, -1.0f, 0.0f, 1.0f, -1.0f, 0.0f, 1.0f, -1.0f, -0.40000000000000002f,
  0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, -0.84999999999999998f,
  -0.69999999999999996f, -0.40000000000000002f, -0.34999999999999998f, -0.10000000000000001f, 0.20000000000000001f, 0.40000000000000002f, 0.45000000000000001f, 0.55000000000000004f, 0.57999999999999996f, 0.69999999999999996f, -1.0f, 0.0f,
  1.0f, -1.0f, 0.0f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f,
  0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, -1.0f, -0.40000000000000002f,
  0.0f, 0.40000000000000002f, 1.0f, -0.11f, 0.029999999999999999f, 0.65000000000000002f, -1.0f, -0.78000000000000003f, -0.57750000000000001f, -0.375f, 0.19999998999999999f, 0.44999995999999998f,
  1.0f, -0.01f, 0.01f, 0.19999998999999999f, 0.44999995999999998f, 1.0f, -0.01f, 0.01f, -1.0f, -0.78000000000000003f, -0.57750000000000001f, -0.375f,
  0.19999998999999999f, 0.44999995999999998f, 1.0f, -0.19f, -0.14999999999999999f, -0.10000000000000001f, 0.029999999999999999f, 0.059999999999999998f, -0.59999999999999998f, -0.5f, -0.34999999999999998f, -0.25f,
  -0.10000000000000001f, 0.029999999999999999f, 0.34999999999999998f, 0.45000000000000001f, 0.55000000000000004f, 0.62f, -0.20000000000000001f, 0.20000000000000001f, -0.050000000000000003f, 0.050000000000000003f, -0.050000000000000003f, 0.050000000000000003f,
  -0.90000000000000002f, -0.68999999999999995f, 0.0f, 0.10000000000000001f, -0.59999999999999998f, -0.5f, -0.34999999999999998f, -0.25f, -0.10000000000000001f, 0.029999999999999999f, 0.34999999999999998f, 0.45000000000000001f,
  0.55000000000000004f, 0.62f, -0.20000000000000001f, 0.20000000000000001f, -0.90000000000000002f, -0.68999999999999995f, 0.0f, 0.10000000000000001f, -0.59999999999999998f, -0.5f, -0.34999999999999998f, -0.25f,
  -0.10000000000000001f, 0.029999999999999999f, 0.34999999999999998f, 0.45000000000000001f, 0.55000000000000004f, 0.62f, -0.20000000000000001f, 0.20000000000000001f, -0.90000000000000002f, -0.68999999999999995f, 0.0f, 0.10000000000000001f,
  -0.59999999999999998f, -0.5f, -0.34999999999999998f, -0.25f, -0.10000000000000001f, 0.029999999999999999f, 0.050000000000000003f, 0.40000000000000002f, 0.45000000000000001f, 0.55000000000000004f, 0.57999999999999996f, -0.20000000000000001f,
  0.20000000000000001f, 0.45000000000000001f, 0.69999999999999996f, -0.69999999999999996f, -0.14999999999999999f
};

static const float SPLINE_DERS[245] = {
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.38940096000000002f, 0.38940096000000002f, 0.37788021999999999f, 0.37788021999999999f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.25345630000000002f, 0.25345630000000002f, 0.5f, 0.0f, 0.0f, 0.0f, 0.0070000009999999996f, 0.5f, 0.0f, 0.0f, 0.10000000000000001f,
  0.0070000009999999996f, 0.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.059999999999999998f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.5f, 0.0f, 0.0f, 0.0f, 0.0070000009999999996f, 0.5f,
  0.01f, 0.01f, 0.094000003999999998f, 0.0070000009999999996f, 0.5f, 0.0f, 0.0f, 0.040000000000000001f, 0.049000000000000002f, 0.0f, 0.0f, 0.0f,
  0.12f, 0.049000000000000002f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.51382490000000003f, 0.51382490000000003f, 0.0f, 0.43317973999999998f, 0.43317973999999998f, 0.0f, 0.39170509999999997f, 0.39170509999999997f, 0.5f, 0.0f,
  0.0f, 0.0f, 0.049000014000000001f, 0.5f, 0.070000000000000007f, 0.070000000000000007f, 0.65800000000000003f, 0.049000014000000001f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.57603689999999996f,
  0.57603689999999996f, 0.0f, 0.4608295f, 0.4608295f, 0.5f, 0.0f, 0.0f, 0.0f, 0.070000014999999999f, 0.5f, 0.099999993999999995f, 0.099999993999999995f,
  0.93999999999999995f, 0.070000014999999999f, 0.5f, 0.0f, 0.0f, 0.040000000000000001f, 0.049000000000000002f, 0.0f, 0.0f, 0.0f, 0.014999999999999999f, 0.0f,
  0.0f, 0.040000000000000001f, 0.049000000000000002f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f
};

static const float SPLINE_VALF[245] = {
  -0.088801859999999996f, 0.69000006000000003f, -0.11576035599999999f, 0.64000009999999996f, -0.22220000000000001f, -0.22220000000000001f, 0.0f, 2.9802322000000001e-08f, 2.9802322000000001e-08f, 0.10000002400000001f, -0.29999999999999999f, 0.050000000000000003f,
  0.050000000000000003f, 0.050000000000000003f, 0.060000001999999997f, -0.14999999999999999f, 0.0f, 0.0f, 0.050000000000000003f, 0.060000001999999997f, -0.14999999999999999f, 0.0f, 0.0f, 0.0f,
  0.0f, -0.02f, -0.029999999999999999f, -0.029999999999999999f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, -0.25f, 0.050000000000000003f, 0.050000000000000003f, 0.050000000000000003f, 0.060000001999999997f, -0.10000000000000001f, 0.001f, 0.0030000000000000001f, 0.050000000000000003f, 0.060000001999999997f, -0.10000000000000001f,
  0.01f, 0.01f, 0.029999999999999999f, 0.10000000000000001f, -0.02f, -0.029999999999999999f, -0.029999999999999999f, 0.029999999999999999f, 0.10000000000000001f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.20235021f, 0.71617509999999995f, 1.23f, 0.20000000000000001f, 0.44682026000000002f, 0.88f, 0.20000000000000001f, 0.30829495000000001f,
  0.70000004999999998f, -0.25f, 0.34999999999999998f, 0.34999999999999998f, 0.34999999999999998f, 0.42000001999999997f, -0.10000000000000001f, 0.0069999998000000001f, 0.021000000000000001f, 0.34999999999999998f, 0.42000001999999997f, -0.10000000000000001f,
  0.0f, 0.17000000000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.34792625999999999f, 0.92396310000000004f, 1.5f, 0.20000000000000001f, 0.5391705f, 1.0f, -0.20000000000000001f, 0.5f, 0.5f, 0.5f, 0.59999999999999998f,
  -0.050000000000000003f, 0.01f, 0.029999999999999999f, 0.5f, 0.59999999999999998f, -0.050000000000000003f, 0.01f, 0.01f, 0.029999999999999999f, 0.10000000000000001f, -0.050000000000000003f, 0.0f,
  0.17000000000000001f, -0.02f, 0.01f, 0.01f, 0.029999999999999999f, 0.10000000000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.043999999999999997f, -0.22220000000000001f, -0.22220000000000001f, -0.12f, -0.12f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.63f, 0.29999999999999999f, 0.0f, 0.0f, 0.0f, 0.315f, 0.14999999999999999f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 6.2999999999999998f, 6.25f, 6.2999999999999998f, 2.6699999999999999f, 2.6699999999999999f, 6.2999999999999998f, 6.25f, 0.625f, 6.25f,
  0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 6.25f, 0.0f, 0.0f, 6.25f, 6.2999999999999998f,
  5.4699999999999998f, 5.4699999999999998f, 0.625f, 5.4699999999999998f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 5.4699999999999998f,
  0.0f, 0.0f, 5.4699999999999998f, 6.2999999999999998f, 5.0800000000000001f, 5.0800000000000001f, 0.625f, 5.0800000000000001f, 0.0f, 0.0f, 0.0f, 0.0f,
  0.0f, 0.0f, 0.0f, 5.0800000000000001f, 0.0f, 0.0f, 5.0800000000000001f, 6.2999999999999998f, 4.6900000000000004f, 0.0f, 1.5600000000000001f, 0.0f,
  1.3700000000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 4.6900000000000004f,
  3.9500000000000002f, 0.0f, 0.0f, 0.0f, 0.0f
};

static const int SPLINE_VALKIND[245] = {
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1,
  1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
  0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0,
  0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1,
  0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
  1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0,
  1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
  0, 1, 1, 1, 1
};

// ---- 简化的 spline_coord（MVP 第 1 步：先验算法，coord 用模拟值；完整接 normal_noise 待第 2 步）----
// 真实 coordType 分派在 dfc_gen（coord_glsl），此处每个 coordType 给一个确定性 float（模拟不同节点坐标）
static float spline_coord(int coordType, int corner, int sIdx, int ix, int iy, int iz) {
    // MVP：coord 用确定性伪值（coordType-based），仅验证显式栈 vs 递归的结构正确性。
    // 真实 coord（normal_noise + split 链）第 2 步接入。
    return 0.1f * (float)(coordType + 1) + 0.001f * (float)(iy + iz);  // 大致范围（与 locs 匹配）
}

// ---- vanilla MathHelper.binarySearch 精确复刻 ----
static int spline_find_range(float x, int locBegin, int n, const float* locs) {
    int min = 0;
    int i = n;
    while (i > 0) {
        int j = i / 2;
        int k = min + j;
        if (x < locs[locBegin + k]) { i = j; }
        else { min = k + 1; i -= j + 1; }
    }
    return min - 1;
}

static float spline_hermite(float coord, float lo, float span, float nv, float ov, float d0, float d1) {
    float kd = (coord - lo) / span;
    float p = d0 * span - (ov - nv);
    float q = -d1 * span + (ov - nv);
    return (nv + kd * (ov - nv)) + kd * (1.0f - kd) * (p + kd * (q - p));
}

// ---- 递归 spline（= production SplineDF 形态，虚调用/递归；权威参照）----
static float spline_recursive(int nodeId, int corner, int sIdx, int ix, int iy, int iz) {
    int p = nodeId * 5;
    int ct = NP[p + 0];
    int n = NP[p + 1];
    int locB = NP[p + 2];
    int derB = NP[p + 3];
    int valB = NP[p + 4];
    float coord = spline_coord(ct, corner, sIdx, ix, iy, iz);
    int i = spline_find_range(coord, locB, n, SPLINE_LOCS);
    if (i < 0) {
        if (SPLINE_VALKIND[valB] == 0)
            return SPLINE_VALF[valB] + SPLINE_DERS[derB] * (coord - SPLINE_LOCS[locB]);
        return spline_recursive(SPLINE_VALNODE[valB], corner, sIdx, ix, iy, iz)
             + SPLINE_DERS[derB] * (coord - SPLINE_LOCS[locB]);
    } else if (i >= n - 1) {
        if (SPLINE_VALKIND[valB + n - 1] == 0)
            return SPLINE_VALF[valB + n - 1] + SPLINE_DERS[derB + n - 1] * (coord - SPLINE_LOCS[locB + n - 1]);
        return spline_recursive(SPLINE_VALNODE[valB + n - 1], corner, sIdx, ix, iy, iz)
             + SPLINE_DERS[derB + n - 1] * (coord - SPLINE_LOCS[locB + n - 1]);
    } else {
        int k = i;
        float g = SPLINE_LOCS[locB + k], h = SPLINE_LOCS[locB + k + 1];
        float nv, ov;
        if (SPLINE_VALKIND[valB + k] == 0) nv = SPLINE_VALF[valB + k];
        else nv = spline_recursive(SPLINE_VALNODE[valB + k], corner, sIdx, ix, iy, iz);
        if (SPLINE_VALKIND[valB + k + 1] == 0) ov = SPLINE_VALF[valB + k + 1];
        else ov = spline_recursive(SPLINE_VALNODE[valB + k + 1], corner, sIdx, ix, iy, iz);
        return spline_hermite(coord, g, h - g, nv, ov, SPLINE_DERS[derB + k], SPLINE_DERS[derB + k + 1]);
    }
}

// ---- 显式栈 spline_eval（= DFC GLSL 形态；算法被验证）----
static float spline_eval(int rootNode, int corner, int sIdx, int ix, int iy, int iz) {
    int st_node[64]; int st_i[64]; int st_stage[64];
    float st_coord[64]; float st_v0[64]; float st_v1[64];
    int sp = 0;
    st_node[0] = rootNode; st_stage[0] = 0; sp = 1;
    float outVal = 0.0f;
    while (sp > 0) {
        int f = sp - 1;
        int node = st_node[f];
        int p = node * 5;
        int ct = NP[p + 0];
        int n = NP[p + 1];
        int locB = NP[p + 2];
        int derB = NP[p + 3];
        int valB = NP[p + 4];
        if (st_stage[f] == 0) {
            float coord = spline_coord(ct, corner, sIdx, ix, iy, iz);
            int i = spline_find_range(coord, locB, n, SPLINE_LOCS);
            st_coord[f] = coord; st_i[f] = i;
            if (i < 0) {
                if (SPLINE_VALKIND[valB] == 0) { outVal = SPLINE_VALF[valB] + SPLINE_DERS[derB] * (coord - SPLINE_LOCS[locB]); sp--; }
                else { st_stage[f] = 4; st_node[sp] = SPLINE_VALNODE[valB]; st_stage[sp] = 0; sp++; }
            } else if (i >= n - 1) {
                if (SPLINE_VALKIND[valB + n - 1] == 0) { outVal = SPLINE_VALF[valB + n - 1] + SPLINE_DERS[derB + n - 1] * (coord - SPLINE_LOCS[locB + n - 1]); sp--; }
                else { st_stage[f] = 5; st_node[sp] = SPLINE_VALNODE[valB + n - 1]; st_stage[sp] = 0; sp++; }
            } else {
                st_stage[f] = 1;
                if (SPLINE_VALKIND[valB + i] == 0) {
                    st_v0[f] = SPLINE_VALF[valB + i];
                    st_stage[f] = 2;
                    if (SPLINE_VALKIND[valB + i + 1] == 0) {
                        st_v1[f] = SPLINE_VALF[valB + i + 1];
                        float lo = SPLINE_LOCS[locB + i];
                        outVal = spline_hermite(coord, lo, SPLINE_LOCS[locB + i + 1] - lo, st_v0[f], st_v1[f], SPLINE_DERS[derB + i], SPLINE_DERS[derB + i + 1]);
                        sp--;
                    } else { st_stage[f] = 3; st_node[sp] = SPLINE_VALNODE[valB + i + 1]; st_stage[sp] = 0; sp++; }
                } else { st_node[sp] = SPLINE_VALNODE[valB + i]; st_stage[sp] = 0; sp++; }
            }
        } else if (st_stage[f] == 4) {
            float coord = st_coord[f];
            outVal += SPLINE_DERS[derB] * (coord - SPLINE_LOCS[locB]);
            sp--;
        } else if (st_stage[f] == 5) {
            float coord = st_coord[f];
            outVal += SPLINE_DERS[derB + n - 1] * (coord - SPLINE_LOCS[locB + n - 1]);
            sp--;
        } else if (st_stage[f] == 1) {
            st_v0[f] = outVal;
            st_stage[f] = 2;
            int i = st_i[f];
            if (SPLINE_VALKIND[valB + i + 1] == 0) {
                st_v1[f] = SPLINE_VALF[valB + i + 1];
                float lo = SPLINE_LOCS[locB + i];
                outVal = spline_hermite(st_coord[f], lo, SPLINE_LOCS[locB + i + 1] - lo, st_v0[f], st_v1[f], SPLINE_DERS[derB + i], SPLINE_DERS[derB + i + 1]);
                sp--;
            } else { st_stage[f] = 3; st_node[sp] = SPLINE_VALNODE[valB + i + 1]; st_stage[sp] = 0; sp++; }
        } else if (st_stage[f] == 2) {
            st_v1[f] = outVal;
            int i = st_i[f];
            float lo = SPLINE_LOCS[locB + i];
            outVal = spline_hermite(st_coord[f], lo, SPLINE_LOCS[locB + i + 1] - lo, st_v0[f], st_v1[f], SPLINE_DERS[derB + i], SPLINE_DERS[derB + i + 1]);
            sp--;
        } else if (st_stage[f] == 3) {
            float v1 = outVal;
            int i = st_i[f];
            float lo = SPLINE_LOCS[locB + i];
            outVal = spline_hermite(st_coord[f], lo, SPLINE_LOCS[locB + i + 1] - lo, st_v0[f], v1, SPLINE_DERS[derB + i], SPLINE_DERS[derB + i + 1]);
            sp--;
        }
    }
    return outVal;
}

// ===== MVP 第 2 步：虚调用形态（模拟 production SplineDF 的 locationFunctions[locFn]->sample 虚调用 + shared_ptr）=====
// production: locationFunctions 是 std::vector<DF>（shared_ptr<DensityFunction>），每节点 locationFunctions[nd.locFn]->sample(pos)
// = 虚函数调用（vtable 跳转）+ shared_ptr 解引用。这是 density 11× 的真实开销来源。
// 关键：production 的 locationFunctions 是【多形子类型】（normal_noise / spline_coord / const / wrapping 等），
// 虚调用目标随时间变化 → indirect branch predictor 无法稳定预测 + BTB/I-cache miss。单形虚调用会低估成本。
// 因此每个 coordType 映射到一个【不同】子类实例（不同虚函数实现体），模拟真实多形态。
// 数值一致性：每个子类的 sample 返回值必须 == spline_coord(ct=本实例的 ct)，保证第 2 步对拍 maxDiff 仍为 0。
struct BaseDF {
    virtual ~BaseDF() = default;
    virtual float sample(float x, float y, float z) const { return 0.0f; }
};
// 4 种实际子类（模拟 production 的 4 类 locationFunction），每种函数体不同（多形态虚调用目标），
// 但都构造绑定 ct，sample 用不同计算路径得到相同结果 0.1f*(ct+1)+0.001f*(y+z)。
struct LocFnNoise : BaseDF {
    int ct; explicit LocFnNoise(int c):ct(c){}
    float sample(float x, float y, float z) const override { return 0.1f*(float)(ct+1) + 0.001f*(float)(y+z); }
};
struct LocFnSpline : BaseDF {
    int ct; explicit LocFnSpline(int c):ct(c){}
    float sample(float x, float y, float z) const override { return 0.1f + 0.1f*(float)ct + 0.001f*(float)y + 0.001f*(float)z; }
};
struct LocFnConst : BaseDF {
    int ct; explicit LocFnConst(int c):ct(c){}
    float sample(float x, float y, float z) const override { float a=0.1f*(float)(ct+1); float b=0.001f*(float)(y+z); return a+b; }
};
struct LocFnWrap : BaseDF {
    int ct; explicit LocFnWrap(int c):ct(c){}
    float sample(float x, float y, float z) const override { return (0.1f + 0.1f*(float)ct) + 0.001f*(float)(y+z); }
};
// locationFunctions 池：CT_TO_LOCFN[ct&7] -> pool 索引，pool 内交错注册多形态子类
static const int CT_TO_LOCFN[8] = {0,1,2,3,4,5,6,7};  // coordType(0-3) → locationFunction 索引（MVP 简化）
static std::vector<std::shared_ptr<BaseDF>> g_locFnPool;
static std::vector<std::shared_ptr<BaseDF>> buildLocFnPool() {
    std::vector<std::shared_ptr<BaseDF>> pool;
    // 交错注册多形子类型：0,4=Noise；1,5=Spline；2,6=Const；3,7=Wrap（真实多形态）
    pool.push_back(std::make_shared<LocFnNoise>(0));   // ct=0 -> Noise
    pool.push_back(std::make_shared<LocFnSpline>(1));  // ct=1 -> Spline
    pool.push_back(std::make_shared<LocFnConst>(2));   // ct=2 -> Const
    pool.push_back(std::make_shared<LocFnWrap>(3));    // ct=3 -> Wrap
    pool.push_back(std::make_shared<LocFnNoise>(4));
    pool.push_back(std::make_shared<LocFnSpline>(5));
    pool.push_back(std::make_shared<LocFnConst>(6));
    pool.push_back(std::make_shared<LocFnWrap>(7));
    return pool;
}
// 虚调用版递归（= production SplineDF 形态：每节点 locationFunctions[locFn]->sample 虚调用）
static float spline_recursive_virtual(int nodeId, int corner, int sIdx, int ix, int iy, int iz) {
    int p = nodeId * 5;
    int ct = NP[p + 0];
    int n = NP[p + 1];
    int locB = NP[p + 2];
    int derB = NP[p + 3];
    int valB = NP[p + 4];
    // production: locationFunctions[locFn]->sample(pos) 虚调用 + shared_ptr 解引用
    float coord = g_locFnPool[CT_TO_LOCFN[ct & 7]]->sample((float)ix, (float)iy, (float)iz);
    int i = spline_find_range(coord, locB, n, SPLINE_LOCS);
    if (i < 0) {
        if (SPLINE_VALKIND[valB] == 0)
            return SPLINE_VALF[valB] + SPLINE_DERS[derB] * (coord - SPLINE_LOCS[locB]);
        return spline_recursive_virtual(SPLINE_VALNODE[valB], corner, sIdx, ix, iy, iz)
             + SPLINE_DERS[derB] * (coord - SPLINE_LOCS[locB]);
    } else if (i >= n - 1) {
        if (SPLINE_VALKIND[valB + n - 1] == 0)
            return SPLINE_VALF[valB + n - 1] + SPLINE_DERS[derB + n - 1] * (coord - SPLINE_LOCS[locB + n - 1]);
        return spline_recursive_virtual(SPLINE_VALNODE[valB + n - 1], corner, sIdx, ix, iy, iz)
             + SPLINE_DERS[derB + n - 1] * (coord - SPLINE_LOCS[locB + n - 1]);
    } else {
        int k = i;
        float g = SPLINE_LOCS[locB + k], h = SPLINE_LOCS[locB + k + 1];
        float nv, ov;
        if (SPLINE_VALKIND[valB + k] == 0) nv = SPLINE_VALF[valB + k];
        else nv = spline_recursive_virtual(SPLINE_VALNODE[valB + k], corner, sIdx, ix, iy, iz);
        if (SPLINE_VALKIND[valB + k + 1] == 0) ov = SPLINE_VALF[valB + k + 1];
        else ov = spline_recursive_virtual(SPLINE_VALNODE[valB + k + 1], corner, sIdx, ix, iy, iz);
        return spline_hermite(coord, g, h - g, nv, ov, SPLINE_DERS[derB + k], SPLINE_DERS[derB + k + 1]);
    }
}

int main() {
    g_locFnPool = buildLocFnPool();  // MVP 第 2 步：初始化 locationFunctions 池
    // 对拍：显式栈 vs 递归（multi 坐标）
    double maxDiff = 0.0; int maxN = 0;
    for (int iy = 0; iy < 16; iy++) for (int iz = 0; iz < 16; iz++) {
        // 用 node 0（首 spline）采样
        float a = spline_eval(0, 0, 0, 8, iy, iz);
        float b = spline_recursive(0, 0, 0, 8, iy, iz);
        double d = std::fabs((double)a - b);
        if (d > maxDiff) { maxDiff = d; maxN = iy*16+iz; }
    }
    std::printf("[MVP] explicit-stack vs recursive: maxDiff=%.3e @n=%d\n", maxDiff, maxN);

    // MVP 第 2 步：虚调用版正确性（vs 显式栈）
    double maxDiffV = 0.0;
    for (int iy = 0; iy < 16; iy++) for (int iz = 0; iz < 16; iz++) {
        float a = spline_eval(0, 0, 0, 8, iy, iz);
        float b = spline_recursive_virtual(0, 0, 0, 8, iy, iz);
        double d = std::fabs((double)a - b);
        if (d > maxDiffV) maxDiffV = d;
    }
    std::printf("[MVP-step2] virtual-call recursive (production-form) vs explicit-stack: maxDiff=%.3e\n", maxDiffV);

    // 性能：const-arr 递归 vs 显式栈 vs 虚调用递归（串行）——关键：虚调用 vs 显式栈
    const int N = 200000;
    auto t0 = std::chrono::steady_clock::now();
    double acc = 0; for (int i = 0; i < N; i++) acc += spline_recursive(i % SPLINE_NODES, 0, 0, 8, i % 16, i % 16);
    auto t1 = std::chrono::steady_clock::now();
    double acc2 = 0; for (int i = 0; i < N; i++) acc2 += spline_eval(i % SPLINE_NODES, 0, 0, 8, i % 16, i % 16);
    auto t2 = std::chrono::steady_clock::now();
    double acc3 = 0; for (int i = 0; i < N; i++) acc3 += spline_recursive_virtual(i % SPLINE_NODES, 0, 0, 8, i % 16, i % 16);
    auto t3 = std::chrono::steady_clock::now();
    double msRec = std::chrono::duration<double,std::milli>(t1-t0).count();
    double msStack = std::chrono::duration<double,std::milli>(t2-t1).count();
    double msVirt = std::chrono::duration<double,std::milli>(t3-t2).count();
    std::printf("[MVP-step2] N=%d const-recursive=%.2fms  explicit-stack=%.2fms  virtual-call-recursive=%.2fms  acc=%.4f acc2=%.4f acc3=%.4f\n",
                N, msRec, msStack, msVirt, acc, acc2, acc3);

    // ===== MVP 第 3 步（决定性）：线程扫描 + 每样本成本 =====
    // 回答两个问题：
    //  ① spline 采样是否随并发线性变慢（T=1→8），复现生产「单样本 15.8→190μs 12×」
    //  ② DFC 显式栈是否免疫——若显式栈也随线程放大 → 共享数据/内存延迟（DFC 也免不了）；
    //    若显式栈不放大而虚调用放大 → 指针追逐/虚调用（DFC 有效）。
    // 测量纪律：每线程固定采样数（计算主导）；各形态各线程数多轮取 min；计算 ns/sample。
    const int SCAN_T[] = {1, 2, 4, 8};
    const int NPER = 100000;                // 每线程采样数
    const int NR = 5;                        // 每组合的轮数（取 min）
    // three 形态：0=const递归(无虚调用) 1=显式栈(DFC) 2=虚调用递归(production)
    auto runOne = [&](int useForm, int threads) -> double {
        std::vector<double> results(threads, 0.0);
        auto start = std::chrono::steady_clock::now();
        std::vector<std::thread> ts;
        for (int t = 0; t < threads; t++) {
            ts.emplace_back([&, t] {
                double a = 0;
                for (int i = 0; i < NPER; i++) {
                    int node = ((t * NPER + i) % SPLINE_NODES);
                    if (useForm == 0) a += spline_recursive(node, 0, 0, 8, i % 16, i % 16);
                    else if (useForm == 1) a += spline_eval(node, 0, 0, 8, i % 16, i % 16);
                    else a += spline_recursive_virtual(node, 0, 0, 8, i % 16, i % 16);
                }
                results[t] = a;   // 每线程独立槽位
            });
        }
        for (auto& th : ts) th.join();
        double ms = std::chrono::duration<double,std::milli>(std::chrono::steady_clock::now()-start).count();
        return ms;
    };
    auto bestNs = [&](int form, int threads) -> double {   // 每样本 ns（min 轮）
        double best = 1e18;
        for (int r = 0; r < NR; r++) {
            double ms = runOne(form, threads);
            double ns = ms * 1e6 / ((double)threads * NPER);
            if (ns < best) best = ns;
        }
        return best;
    };
    std::printf("[MVP-step3-DECISIVE] per-sample ns  (cond: %d/thread, min of %d rounds)\n", NPER, NR);
    std::printf("  %-10s %10s %10s %10s\n", "threads", "constRec", "explicit", "virtual");
    for (int T : SCAN_T) {
        double c = bestNs(0, T);
        double e = bestNs(1, T);
        double v = bestNs(2, T);
        std::printf("  %-10d %10.1f %10.1f %10.1f   (virtual/const=%.1fx  explicit vs virtual=%.1fx)\n",
                    T, c, e, v, v/c, e/v);
    }
    // 1 基线 vs 8 的放大（关键判据）
    double v1 = bestNs(2, 1), v8 = bestNs(2, 8);
    double e1 = bestNs(1, 1), e8 = bestNs(1, 8);
    double c1 = bestNs(0, 1), c8 = bestNs(0, 8);
    std::printf("[MVP-step3-DECISIVE] concurrency amplification (T=8 vs T=1): "
                "constRec=%.1fx  explicit(DFC)=%.1fx  virtual=%.1fx\n", c8/c1, e8/e1, v8/v1);
    std::printf("[MVP-step3-DECISIVE] 若 explicit 放大≈virtual 放大 → 共享数据/内存延迟(DFC免不了免疫)\n");
    std::printf("[MVP-step3-DECISIVE] 若 explicit 放大<<virtual 放大 → 虚调用指针追逐(DFC有效)\n");
    return 0;
}
