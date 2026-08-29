// 自动生成（DFC CPU 后端），勿手改
#pragma once
#include <vector>
#include <map>
#include <string>
#include <cmath>
#include <cstdint>
#include <algorithm>
#include "noise.h"
#include "xoroshiro.h"
#include "density.h"

struct CpuBackend {
    std::map<std::string, wg::DoublePerlinNoiseSampler> shiftNoises;
    std::vector<wg::DoublePerlinNoiseSampler> normals;
    std::vector<int> n, octBase, splitBase;
    std::vector<std::shared_ptr<wg::InterpolatedNoiseDF>> oldBlendeds;
    std::vector<int> oldBase, oldSplitBase;
    int splitTotal = 8672;
    int permSize = 356352;
    int perSample = 352;   // D19: valBuf 每采样点槽数（与 shader PER_SAMPLE 一致）
    // A1b：spline SSBO 数据（生成器导出，宿主上传——D19 铁律）
    int splineBindBase = 6;   // P2-2: spline 6 表 binding 起始号（6-11）
    int splineNodes = 56;
    std::vector<int> splineNodePack = {{2, 2, 17, 17, 0, 2, 2, 19, 19, 2, 2, 6, 21, 21, 4, 2, 5, 27, 27, 10, 2, 5, 32, 32, 15, 2, 5, 37, 37, 20, 2, 5, 42, 42, 25, 1, 7, 10, 10, 30, 2, 5, 54, 54, 37, 2, 5, 59, 59, 42, 2, 5, 64, 64, 47, 2, 5, 69, 69, 52, 1, 7, 47, 47, 57, 2, 3, 85, 85, 64, 2, 3, 88, 88, 67, 2, 3, 91, 91, 70, 2, 5, 94, 94, 73, 2, 5, 99, 99, 78, 2, 3, 104, 104, 83, 1, 11, 74, 74, 86, 2, 3, 118, 118, 97, 2, 3, 121, 121, 100, 2, 5, 124, 124, 103, 2, 5, 129, 129, 108, 2, 5, 134, 134, 113, 2, 3, 139, 139, 118, 2, 5, 142, 142, 121, 1, 11, 107, 107, 126, 0, 10, 0, 0, 137, 3, 2, 157, 157, 147, 2, 3, 154, 154, 149, 3, 2, 162, 162, 152, 2, 3, 159, 159, 154, 1, 4, 150, 150, 157, 2, 3, 168, 168, 161, 1, 4, 164, 164, 164, 0, 3, 147, 147, 168, 3, 2, 186, 186, 171, 3, 2, 188, 188, 173, 3, 2, 190, 190, 175, 3, 2, 194, 194, 177, 2, 2, 192, 192, 179, 1, 10, 176, 176, 181, 3, 2, 206, 206, 191, 3, 2, 210, 210, 193, 2, 2, 208, 208, 195, 1, 10, 196, 196, 197, 3, 2, 222, 222, 207, 3, 2, 226, 226, 209, 2, 2, 224, 224, 211, 1, 10, 212, 212, 213, 3, 2, 239, 239, 223, 2, 2, 241, 241, 225, 2, 2, 243, 243, 227, 1, 11, 228, 228, 229, 0, 5, 171, 171, 240}};
    std::vector<float> splineLocs = {{-1.1000000000000001f, -1.02f, -0.51000000000000001f, -0.44f, -0.17999999999999999f, -0.16f, -0.14999999999999999f, -0.10000000000000001f, 0.25f, 1.0f, -0.84999999999999998f, -0.69999999999999996f, -0.40000000000000002f, -0.34999999999999998f, -0.10000000000000001f, 0.20000000000000001f, 0.69999999999999996f, -1.0f, 1.0f, -1.0f, 1.0f, -1.0f, -0.75f, -0.65000000000000002f, 0.5954547f, 0.60545470000000001f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -0.84999999999999998f, -0.69999999999999996f, -0.40000000000000002f, -0.34999999999999998f, -0.10000000000000001f, 0.20000000000000001f, 0.69999999999999996f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -0.84999999999999998f, -0.69999999999999996f, -0.40000000000000002f, -0.34999999999999998f, -0.10000000000000001f, 0.20000000000000001f, 0.40000000000000002f, 0.45000000000000001f, 0.55000000000000004f, 0.57999999999999996f, 0.69999999999999996f, -1.0f, 0.0f, 1.0f, -1.0f, 0.0f, 1.0f, -1.0f, 0.0f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, -0.84999999999999998f, -0.69999999999999996f, -0.40000000000000002f, -0.34999999999999998f, -0.10000000000000001f, 0.20000000000000001f, 0.40000000000000002f, 0.45000000000000001f, 0.55000000000000004f, 0.57999999999999996f, 0.69999999999999996f, -1.0f, 0.0f, 1.0f, -1.0f, 0.0f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -1.0f, -0.40000000000000002f, 0.0f, -1.0f, -0.40000000000000002f, 0.0f, 0.40000000000000002f, 1.0f, -0.11f, 0.029999999999999999f, 0.65000000000000002f, -1.0f, -0.78000000000000003f, -0.57750000000000001f, -0.375f, 0.19999998999999999f, 0.44999995999999998f, 1.0f, -0.01f, 0.01f, 0.19999998999999999f, 0.44999995999999998f, 1.0f, -0.01f, 0.01f, -1.0f, -0.78000000000000003f, -0.57750000000000001f, -0.375f, 0.19999998999999999f, 0.44999995999999998f, 1.0f, -0.19f, -0.14999999999999999f, -0.10000000000000001f, 0.029999999999999999f, 0.059999999999999998f, -0.59999999999999998f, -0.5f, -0.34999999999999998f, -0.25f, -0.10000000000000001f, 0.029999999999999999f, 0.34999999999999998f, 0.45000000000000001f, 0.55000000000000004f, 0.62f, -0.20000000000000001f, 0.20000000000000001f, -0.050000000000000003f, 0.050000000000000003f, -0.050000000000000003f, 0.050000000000000003f, -0.90000000000000002f, -0.68999999999999995f, 0.0f, 0.10000000000000001f, -0.59999999999999998f, -0.5f, -0.34999999999999998f, -0.25f, -0.10000000000000001f, 0.029999999999999999f, 0.34999999999999998f, 0.45000000000000001f, 0.55000000000000004f, 0.62f, -0.20000000000000001f, 0.20000000000000001f, -0.90000000000000002f, -0.68999999999999995f, 0.0f, 0.10000000000000001f, -0.59999999999999998f, -0.5f, -0.34999999999999998f, -0.25f, -0.10000000000000001f, 0.029999999999999999f, 0.34999999999999998f, 0.45000000000000001f, 0.55000000000000004f, 0.62f, -0.20000000000000001f, 0.20000000000000001f, -0.90000000000000002f, -0.68999999999999995f, 0.0f, 0.10000000000000001f, -0.59999999999999998f, -0.5f, -0.34999999999999998f, -0.25f, -0.10000000000000001f, 0.029999999999999999f, 0.050000000000000003f, 0.40000000000000002f, 0.45000000000000001f, 0.55000000000000004f, 0.57999999999999996f, -0.20000000000000001f, 0.20000000000000001f, 0.45000000000000001f, 0.69999999999999996f, -0.69999999999999996f, -0.14999999999999999f}};
    std::vector<float> splineDers = {{0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.38940096000000002f, 0.38940096000000002f, 0.37788021999999999f, 0.37788021999999999f, 0.0f, 0.0f, 0.0f, 0.0f, 0.25345630000000002f, 0.25345630000000002f, 0.5f, 0.0f, 0.0f, 0.0f, 0.0070000009999999996f, 0.5f, 0.0f, 0.0f, 0.10000000000000001f, 0.0070000009999999996f, 0.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.059999999999999998f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.5f, 0.0f, 0.0f, 0.0f, 0.0070000009999999996f, 0.5f, 0.01f, 0.01f, 0.094000003999999998f, 0.0070000009999999996f, 0.5f, 0.0f, 0.0f, 0.040000000000000001f, 0.049000000000000002f, 0.0f, 0.0f, 0.0f, 0.12f, 0.049000000000000002f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.51382490000000003f, 0.51382490000000003f, 0.0f, 0.43317973999999998f, 0.43317973999999998f, 0.0f, 0.39170509999999997f, 0.39170509999999997f, 0.5f, 0.0f, 0.0f, 0.0f, 0.049000014000000001f, 0.5f, 0.070000000000000007f, 0.070000000000000007f, 0.65800000000000003f, 0.049000014000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.57603689999999996f, 0.57603689999999996f, 0.0f, 0.4608295f, 0.4608295f, 0.5f, 0.0f, 0.0f, 0.0f, 0.070000014999999999f, 0.5f, 0.099999993999999995f, 0.099999993999999995f, 0.93999999999999995f, 0.070000014999999999f, 0.5f, 0.0f, 0.0f, 0.040000000000000001f, 0.049000000000000002f, 0.0f, 0.0f, 0.0f, 0.014999999999999999f, 0.0f, 0.0f, 0.040000000000000001f, 0.049000000000000002f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f}};
    std::vector<float> splineValF = {{-0.088801859999999996f, 0.69000006000000003f, -0.11576035599999999f, 0.64000009999999996f, -0.22220000000000001f, -0.22220000000000001f, 0.0f, 2.9802322000000001e-08f, 2.9802322000000001e-08f, 0.10000002400000001f, -0.29999999999999999f, 0.050000000000000003f, 0.050000000000000003f, 0.050000000000000003f, 0.060000001999999997f, -0.14999999999999999f, 0.0f, 0.0f, 0.050000000000000003f, 0.060000001999999997f, -0.14999999999999999f, 0.0f, 0.0f, 0.0f, 0.0f, -0.02f, -0.029999999999999999f, -0.029999999999999999f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -0.25f, 0.050000000000000003f, 0.050000000000000003f, 0.050000000000000003f, 0.060000001999999997f, -0.10000000000000001f, 0.001f, 0.0030000000000000001f, 0.050000000000000003f, 0.060000001999999997f, -0.10000000000000001f, 0.01f, 0.01f, 0.029999999999999999f, 0.10000000000000001f, -0.02f, -0.029999999999999999f, -0.029999999999999999f, 0.029999999999999999f, 0.10000000000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.20235021f, 0.71617509999999995f, 1.23f, 0.20000000000000001f, 0.44682026000000002f, 0.88f, 0.20000000000000001f, 0.30829495000000001f, 0.70000004999999998f, -0.25f, 0.34999999999999998f, 0.34999999999999998f, 0.34999999999999998f, 0.42000001999999997f, -0.10000000000000001f, 0.0069999998000000001f, 0.021000000000000001f, 0.34999999999999998f, 0.42000001999999997f, -0.10000000000000001f, 0.0f, 0.17000000000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.34792625999999999f, 0.92396310000000004f, 1.5f, 0.20000000000000001f, 0.5391705f, 1.0f, -0.20000000000000001f, 0.5f, 0.5f, 0.5f, 0.59999999999999998f, -0.050000000000000003f, 0.01f, 0.029999999999999999f, 0.5f, 0.59999999999999998f, -0.050000000000000003f, 0.01f, 0.01f, 0.029999999999999999f, 0.10000000000000001f, -0.050000000000000003f, 0.0f, 0.17000000000000001f, -0.02f, 0.01f, 0.01f, 0.029999999999999999f, 0.10000000000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.043999999999999997f, -0.22220000000000001f, -0.22220000000000001f, -0.12f, -0.12f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.63f, 0.29999999999999999f, 0.0f, 0.0f, 0.0f, 0.315f, 0.14999999999999999f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 6.2999999999999998f, 6.25f, 6.2999999999999998f, 2.6699999999999999f, 2.6699999999999999f, 6.2999999999999998f, 6.25f, 0.625f, 6.25f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 6.25f, 0.0f, 0.0f, 6.25f, 6.2999999999999998f, 5.4699999999999998f, 5.4699999999999998f, 0.625f, 5.4699999999999998f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 5.4699999999999998f, 0.0f, 0.0f, 5.4699999999999998f, 6.2999999999999998f, 5.0800000000000001f, 5.0800000000000001f, 0.625f, 5.0800000000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 5.0800000000000001f, 0.0f, 0.0f, 5.0800000000000001f, 6.2999999999999998f, 4.6900000000000004f, 0.0f, 1.5600000000000001f, 0.0f, 1.3700000000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 4.6900000000000004f, 3.9500000000000002f, 0.0f, 0.0f, 0.0f, 0.0f}};
    std::vector<int> splineValKind = {{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1}};
    std::vector<int> splineValNode = {{-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 8, 9, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10, -1, 13, 14, 15, 16, 17, 10, 10, 18, 18, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 24, -1, -1, -1, -1, -1, -1, 20, 21, 21, 22, 23, 24, 24, 25, 25, 24, 26, -1, -1, -1, -1, -1, 7, 7, 12, 19, 27, -1, -1, -1, -1, 29, -1, -1, -1, -1, 31, 30, 32, 32, -1, -1, 29, 29, 34, 30, 30, -1, -1, 33, 35, -1, -1, -1, -1, -1, -1, -1, -1, -1, 40, 37, 38, 37, 37, 39, 37, -1, 41, 41, -1, -1, -1, -1, -1, -1, 44, 43, 38, 43, 43, 39, 43, -1, 45, 45, -1, -1, -1, -1, -1, -1, 48, 47, 38, 47, 47, 39, 47, -1, 49, 49, -1, -1, -1, 51, -1, 51, -1, 51, 38, 51, 51, 39, 51, 52, 52, 53, 53, -1, -1, 42, 46, 50, 54}};


    static int floorDiv(int a, int b) { int r = a / b; if ((a % b) != 0 && ((a ^ b) < 0)) r--; return r; }
    static const int minY = -64;   // overworld 维度 minY（interpolated cell 网格）
    static double maintainPrecision(double v) { return v - (long)(v / 3.3554432E7 + 0.5) * 3.3554432E7; }

    void init(uint64_t worldSeed) {
        wg::XoroshiroRandom base(worldSeed);
        auto rd = base.nextSplitter();
    { auto r = rd.split("minecraft:offset"); shiftNoises.emplace("minecraft:offset", wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-3, {1, 1, 1, 0}})); }
    { auto r = rd.split("minecraft:continentalness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 2, 2, 2, 1, 1, 1, 1}})); n.push_back(9); octBase.push_back(0); splitBase.push_back(0); }
    { auto r = rd.split("minecraft:continentalness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 2, 2, 2, 1, 1, 1, 1}})); n.push_back(9); octBase.push_back(18); splitBase.push_back(108); }
    { auto r = rd.split("minecraft:continentalness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 2, 2, 2, 1, 1, 1, 1}})); n.push_back(9); octBase.push_back(36); splitBase.push_back(216); }
    { auto r = rd.split("minecraft:continentalness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 2, 2, 2, 1, 1, 1, 1}})); n.push_back(9); octBase.push_back(54); splitBase.push_back(324); }
    { auto r = rd.split("minecraft:continentalness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 2, 2, 2, 1, 1, 1, 1}})); n.push_back(9); octBase.push_back(72); splitBase.push_back(432); }
    { auto r = rd.split("minecraft:continentalness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 2, 2, 2, 1, 1, 1, 1}})); n.push_back(9); octBase.push_back(90); splitBase.push_back(540); }
    { auto r = rd.split("minecraft:continentalness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 2, 2, 2, 1, 1, 1, 1}})); n.push_back(9); octBase.push_back(108); splitBase.push_back(648); }
    { auto r = rd.split("minecraft:continentalness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 2, 2, 2, 1, 1, 1, 1}})); n.push_back(9); octBase.push_back(126); splitBase.push_back(756); }
    { auto r = rd.split("minecraft:erosion"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 0, 1, 1}})); n.push_back(5); octBase.push_back(144); splitBase.push_back(864); }
    { auto r = rd.split("minecraft:erosion"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 0, 1, 1}})); n.push_back(5); octBase.push_back(154); splitBase.push_back(924); }
    { auto r = rd.split("minecraft:erosion"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 0, 1, 1}})); n.push_back(5); octBase.push_back(164); splitBase.push_back(984); }
    { auto r = rd.split("minecraft:erosion"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 0, 1, 1}})); n.push_back(5); octBase.push_back(174); splitBase.push_back(1044); }
    { auto r = rd.split("minecraft:erosion"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 0, 1, 1}})); n.push_back(5); octBase.push_back(184); splitBase.push_back(1104); }
    { auto r = rd.split("minecraft:erosion"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 0, 1, 1}})); n.push_back(5); octBase.push_back(194); splitBase.push_back(1164); }
    { auto r = rd.split("minecraft:erosion"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 0, 1, 1}})); n.push_back(5); octBase.push_back(204); splitBase.push_back(1224); }
    { auto r = rd.split("minecraft:erosion"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-9, {1, 1, 0, 1, 1}})); n.push_back(5); octBase.push_back(214); splitBase.push_back(1284); }
    { auto r = rd.split("minecraft:ridge"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 2, 1, 0, 0, 0}})); n.push_back(6); octBase.push_back(224); splitBase.push_back(1344); }
    { auto r = rd.split("minecraft:ridge"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 2, 1, 0, 0, 0}})); n.push_back(6); octBase.push_back(236); splitBase.push_back(1416); }
    { auto r = rd.split("minecraft:ridge"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 2, 1, 0, 0, 0}})); n.push_back(6); octBase.push_back(248); splitBase.push_back(1488); }
    { auto r = rd.split("minecraft:ridge"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 2, 1, 0, 0, 0}})); n.push_back(6); octBase.push_back(260); splitBase.push_back(1560); }
    { auto r = rd.split("minecraft:ridge"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 2, 1, 0, 0, 0}})); n.push_back(6); octBase.push_back(272); splitBase.push_back(1632); }
    { auto r = rd.split("minecraft:ridge"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 2, 1, 0, 0, 0}})); n.push_back(6); octBase.push_back(284); splitBase.push_back(1704); }
    { auto r = rd.split("minecraft:ridge"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 2, 1, 0, 0, 0}})); n.push_back(6); octBase.push_back(296); splitBase.push_back(1776); }
    { auto r = rd.split("minecraft:ridge"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 2, 1, 0, 0, 0}})); n.push_back(6); octBase.push_back(308); splitBase.push_back(1848); }
    { auto r = rd.split("minecraft:jagged"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-16, {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}})); n.push_back(16); octBase.push_back(320); splitBase.push_back(1920); }
    { auto r = rd.split("minecraft:jagged"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-16, {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}})); n.push_back(16); octBase.push_back(352); splitBase.push_back(2112); }
    { auto r = rd.split("minecraft:jagged"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-16, {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}})); n.push_back(16); octBase.push_back(384); splitBase.push_back(2304); }
    { auto r = rd.split("minecraft:jagged"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-16, {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}})); n.push_back(16); octBase.push_back(416); splitBase.push_back(2496); }
    { auto r = rd.split("minecraft:jagged"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-16, {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}})); n.push_back(16); octBase.push_back(448); splitBase.push_back(2688); }
    { auto r = rd.split("minecraft:jagged"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-16, {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}})); n.push_back(16); octBase.push_back(480); splitBase.push_back(2880); }
    { auto r = rd.split("minecraft:jagged"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-16, {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}})); n.push_back(16); octBase.push_back(512); splitBase.push_back(3072); }
    { auto r = rd.split("minecraft:jagged"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-16, {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}})); n.push_back(16); octBase.push_back(544); splitBase.push_back(3264); }
    { auto r = rd.split("minecraft:cave_entrance"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {0.40000000000000002, 0.5, 1}})); n.push_back(3); octBase.push_back(896); splitBase.push_back(5696); }
    { auto r = rd.split("minecraft:cave_entrance"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {0.40000000000000002, 0.5, 1}})); n.push_back(3); octBase.push_back(902); splitBase.push_back(5732); }
    { auto r = rd.split("minecraft:cave_entrance"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {0.40000000000000002, 0.5, 1}})); n.push_back(3); octBase.push_back(908); splitBase.push_back(5768); }
    { auto r = rd.split("minecraft:cave_entrance"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {0.40000000000000002, 0.5, 1}})); n.push_back(3); octBase.push_back(914); splitBase.push_back(5804); }
    { auto r = rd.split("minecraft:cave_entrance"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {0.40000000000000002, 0.5, 1}})); n.push_back(3); octBase.push_back(920); splitBase.push_back(5840); }
    { auto r = rd.split("minecraft:cave_entrance"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {0.40000000000000002, 0.5, 1}})); n.push_back(3); octBase.push_back(926); splitBase.push_back(5876); }
    { auto r = rd.split("minecraft:cave_entrance"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {0.40000000000000002, 0.5, 1}})); n.push_back(3); octBase.push_back(932); splitBase.push_back(5912); }
    { auto r = rd.split("minecraft:cave_entrance"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {0.40000000000000002, 0.5, 1}})); n.push_back(3); octBase.push_back(938); splitBase.push_back(5948); }
    { auto r = rd.split("minecraft:spaghetti_roughness_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(944); splitBase.push_back(5984); }
    { auto r = rd.split("minecraft:spaghetti_roughness_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(946); splitBase.push_back(5996); }
    { auto r = rd.split("minecraft:spaghetti_roughness_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(948); splitBase.push_back(6008); }
    { auto r = rd.split("minecraft:spaghetti_roughness_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(950); splitBase.push_back(6020); }
    { auto r = rd.split("minecraft:spaghetti_roughness_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(952); splitBase.push_back(6032); }
    { auto r = rd.split("minecraft:spaghetti_roughness_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(954); splitBase.push_back(6044); }
    { auto r = rd.split("minecraft:spaghetti_roughness_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(956); splitBase.push_back(6056); }
    { auto r = rd.split("minecraft:spaghetti_roughness_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(958); splitBase.push_back(6068); }
    { auto r = rd.split("minecraft:spaghetti_roughness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-5, {1}})); n.push_back(1); octBase.push_back(960); splitBase.push_back(6080); }
    { auto r = rd.split("minecraft:spaghetti_roughness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-5, {1}})); n.push_back(1); octBase.push_back(962); splitBase.push_back(6092); }
    { auto r = rd.split("minecraft:spaghetti_roughness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-5, {1}})); n.push_back(1); octBase.push_back(964); splitBase.push_back(6104); }
    { auto r = rd.split("minecraft:spaghetti_roughness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-5, {1}})); n.push_back(1); octBase.push_back(966); splitBase.push_back(6116); }
    { auto r = rd.split("minecraft:spaghetti_roughness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-5, {1}})); n.push_back(1); octBase.push_back(968); splitBase.push_back(6128); }
    { auto r = rd.split("minecraft:spaghetti_roughness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-5, {1}})); n.push_back(1); octBase.push_back(970); splitBase.push_back(6140); }
    { auto r = rd.split("minecraft:spaghetti_roughness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-5, {1}})); n.push_back(1); octBase.push_back(972); splitBase.push_back(6152); }
    { auto r = rd.split("minecraft:spaghetti_roughness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-5, {1}})); n.push_back(1); octBase.push_back(974); splitBase.push_back(6164); }
    { auto r = rd.split("minecraft:spaghetti_3d_rarity"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(976); splitBase.push_back(6176); }
    { auto r = rd.split("minecraft:spaghetti_3d_rarity"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(978); splitBase.push_back(6188); }
    { auto r = rd.split("minecraft:spaghetti_3d_rarity"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(980); splitBase.push_back(6200); }
    { auto r = rd.split("minecraft:spaghetti_3d_rarity"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(982); splitBase.push_back(6212); }
    { auto r = rd.split("minecraft:spaghetti_3d_rarity"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(984); splitBase.push_back(6224); }
    { auto r = rd.split("minecraft:spaghetti_3d_rarity"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(986); splitBase.push_back(6236); }
    { auto r = rd.split("minecraft:spaghetti_3d_rarity"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(988); splitBase.push_back(6248); }
    { auto r = rd.split("minecraft:spaghetti_3d_rarity"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(990); splitBase.push_back(6260); }
    { auto r = rd.split("minecraft:spaghetti_3d_1"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(992); splitBase.push_back(6272); }
    { auto r = rd.split("minecraft:spaghetti_3d_1"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(994); splitBase.push_back(6284); }
    { auto r = rd.split("minecraft:spaghetti_3d_1"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(996); splitBase.push_back(6296); }
    { auto r = rd.split("minecraft:spaghetti_3d_1"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(998); splitBase.push_back(6308); }
    { auto r = rd.split("minecraft:spaghetti_3d_1"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1000); splitBase.push_back(6320); }
    { auto r = rd.split("minecraft:spaghetti_3d_1"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1002); splitBase.push_back(6332); }
    { auto r = rd.split("minecraft:spaghetti_3d_1"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1004); splitBase.push_back(6344); }
    { auto r = rd.split("minecraft:spaghetti_3d_1"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1006); splitBase.push_back(6356); }
    { auto r = rd.split("minecraft:spaghetti_3d_2"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1008); splitBase.push_back(6368); }
    { auto r = rd.split("minecraft:spaghetti_3d_2"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1010); splitBase.push_back(6380); }
    { auto r = rd.split("minecraft:spaghetti_3d_2"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1012); splitBase.push_back(6392); }
    { auto r = rd.split("minecraft:spaghetti_3d_2"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1014); splitBase.push_back(6404); }
    { auto r = rd.split("minecraft:spaghetti_3d_2"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1016); splitBase.push_back(6416); }
    { auto r = rd.split("minecraft:spaghetti_3d_2"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1018); splitBase.push_back(6428); }
    { auto r = rd.split("minecraft:spaghetti_3d_2"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1020); splitBase.push_back(6440); }
    { auto r = rd.split("minecraft:spaghetti_3d_2"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1022); splitBase.push_back(6452); }
    { auto r = rd.split("minecraft:spaghetti_3d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1024); splitBase.push_back(6464); }
    { auto r = rd.split("minecraft:spaghetti_3d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1026); splitBase.push_back(6476); }
    { auto r = rd.split("minecraft:spaghetti_3d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1028); splitBase.push_back(6488); }
    { auto r = rd.split("minecraft:spaghetti_3d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1030); splitBase.push_back(6500); }
    { auto r = rd.split("minecraft:spaghetti_3d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1032); splitBase.push_back(6512); }
    { auto r = rd.split("minecraft:spaghetti_3d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1034); splitBase.push_back(6524); }
    { auto r = rd.split("minecraft:spaghetti_3d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1036); splitBase.push_back(6536); }
    { auto r = rd.split("minecraft:spaghetti_3d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1038); splitBase.push_back(6548); }
    { auto r = rd.split("minecraft:cave_layer"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1040); splitBase.push_back(6560); }
    { auto r = rd.split("minecraft:cave_layer"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1042); splitBase.push_back(6572); }
    { auto r = rd.split("minecraft:cave_layer"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1044); splitBase.push_back(6584); }
    { auto r = rd.split("minecraft:cave_layer"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1046); splitBase.push_back(6596); }
    { auto r = rd.split("minecraft:cave_layer"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1048); splitBase.push_back(6608); }
    { auto r = rd.split("minecraft:cave_layer"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1050); splitBase.push_back(6620); }
    { auto r = rd.split("minecraft:cave_layer"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1052); splitBase.push_back(6632); }
    { auto r = rd.split("minecraft:cave_layer"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1054); splitBase.push_back(6644); }
    { auto r = rd.split("minecraft:cave_cheese"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {0.5, 1, 2, 1, 2, 1, 0, 2, 0}})); n.push_back(9); octBase.push_back(1056); splitBase.push_back(6656); }
    { auto r = rd.split("minecraft:cave_cheese"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {0.5, 1, 2, 1, 2, 1, 0, 2, 0}})); n.push_back(9); octBase.push_back(1074); splitBase.push_back(6764); }
    { auto r = rd.split("minecraft:cave_cheese"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {0.5, 1, 2, 1, 2, 1, 0, 2, 0}})); n.push_back(9); octBase.push_back(1092); splitBase.push_back(6872); }
    { auto r = rd.split("minecraft:cave_cheese"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {0.5, 1, 2, 1, 2, 1, 0, 2, 0}})); n.push_back(9); octBase.push_back(1110); splitBase.push_back(6980); }
    { auto r = rd.split("minecraft:cave_cheese"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {0.5, 1, 2, 1, 2, 1, 0, 2, 0}})); n.push_back(9); octBase.push_back(1128); splitBase.push_back(7088); }
    { auto r = rd.split("minecraft:cave_cheese"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {0.5, 1, 2, 1, 2, 1, 0, 2, 0}})); n.push_back(9); octBase.push_back(1146); splitBase.push_back(7196); }
    { auto r = rd.split("minecraft:cave_cheese"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {0.5, 1, 2, 1, 2, 1, 0, 2, 0}})); n.push_back(9); octBase.push_back(1164); splitBase.push_back(7304); }
    { auto r = rd.split("minecraft:cave_cheese"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {0.5, 1, 2, 1, 2, 1, 0, 2, 0}})); n.push_back(9); octBase.push_back(1182); splitBase.push_back(7412); }
    { auto r = rd.split("minecraft:spaghetti_2d_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1200); splitBase.push_back(7520); }
    { auto r = rd.split("minecraft:spaghetti_2d_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1202); splitBase.push_back(7532); }
    { auto r = rd.split("minecraft:spaghetti_2d_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1204); splitBase.push_back(7544); }
    { auto r = rd.split("minecraft:spaghetti_2d_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1206); splitBase.push_back(7556); }
    { auto r = rd.split("minecraft:spaghetti_2d_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1208); splitBase.push_back(7568); }
    { auto r = rd.split("minecraft:spaghetti_2d_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1210); splitBase.push_back(7580); }
    { auto r = rd.split("minecraft:spaghetti_2d_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1212); splitBase.push_back(7592); }
    { auto r = rd.split("minecraft:spaghetti_2d_modulator"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1214); splitBase.push_back(7604); }
    { auto r = rd.split("minecraft:spaghetti_2d"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1216); splitBase.push_back(7616); }
    { auto r = rd.split("minecraft:spaghetti_2d"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1218); splitBase.push_back(7628); }
    { auto r = rd.split("minecraft:spaghetti_2d"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1220); splitBase.push_back(7640); }
    { auto r = rd.split("minecraft:spaghetti_2d"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1222); splitBase.push_back(7652); }
    { auto r = rd.split("minecraft:spaghetti_2d"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1224); splitBase.push_back(7664); }
    { auto r = rd.split("minecraft:spaghetti_2d"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1226); splitBase.push_back(7676); }
    { auto r = rd.split("minecraft:spaghetti_2d"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1228); splitBase.push_back(7688); }
    { auto r = rd.split("minecraft:spaghetti_2d"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1230); splitBase.push_back(7700); }
    { auto r = rd.split("minecraft:spaghetti_2d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1232); splitBase.push_back(7712); }
    { auto r = rd.split("minecraft:spaghetti_2d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1234); splitBase.push_back(7724); }
    { auto r = rd.split("minecraft:spaghetti_2d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1236); splitBase.push_back(7736); }
    { auto r = rd.split("minecraft:spaghetti_2d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1238); splitBase.push_back(7748); }
    { auto r = rd.split("minecraft:spaghetti_2d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1240); splitBase.push_back(7760); }
    { auto r = rd.split("minecraft:spaghetti_2d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1242); splitBase.push_back(7772); }
    { auto r = rd.split("minecraft:spaghetti_2d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1244); splitBase.push_back(7784); }
    { auto r = rd.split("minecraft:spaghetti_2d_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-11, {1}})); n.push_back(1); octBase.push_back(1246); splitBase.push_back(7796); }
    { auto r = rd.split("minecraft:spaghetti_2d_elevation"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1248); splitBase.push_back(7808); }
    { auto r = rd.split("minecraft:spaghetti_2d_elevation"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1250); splitBase.push_back(7820); }
    { auto r = rd.split("minecraft:spaghetti_2d_elevation"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1252); splitBase.push_back(7832); }
    { auto r = rd.split("minecraft:spaghetti_2d_elevation"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1254); splitBase.push_back(7844); }
    { auto r = rd.split("minecraft:spaghetti_2d_elevation"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1256); splitBase.push_back(7856); }
    { auto r = rd.split("minecraft:spaghetti_2d_elevation"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1258); splitBase.push_back(7868); }
    { auto r = rd.split("minecraft:spaghetti_2d_elevation"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1260); splitBase.push_back(7880); }
    { auto r = rd.split("minecraft:spaghetti_2d_elevation"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1262); splitBase.push_back(7892); }
    { auto r = rd.split("minecraft:pillar"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 1}})); n.push_back(2); octBase.push_back(1264); splitBase.push_back(7904); }
    { auto r = rd.split("minecraft:pillar"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 1}})); n.push_back(2); octBase.push_back(1268); splitBase.push_back(7928); }
    { auto r = rd.split("minecraft:pillar"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 1}})); n.push_back(2); octBase.push_back(1272); splitBase.push_back(7952); }
    { auto r = rd.split("minecraft:pillar"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 1}})); n.push_back(2); octBase.push_back(1276); splitBase.push_back(7976); }
    { auto r = rd.split("minecraft:pillar"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 1}})); n.push_back(2); octBase.push_back(1280); splitBase.push_back(8000); }
    { auto r = rd.split("minecraft:pillar"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 1}})); n.push_back(2); octBase.push_back(1284); splitBase.push_back(8024); }
    { auto r = rd.split("minecraft:pillar"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 1}})); n.push_back(2); octBase.push_back(1288); splitBase.push_back(8048); }
    { auto r = rd.split("minecraft:pillar"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1, 1}})); n.push_back(2); octBase.push_back(1292); splitBase.push_back(8072); }
    { auto r = rd.split("minecraft:pillar_rareness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1296); splitBase.push_back(8096); }
    { auto r = rd.split("minecraft:pillar_rareness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1298); splitBase.push_back(8108); }
    { auto r = rd.split("minecraft:pillar_rareness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1300); splitBase.push_back(8120); }
    { auto r = rd.split("minecraft:pillar_rareness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1302); splitBase.push_back(8132); }
    { auto r = rd.split("minecraft:pillar_rareness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1304); splitBase.push_back(8144); }
    { auto r = rd.split("minecraft:pillar_rareness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1306); splitBase.push_back(8156); }
    { auto r = rd.split("minecraft:pillar_rareness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1308); splitBase.push_back(8168); }
    { auto r = rd.split("minecraft:pillar_rareness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1310); splitBase.push_back(8180); }
    { auto r = rd.split("minecraft:pillar_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1312); splitBase.push_back(8192); }
    { auto r = rd.split("minecraft:pillar_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1314); splitBase.push_back(8204); }
    { auto r = rd.split("minecraft:pillar_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1316); splitBase.push_back(8216); }
    { auto r = rd.split("minecraft:pillar_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1318); splitBase.push_back(8228); }
    { auto r = rd.split("minecraft:pillar_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1320); splitBase.push_back(8240); }
    { auto r = rd.split("minecraft:pillar_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1322); splitBase.push_back(8252); }
    { auto r = rd.split("minecraft:pillar_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1324); splitBase.push_back(8264); }
    { auto r = rd.split("minecraft:pillar_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1326); splitBase.push_back(8276); }
    { auto r = rd.split("minecraft:noodle"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1328); splitBase.push_back(8288); }
    { auto r = rd.split("minecraft:noodle"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1330); splitBase.push_back(8300); }
    { auto r = rd.split("minecraft:noodle"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1332); splitBase.push_back(8312); }
    { auto r = rd.split("minecraft:noodle"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1334); splitBase.push_back(8324); }
    { auto r = rd.split("minecraft:noodle"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1336); splitBase.push_back(8336); }
    { auto r = rd.split("minecraft:noodle"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1338); splitBase.push_back(8348); }
    { auto r = rd.split("minecraft:noodle"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1340); splitBase.push_back(8360); }
    { auto r = rd.split("minecraft:noodle"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1342); splitBase.push_back(8372); }
    { auto r = rd.split("minecraft:noodle_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1344); splitBase.push_back(8384); }
    { auto r = rd.split("minecraft:noodle_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1346); splitBase.push_back(8396); }
    { auto r = rd.split("minecraft:noodle_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1348); splitBase.push_back(8408); }
    { auto r = rd.split("minecraft:noodle_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1350); splitBase.push_back(8420); }
    { auto r = rd.split("minecraft:noodle_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1352); splitBase.push_back(8432); }
    { auto r = rd.split("minecraft:noodle_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1354); splitBase.push_back(8444); }
    { auto r = rd.split("minecraft:noodle_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1356); splitBase.push_back(8456); }
    { auto r = rd.split("minecraft:noodle_thickness"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1}})); n.push_back(1); octBase.push_back(1358); splitBase.push_back(8468); }
    { auto r = rd.split("minecraft:noodle_ridge_a"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1360); splitBase.push_back(8480); }
    { auto r = rd.split("minecraft:noodle_ridge_a"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1362); splitBase.push_back(8492); }
    { auto r = rd.split("minecraft:noodle_ridge_a"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1364); splitBase.push_back(8504); }
    { auto r = rd.split("minecraft:noodle_ridge_a"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1366); splitBase.push_back(8516); }
    { auto r = rd.split("minecraft:noodle_ridge_a"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1368); splitBase.push_back(8528); }
    { auto r = rd.split("minecraft:noodle_ridge_a"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1370); splitBase.push_back(8540); }
    { auto r = rd.split("minecraft:noodle_ridge_a"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1372); splitBase.push_back(8552); }
    { auto r = rd.split("minecraft:noodle_ridge_a"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1374); splitBase.push_back(8564); }
    { auto r = rd.split("minecraft:noodle_ridge_b"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1376); splitBase.push_back(8576); }
    { auto r = rd.split("minecraft:noodle_ridge_b"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1378); splitBase.push_back(8588); }
    { auto r = rd.split("minecraft:noodle_ridge_b"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1380); splitBase.push_back(8600); }
    { auto r = rd.split("minecraft:noodle_ridge_b"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1382); splitBase.push_back(8612); }
    { auto r = rd.split("minecraft:noodle_ridge_b"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1384); splitBase.push_back(8624); }
    { auto r = rd.split("minecraft:noodle_ridge_b"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1386); splitBase.push_back(8636); }
    { auto r = rd.split("minecraft:noodle_ridge_b"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1388); splitBase.push_back(8648); }
    { auto r = rd.split("minecraft:noodle_ridge_b"); normals.emplace_back(wg::DoublePerlinNoiseSampler(r, wg::DoublePerlinNoiseSampler::NoiseParameters{-7, {1}})); n.push_back(1); octBase.push_back(1390); splitBase.push_back(8660); }
    { wg::XoroshiroRandom r = rd.split("minecraft:terrain"); oldBlendeds.push_back(std::make_shared<wg::InterpolatedNoiseDF>(r, 0.25, 0.125, 80, 160, 8)); oldBase.push_back(576); oldSplitBase.push_back(3456); }
    { wg::XoroshiroRandom r = rd.split("minecraft:terrain"); oldBlendeds.push_back(std::make_shared<wg::InterpolatedNoiseDF>(r, 0.25, 0.125, 80, 160, 8)); oldBase.push_back(616); oldSplitBase.push_back(3736); }
    { wg::XoroshiroRandom r = rd.split("minecraft:terrain"); oldBlendeds.push_back(std::make_shared<wg::InterpolatedNoiseDF>(r, 0.25, 0.125, 80, 160, 8)); oldBase.push_back(656); oldSplitBase.push_back(4016); }
    { wg::XoroshiroRandom r = rd.split("minecraft:terrain"); oldBlendeds.push_back(std::make_shared<wg::InterpolatedNoiseDF>(r, 0.25, 0.125, 80, 160, 8)); oldBase.push_back(696); oldSplitBase.push_back(4296); }
    { wg::XoroshiroRandom r = rd.split("minecraft:terrain"); oldBlendeds.push_back(std::make_shared<wg::InterpolatedNoiseDF>(r, 0.25, 0.125, 80, 160, 8)); oldBase.push_back(736); oldSplitBase.push_back(4576); }
    { wg::XoroshiroRandom r = rd.split("minecraft:terrain"); oldBlendeds.push_back(std::make_shared<wg::InterpolatedNoiseDF>(r, 0.25, 0.125, 80, 160, 8)); oldBase.push_back(776); oldSplitBase.push_back(4856); }
    { wg::XoroshiroRandom r = rd.split("minecraft:terrain"); oldBlendeds.push_back(std::make_shared<wg::InterpolatedNoiseDF>(r, 0.25, 0.125, 80, 160, 8)); oldBase.push_back(816); oldSplitBase.push_back(5136); }
    { wg::XoroshiroRandom r = rd.split("minecraft:terrain"); oldBlendeds.push_back(std::make_shared<wg::InterpolatedNoiseDF>(r, 0.25, 0.125, 80, 160, 8)); oldBase.push_back(856); oldSplitBase.push_back(5416); }
    }

    static void splitOctave(const wg::PerlinNoiseSampler* pn, double cx, double cy, double cz, float* out) {
        double ox = pn ? pn->originX : 0.0, oy = pn ? pn->originY : 0.0, oz = pn ? pn->originZ : 0.0;
        int ix = (int)std::floor(cx + ox), iy = (int)std::floor(cy + oy), iz = (int)std::floor(cz + oz);
        out[0] = (float)ix; out[1] = (float)iy; out[2] = (float)iz;
        out[3] = (float)(cx + ox - ix); out[4] = (float)(cy + oy - iy); out[5] = (float)(cz + oz - iz);
    }

    static void splitDouble(const wg::DoublePerlinNoiseSampler& noise, double dx, double dy, double dz, float* out, int base, int nn) {
        double lacunarity = std::pow(2.0, noise.firstSampler.firstOctave);
        double e = lacunarity;
        for (int i = 0; i < nn; i++) {
            splitOctave(noise.firstSampler.octaveSamplers[i].get(),
                        maintainPrecision(dx*e), maintainPrecision(dy*e), maintainPrecision(dz*e),
                        &out[base + i * 6]);
            splitOctave(noise.secondSampler.octaveSamplers[i].get(),
                        maintainPrecision(dx*1.0181268882175227*e), maintainPrecision(dy*1.0181268882175227*e), maintainPrecision(dz*1.0181268882175227*e),
                        &out[base + 6 * nn + i * 6]);
            e *= 2.0;
        }
    }

    // 5 参数 sample 拆分：out = [ix,iy,iz,gx,gy(=h-n),gz,fadeY(=h)]
    static void split7(const wg::PerlinNoiseSampler* pn, double x, double y, double z, double yScale, double yMax, float* out) {
        double sx = x + pn->originX, sy = y + pn->originY, sz = z + pn->originZ;
        int ix = wg::floorD(sx), iy = wg::floorD(sy), iz = wg::floorD(sz);
        double gx = sx - ix, gy_raw = sy - iy, gz = sz - iz;
        double n;
        if (yScale != 0.0) {
            double m = (yMax >= 0.0 && yMax < gy_raw) ? yMax : gy_raw;
            n = wg::floorD(m / yScale + 1.0E-7F) * yScale;
        } else n = 0.0;
        out[0] = (float)ix; out[1] = (float)iy; out[2] = (float)iz;
        out[3] = (float)gx; out[4] = (float)(gy_raw - n); out[5] = (float)gz; out[6] = (float)gy_raw;
    }

    // D17: weird_scaled_sampler scaleValue（kind 1=CAVES, 0=TUNNELS）
    static double ws_scale(int kind, double v) {
        if (kind == 1) {
            if (v < -0.75) return 0.5;
            if (v < -0.5) return 0.75;
            if (v < 0.5) return 1.0;
            return v < 0.75 ? 2.0 : 3.0;
        }
        if (v < -0.5) return 0.75;
        if (v < 0.0) return 1.0;
        return v < 0.5 ? 1.5 : 2.0;
    }

    static void splitOldBlended(const wg::InterpolatedNoiseDF& ob, int x, int y, int z, float* out, int base) {
        double d = x * ob.scaledXzScale;
        double e = y * ob.scaledYScale;
        double f = z * ob.scaledXzScale;
        double g = d / ob.xzFactor;
        double h = e / ob.yFactor;
        double i = f / ob.xzFactor;
        double j = ob.scaledYScale * ob.smearScaleMultiplier;
        double k = j / ob.yFactor;
        double o = 1.0;
        for (int q = 0; q < 8; q++) {
            split7(ob.interpolation.getOctave(q), maintainPrecision(g*o), maintainPrecision(h*o), maintainPrecision(i*o), k*o, h*o, &out[base + (32+q)*7]);
            o /= 2.0;
        }
        o = 1.0;
        for (int r = 0; r < 16; r++) {
            double s2 = maintainPrecision(d*o), t2 = maintainPrecision(e*o), u2 = maintainPrecision(f*o);
            split7(ob.lower.getOctave(r), s2, t2, u2, j*o, e*o, &out[base + r*7]);
            split7(ob.upper.getOctave(r), s2, t2, u2, j*o, e*o, &out[base + (16+r)*7]);
            o /= 2.0;
        }
    }

    void split(int x, int y, int z, float* out) {
    {
        int _chunkX = floorDiv(x, 16); int _chunkZ = floorDiv(z, 16);
        int _gx = (x) - _chunkX * 16; int _gy = (y) - minY; int _gz = (z) - _chunkZ * 16;
        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;
    { splitDouble(normals[0], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 0, 9); }
    { splitDouble(normals[8], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 864, 5); }
    { splitDouble(normals[16], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1344, 6); }
    { splitDouble(normals[24], ((_chunkX * 16 + (_cx + 0) * 4)) * 1500, ((minY + (_cy + 0) * 8)) * 0, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1500, out, 1920, 16); }
    { splitOldBlended(*oldBlendeds[0], (_chunkX * 16 + (_cx + 0) * 4), (minY + (_cy + 0) * 8), (_chunkZ * 16 + (_cz + 0) * 4), out, 3456); }
    { splitDouble(normals[32], ((_chunkX * 16 + (_cx + 0) * 4)) * 0.75, ((minY + (_cy + 0) * 8)) * 0.5, ((_chunkZ * 16 + (_cz + 0) * 4)) * 0.75, out, 5696, 3); }
    { splitDouble(normals[40], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 5984, 1); }
    { splitDouble(normals[48], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6080, 1); }
    { splitDouble(normals[56], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 6176, 1); }
    { double _d = ws_scale(0, normals[56].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[64], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 6272, 1); }
    { double _d = ws_scale(0, normals[56].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[72], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 6368, 1); }
    { splitDouble(normals[80], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6464, 1); }
    { splitDouble(normals[88], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 8, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6560, 1); }
    { splitDouble(normals[96], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 0.66666666666666663, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6656, 9); }
    { splitDouble(normals[104], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 7520, 1); }
    { double _d = ws_scale(1, normals[104].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[112], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 7616, 1); }
    { splitDouble(normals[120], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 7712, 1); }
    { splitDouble(normals[128], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 0, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 7808, 1); }
    { splitDouble(normals[136], ((_chunkX * 16 + (_cx + 0) * 4)) * 25, ((minY + (_cy + 0) * 8)) * 0.29999999999999999, ((_chunkZ * 16 + (_cz + 0) * 4)) * 25, out, 7904, 2); }
    { splitDouble(normals[144], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8096, 1); }
    { splitDouble(normals[152], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8192, 1); }
    { splitDouble(normals[1], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 108, 9); }
    { splitDouble(normals[9], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 924, 5); }
    { splitDouble(normals[17], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1416, 6); }
    { splitDouble(normals[25], ((_chunkX * 16 + (_cx + 1) * 4)) * 1500, ((minY + (_cy + 0) * 8)) * 0, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1500, out, 2112, 16); }
    { splitOldBlended(*oldBlendeds[1], (_chunkX * 16 + (_cx + 1) * 4), (minY + (_cy + 0) * 8), (_chunkZ * 16 + (_cz + 0) * 4), out, 3736); }
    { splitDouble(normals[33], ((_chunkX * 16 + (_cx + 1) * 4)) * 0.75, ((minY + (_cy + 0) * 8)) * 0.5, ((_chunkZ * 16 + (_cz + 0) * 4)) * 0.75, out, 5732, 3); }
    { splitDouble(normals[41], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 5996, 1); }
    { splitDouble(normals[49], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6092, 1); }
    { splitDouble(normals[57], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 6188, 1); }
    { double _d = ws_scale(0, normals[57].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[65], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 6284, 1); }
    { double _d = ws_scale(0, normals[57].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[73], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 6380, 1); }
    { splitDouble(normals[81], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6476, 1); }
    { splitDouble(normals[89], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 8, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6572, 1); }
    { splitDouble(normals[97], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 0.66666666666666663, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6764, 9); }
    { splitDouble(normals[105], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 7532, 1); }
    { double _d = ws_scale(1, normals[105].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[113], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 7628, 1); }
    { splitDouble(normals[121], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 7724, 1); }
    { splitDouble(normals[129], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 0, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 7820, 1); }
    { splitDouble(normals[137], ((_chunkX * 16 + (_cx + 1) * 4)) * 25, ((minY + (_cy + 0) * 8)) * 0.29999999999999999, ((_chunkZ * 16 + (_cz + 0) * 4)) * 25, out, 7928, 2); }
    { splitDouble(normals[145], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8108, 1); }
    { splitDouble(normals[153], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8204, 1); }
    { splitDouble(normals[2], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 216, 9); }
    { splitDouble(normals[10], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 984, 5); }
    { splitDouble(normals[18], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1488, 6); }
    { splitDouble(normals[26], ((_chunkX * 16 + (_cx + 0) * 4)) * 1500, ((minY + (_cy + 1) * 8)) * 0, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1500, out, 2304, 16); }
    { splitOldBlended(*oldBlendeds[2], (_chunkX * 16 + (_cx + 0) * 4), (minY + (_cy + 1) * 8), (_chunkZ * 16 + (_cz + 0) * 4), out, 4016); }
    { splitDouble(normals[34], ((_chunkX * 16 + (_cx + 0) * 4)) * 0.75, ((minY + (_cy + 1) * 8)) * 0.5, ((_chunkZ * 16 + (_cz + 0) * 4)) * 0.75, out, 5768, 3); }
    { splitDouble(normals[42], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6008, 1); }
    { splitDouble(normals[50], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6104, 1); }
    { splitDouble(normals[58], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 6200, 1); }
    { double _d = ws_scale(0, normals[58].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[66], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 6296, 1); }
    { double _d = ws_scale(0, normals[58].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[74], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 6392, 1); }
    { splitDouble(normals[82], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6488, 1); }
    { splitDouble(normals[90], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 8, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6584, 1); }
    { splitDouble(normals[98], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 0.66666666666666663, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6872, 9); }
    { splitDouble(normals[106], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 7544, 1); }
    { double _d = ws_scale(1, normals[106].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[114], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 7640, 1); }
    { splitDouble(normals[122], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 7736, 1); }
    { splitDouble(normals[130], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 0, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 7832, 1); }
    { splitDouble(normals[138], ((_chunkX * 16 + (_cx + 0) * 4)) * 25, ((minY + (_cy + 1) * 8)) * 0.29999999999999999, ((_chunkZ * 16 + (_cz + 0) * 4)) * 25, out, 7952, 2); }
    { splitDouble(normals[146], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8120, 1); }
    { splitDouble(normals[154], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8216, 1); }
    { splitDouble(normals[3], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 324, 9); }
    { splitDouble(normals[11], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1044, 5); }
    { splitDouble(normals[19], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1560, 6); }
    { splitDouble(normals[27], ((_chunkX * 16 + (_cx + 1) * 4)) * 1500, ((minY + (_cy + 1) * 8)) * 0, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1500, out, 2496, 16); }
    { splitOldBlended(*oldBlendeds[3], (_chunkX * 16 + (_cx + 1) * 4), (minY + (_cy + 1) * 8), (_chunkZ * 16 + (_cz + 0) * 4), out, 4296); }
    { splitDouble(normals[35], ((_chunkX * 16 + (_cx + 1) * 4)) * 0.75, ((minY + (_cy + 1) * 8)) * 0.5, ((_chunkZ * 16 + (_cz + 0) * 4)) * 0.75, out, 5804, 3); }
    { splitDouble(normals[43], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6020, 1); }
    { splitDouble(normals[51], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6116, 1); }
    { splitDouble(normals[59], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 6212, 1); }
    { double _d = ws_scale(0, normals[59].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[67], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 6308, 1); }
    { double _d = ws_scale(0, normals[59].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[75], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 6404, 1); }
    { splitDouble(normals[83], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6500, 1); }
    { splitDouble(normals[91], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 8, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6596, 1); }
    { splitDouble(normals[99], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 0.66666666666666663, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6980, 9); }
    { splitDouble(normals[107], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 7556, 1); }
    { double _d = ws_scale(1, normals[107].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[115], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 7652, 1); }
    { splitDouble(normals[123], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 7748, 1); }
    { splitDouble(normals[131], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 0, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 7844, 1); }
    { splitDouble(normals[139], ((_chunkX * 16 + (_cx + 1) * 4)) * 25, ((minY + (_cy + 1) * 8)) * 0.29999999999999999, ((_chunkZ * 16 + (_cz + 0) * 4)) * 25, out, 7976, 2); }
    { splitDouble(normals[147], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8132, 1); }
    { splitDouble(normals[155], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8228, 1); }
    { splitDouble(normals[4], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 432, 9); }
    { splitDouble(normals[12], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1104, 5); }
    { splitDouble(normals[20], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1632, 6); }
    { splitDouble(normals[28], ((_chunkX * 16 + (_cx + 0) * 4)) * 1500, ((minY + (_cy + 0) * 8)) * 0, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1500, out, 2688, 16); }
    { splitOldBlended(*oldBlendeds[4], (_chunkX * 16 + (_cx + 0) * 4), (minY + (_cy + 0) * 8), (_chunkZ * 16 + (_cz + 1) * 4), out, 4576); }
    { splitDouble(normals[36], ((_chunkX * 16 + (_cx + 0) * 4)) * 0.75, ((minY + (_cy + 0) * 8)) * 0.5, ((_chunkZ * 16 + (_cz + 1) * 4)) * 0.75, out, 5840, 3); }
    { splitDouble(normals[44], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6032, 1); }
    { splitDouble(normals[52], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6128, 1); }
    { splitDouble(normals[60], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 6224, 1); }
    { double _d = ws_scale(0, normals[60].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[68], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 6320, 1); }
    { double _d = ws_scale(0, normals[60].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[76], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 6416, 1); }
    { splitDouble(normals[84], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6512, 1); }
    { splitDouble(normals[92], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 8, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6608, 1); }
    { splitDouble(normals[100], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 0.66666666666666663, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 7088, 9); }
    { splitDouble(normals[108], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 7568, 1); }
    { double _d = ws_scale(1, normals[108].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[116], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 7664, 1); }
    { splitDouble(normals[124], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 7760, 1); }
    { splitDouble(normals[132], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 0, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 7856, 1); }
    { splitDouble(normals[140], ((_chunkX * 16 + (_cx + 0) * 4)) * 25, ((minY + (_cy + 0) * 8)) * 0.29999999999999999, ((_chunkZ * 16 + (_cz + 1) * 4)) * 25, out, 8000, 2); }
    { splitDouble(normals[148], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8144, 1); }
    { splitDouble(normals[156], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8240, 1); }
    { splitDouble(normals[5], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 540, 9); }
    { splitDouble(normals[13], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1164, 5); }
    { splitDouble(normals[21], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1704, 6); }
    { splitDouble(normals[29], ((_chunkX * 16 + (_cx + 1) * 4)) * 1500, ((minY + (_cy + 0) * 8)) * 0, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1500, out, 2880, 16); }
    { splitOldBlended(*oldBlendeds[5], (_chunkX * 16 + (_cx + 1) * 4), (minY + (_cy + 0) * 8), (_chunkZ * 16 + (_cz + 1) * 4), out, 4856); }
    { splitDouble(normals[37], ((_chunkX * 16 + (_cx + 1) * 4)) * 0.75, ((minY + (_cy + 0) * 8)) * 0.5, ((_chunkZ * 16 + (_cz + 1) * 4)) * 0.75, out, 5876, 3); }
    { splitDouble(normals[45], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6044, 1); }
    { splitDouble(normals[53], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6140, 1); }
    { splitDouble(normals[61], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 6236, 1); }
    { double _d = ws_scale(0, normals[61].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[69], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 6332, 1); }
    { double _d = ws_scale(0, normals[61].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[77], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 6428, 1); }
    { splitDouble(normals[85], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6524, 1); }
    { splitDouble(normals[93], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 8, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6620, 1); }
    { splitDouble(normals[101], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 0.66666666666666663, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 7196, 9); }
    { splitDouble(normals[109], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 7580, 1); }
    { double _d = ws_scale(1, normals[109].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[117], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 7676, 1); }
    { splitDouble(normals[125], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 7772, 1); }
    { splitDouble(normals[133], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 0, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 7868, 1); }
    { splitDouble(normals[141], ((_chunkX * 16 + (_cx + 1) * 4)) * 25, ((minY + (_cy + 0) * 8)) * 0.29999999999999999, ((_chunkZ * 16 + (_cz + 1) * 4)) * 25, out, 8024, 2); }
    { splitDouble(normals[149], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8156, 1); }
    { splitDouble(normals[157], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8252, 1); }
    { splitDouble(normals[6], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 648, 9); }
    { splitDouble(normals[14], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1224, 5); }
    { splitDouble(normals[22], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1776, 6); }
    { splitDouble(normals[30], ((_chunkX * 16 + (_cx + 0) * 4)) * 1500, ((minY + (_cy + 1) * 8)) * 0, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1500, out, 3072, 16); }
    { splitOldBlended(*oldBlendeds[6], (_chunkX * 16 + (_cx + 0) * 4), (minY + (_cy + 1) * 8), (_chunkZ * 16 + (_cz + 1) * 4), out, 5136); }
    { splitDouble(normals[38], ((_chunkX * 16 + (_cx + 0) * 4)) * 0.75, ((minY + (_cy + 1) * 8)) * 0.5, ((_chunkZ * 16 + (_cz + 1) * 4)) * 0.75, out, 5912, 3); }
    { splitDouble(normals[46], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6056, 1); }
    { splitDouble(normals[54], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6152, 1); }
    { splitDouble(normals[62], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 6248, 1); }
    { double _d = ws_scale(0, normals[62].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[70], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 6344, 1); }
    { double _d = ws_scale(0, normals[62].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[78], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 6440, 1); }
    { splitDouble(normals[86], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6536, 1); }
    { splitDouble(normals[94], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 8, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6632, 1); }
    { splitDouble(normals[102], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 0.66666666666666663, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 7304, 9); }
    { splitDouble(normals[110], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 7592, 1); }
    { double _d = ws_scale(1, normals[110].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[118], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 7688, 1); }
    { splitDouble(normals[126], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 7784, 1); }
    { splitDouble(normals[134], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 0, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 7880, 1); }
    { splitDouble(normals[142], ((_chunkX * 16 + (_cx + 0) * 4)) * 25, ((minY + (_cy + 1) * 8)) * 0.29999999999999999, ((_chunkZ * 16 + (_cz + 1) * 4)) * 25, out, 8048, 2); }
    { splitDouble(normals[150], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8168, 1); }
    { splitDouble(normals[158], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8264, 1); }
    { splitDouble(normals[7], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 756, 9); }
    { splitDouble(normals[15], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1284, 5); }
    { splitDouble(normals[23], ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 1) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 1) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1848, 6); }
    { splitDouble(normals[31], ((_chunkX * 16 + (_cx + 1) * 4)) * 1500, ((minY + (_cy + 1) * 8)) * 0, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1500, out, 3264, 16); }
    { splitOldBlended(*oldBlendeds[7], (_chunkX * 16 + (_cx + 1) * 4), (minY + (_cy + 1) * 8), (_chunkZ * 16 + (_cz + 1) * 4), out, 5416); }
    { splitDouble(normals[39], ((_chunkX * 16 + (_cx + 1) * 4)) * 0.75, ((minY + (_cy + 1) * 8)) * 0.5, ((_chunkZ * 16 + (_cz + 1) * 4)) * 0.75, out, 5948, 3); }
    { splitDouble(normals[47], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6068, 1); }
    { splitDouble(normals[55], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6164, 1); }
    { splitDouble(normals[63], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 6260, 1); }
    { double _d = ws_scale(0, normals[63].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[71], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 6356, 1); }
    { double _d = ws_scale(0, normals[63].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[79], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 6452, 1); }
    { splitDouble(normals[87], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6548, 1); }
    { splitDouble(normals[95], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 8, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 6644, 1); }
    { splitDouble(normals[103], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 0.66666666666666663, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 7412, 9); }
    { splitDouble(normals[111], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 7604, 1); }
    { double _d = ws_scale(1, normals[111].sample(((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2)); splitDouble(normals[119], ((_chunkX * 16 + (_cx + 1) * 4))/_d, ((minY + (_cy + 1) * 8))/_d, ((_chunkZ * 16 + (_cz + 1) * 4))/_d, out, 7700, 1); }
    { splitDouble(normals[127], ((_chunkX * 16 + (_cx + 1) * 4)) * 2, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2, out, 7796, 1); }
    { splitDouble(normals[135], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 0, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 7892, 1); }
    { splitDouble(normals[143], ((_chunkX * 16 + (_cx + 1) * 4)) * 25, ((minY + (_cy + 1) * 8)) * 0.29999999999999999, ((_chunkZ * 16 + (_cz + 1) * 4)) * 25, out, 8072, 2); }
    { splitDouble(normals[151], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8180, 1); }
    { splitDouble(normals[159], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8276, 1); }
    }
    {
        int _chunkX = floorDiv(x, 16); int _chunkZ = floorDiv(z, 16);
        int _gx = (x) - _chunkX * 16; int _gy = (y) - minY; int _gz = (z) - _chunkZ * 16;
        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;
    { splitDouble(normals[160], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8288, 1); }
    { splitDouble(normals[161], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8300, 1); }
    { splitDouble(normals[162], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8312, 1); }
    { splitDouble(normals[163], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8324, 1); }
    { splitDouble(normals[164], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8336, 1); }
    { splitDouble(normals[165], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8348, 1); }
    { splitDouble(normals[166], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8360, 1); }
    { splitDouble(normals[167], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8372, 1); }
    }
    {
        int _chunkX = floorDiv(x, 16); int _chunkZ = floorDiv(z, 16);
        int _gx = (x) - _chunkX * 16; int _gy = (y) - minY; int _gz = (z) - _chunkZ * 16;
        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;
    { splitDouble(normals[168], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8384, 1); }
    { splitDouble(normals[169], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8396, 1); }
    { splitDouble(normals[170], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8408, 1); }
    { splitDouble(normals[171], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8420, 1); }
    { splitDouble(normals[172], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8432, 1); }
    { splitDouble(normals[173], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8444, 1); }
    { splitDouble(normals[174], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8456, 1); }
    { splitDouble(normals[175], ((_chunkX * 16 + (_cx + 1) * 4)) * 1, ((minY + (_cy + 1) * 8)) * 1, ((_chunkZ * 16 + (_cz + 1) * 4)) * 1, out, 8468, 1); }
    }
    {
        int _chunkX = floorDiv(x, 16); int _chunkZ = floorDiv(z, 16);
        int _gx = (x) - _chunkX * 16; int _gy = (y) - minY; int _gz = (z) - _chunkZ * 16;
        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;
    { splitDouble(normals[176], ((_chunkX * 16 + (_cx + 0) * 4)) * 2.6666666666666665, ((minY + (_cy + 0) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2.6666666666666665, out, 8480, 1); }
    { splitDouble(normals[177], ((_chunkX * 16 + (_cx + 1) * 4)) * 2.6666666666666665, ((minY + (_cy + 0) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2.6666666666666665, out, 8492, 1); }
    { splitDouble(normals[178], ((_chunkX * 16 + (_cx + 0) * 4)) * 2.6666666666666665, ((minY + (_cy + 1) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2.6666666666666665, out, 8504, 1); }
    { splitDouble(normals[179], ((_chunkX * 16 + (_cx + 1) * 4)) * 2.6666666666666665, ((minY + (_cy + 1) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2.6666666666666665, out, 8516, 1); }
    { splitDouble(normals[180], ((_chunkX * 16 + (_cx + 0) * 4)) * 2.6666666666666665, ((minY + (_cy + 0) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2.6666666666666665, out, 8528, 1); }
    { splitDouble(normals[181], ((_chunkX * 16 + (_cx + 1) * 4)) * 2.6666666666666665, ((minY + (_cy + 0) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2.6666666666666665, out, 8540, 1); }
    { splitDouble(normals[182], ((_chunkX * 16 + (_cx + 0) * 4)) * 2.6666666666666665, ((minY + (_cy + 1) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2.6666666666666665, out, 8552, 1); }
    { splitDouble(normals[183], ((_chunkX * 16 + (_cx + 1) * 4)) * 2.6666666666666665, ((minY + (_cy + 1) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2.6666666666666665, out, 8564, 1); }
    }
    {
        int _chunkX = floorDiv(x, 16); int _chunkZ = floorDiv(z, 16);
        int _gx = (x) - _chunkX * 16; int _gy = (y) - minY; int _gz = (z) - _chunkZ * 16;
        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;
    { splitDouble(normals[184], ((_chunkX * 16 + (_cx + 0) * 4)) * 2.6666666666666665, ((minY + (_cy + 0) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2.6666666666666665, out, 8576, 1); }
    { splitDouble(normals[185], ((_chunkX * 16 + (_cx + 1) * 4)) * 2.6666666666666665, ((minY + (_cy + 0) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2.6666666666666665, out, 8588, 1); }
    { splitDouble(normals[186], ((_chunkX * 16 + (_cx + 0) * 4)) * 2.6666666666666665, ((minY + (_cy + 1) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2.6666666666666665, out, 8600, 1); }
    { splitDouble(normals[187], ((_chunkX * 16 + (_cx + 1) * 4)) * 2.6666666666666665, ((minY + (_cy + 1) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2.6666666666666665, out, 8612, 1); }
    { splitDouble(normals[188], ((_chunkX * 16 + (_cx + 0) * 4)) * 2.6666666666666665, ((minY + (_cy + 0) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2.6666666666666665, out, 8624, 1); }
    { splitDouble(normals[189], ((_chunkX * 16 + (_cx + 1) * 4)) * 2.6666666666666665, ((minY + (_cy + 0) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2.6666666666666665, out, 8636, 1); }
    { splitDouble(normals[190], ((_chunkX * 16 + (_cx + 0) * 4)) * 2.6666666666666665, ((minY + (_cy + 1) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2.6666666666666665, out, 8648, 1); }
    { splitDouble(normals[191], ((_chunkX * 16 + (_cx + 1) * 4)) * 2.6666666666666665, ((minY + (_cy + 1) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 1) * 4)) * 2.6666666666666665, out, 8660, 1); }
    }
    }

    // 顶层（角点 0 仅）拆分：只算 grid 缓存命中时 eval_density 非 interp 路径读的 @c0 实例
    // （顶层 spline 坐标 + interp delegate 的 @c0 节点）。interp 的 8 角点三线性已由 grid 缓存覆盖，
    // 无需在当前点重算 —— 行数为整树 split() 的 1/8，显著降低每点 split 开销。
    // 正确性：corner 恒 0（与 split() 的同 cell 角点 0 坐标一致，见 buildInterpGrid);
    // buildInterpGrid 内部的 splitCoord.swap(saved) 会还原本函数填入的 @c0 值，非 interp 路径读数一致。
    void splitTop(int x, int y, int z, float* out) {
    {
        int _chunkX = floorDiv(x, 16); int _chunkZ = floorDiv(z, 16);
        int _gx = (x) - _chunkX * 16; int _gy = (y) - minY; int _gz = (z) - _chunkZ * 16;
        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;
    { splitDouble(normals[0], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 0, 9); }
    { splitDouble(normals[8], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 864, 5); }
    { splitDouble(normals[16], ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0, ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25) * 4.0), (0) * 0 + (0), ((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25 + (shiftNoises.at("minecraft:offset").sample(((((_chunkZ * 16 + (_cz + 0) * 4)) >> 2) << 2) * 0.25, ((((_chunkX * 16 + (_cx + 0) * 4)) >> 2) << 2) * 0.25, 0.0) * 4.0), out, 1344, 6); }
    { splitDouble(normals[24], ((_chunkX * 16 + (_cx + 0) * 4)) * 1500, ((minY + (_cy + 0) * 8)) * 0, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1500, out, 1920, 16); }
    { splitOldBlended(*oldBlendeds[0], (_chunkX * 16 + (_cx + 0) * 4), (minY + (_cy + 0) * 8), (_chunkZ * 16 + (_cz + 0) * 4), out, 3456); }
    { splitDouble(normals[32], ((_chunkX * 16 + (_cx + 0) * 4)) * 0.75, ((minY + (_cy + 0) * 8)) * 0.5, ((_chunkZ * 16 + (_cz + 0) * 4)) * 0.75, out, 5696, 3); }
    { splitDouble(normals[40], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 5984, 1); }
    { splitDouble(normals[48], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6080, 1); }
    { splitDouble(normals[56], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 6176, 1); }
    { double _d = ws_scale(0, normals[56].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[64], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 6272, 1); }
    { double _d = ws_scale(0, normals[56].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[72], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 6368, 1); }
    { splitDouble(normals[80], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6464, 1); }
    { splitDouble(normals[88], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 8, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6560, 1); }
    { splitDouble(normals[96], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 0.66666666666666663, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 6656, 9); }
    { splitDouble(normals[104], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 7520, 1); }
    { double _d = ws_scale(1, normals[104].sample(((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2)); splitDouble(normals[112], ((_chunkX * 16 + (_cx + 0) * 4))/_d, ((minY + (_cy + 0) * 8))/_d, ((_chunkZ * 16 + (_cz + 0) * 4))/_d, out, 7616, 1); }
    { splitDouble(normals[120], ((_chunkX * 16 + (_cx + 0) * 4)) * 2, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2, out, 7712, 1); }
    { splitDouble(normals[128], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 0, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 7808, 1); }
    { splitDouble(normals[136], ((_chunkX * 16 + (_cx + 0) * 4)) * 25, ((minY + (_cy + 0) * 8)) * 0.29999999999999999, ((_chunkZ * 16 + (_cz + 0) * 4)) * 25, out, 7904, 2); }
    { splitDouble(normals[144], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8096, 1); }
    { splitDouble(normals[152], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8192, 1); }
    }
    {
        int _chunkX = floorDiv(x, 16); int _chunkZ = floorDiv(z, 16);
        int _gx = (x) - _chunkX * 16; int _gy = (y) - minY; int _gz = (z) - _chunkZ * 16;
        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;
    { splitDouble(normals[160], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8288, 1); }
    }
    {
        int _chunkX = floorDiv(x, 16); int _chunkZ = floorDiv(z, 16);
        int _gx = (x) - _chunkX * 16; int _gy = (y) - minY; int _gz = (z) - _chunkZ * 16;
        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;
    { splitDouble(normals[168], ((_chunkX * 16 + (_cx + 0) * 4)) * 1, ((minY + (_cy + 0) * 8)) * 1, ((_chunkZ * 16 + (_cz + 0) * 4)) * 1, out, 8384, 1); }
    }
    {
        int _chunkX = floorDiv(x, 16); int _chunkZ = floorDiv(z, 16);
        int _gx = (x) - _chunkX * 16; int _gy = (y) - minY; int _gz = (z) - _chunkZ * 16;
        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;
    { splitDouble(normals[176], ((_chunkX * 16 + (_cx + 0) * 4)) * 2.6666666666666665, ((minY + (_cy + 0) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2.6666666666666665, out, 8480, 1); }
    }
    {
        int _chunkX = floorDiv(x, 16); int _chunkZ = floorDiv(z, 16);
        int _gx = (x) - _chunkX * 16; int _gy = (y) - minY; int _gz = (z) - _chunkZ * 16;
        int _cx = _gx / 4; int _cy = _gy / 8; int _cz = _gz / 4;
    { splitDouble(normals[184], ((_chunkX * 16 + (_cx + 0) * 4)) * 2.6666666666666665, ((minY + (_cy + 0) * 8)) * 2.6666666666666665, ((_chunkZ * 16 + (_cz + 0) * 4)) * 2.6666666666666665, out, 8576, 1); }
    }
    }

    void collectPerm(std::vector<uint32_t>& perm) {
        perm.assign((size_t)permSize, 0);
        for (int i = 0; i < (int)oldBlendeds.size(); i++) {
            for (int r = 0; r < 16; r++) {
                const wg::PerlinNoiseSampler* pn = oldBlendeds[i]->lower.getOctave(r);
                if (pn) for (int j = 0; j < 256; j++) perm[(size_t)(oldBase[i] + r) * 256 + j] = (uint32_t)pn->permutation[j];
                pn = oldBlendeds[i]->upper.getOctave(r);
                if (pn) for (int j = 0; j < 256; j++) perm[(size_t)(oldBase[i] + 16 + r) * 256 + j] = (uint32_t)pn->permutation[j];
            }
            for (int q = 0; q < 8; q++) {
                const wg::PerlinNoiseSampler* pn = oldBlendeds[i]->interpolation.getOctave(q);
                if (pn) for (int j = 0; j < 256; j++) perm[(size_t)(oldBase[i] + 32 + q) * 256 + j] = (uint32_t)pn->permutation[j];
            }
        }
        for (int i = 0; i < (int)normals.size(); i++) {
            for (int k = 0; k < n[i]; k++) {
                const wg::PerlinNoiseSampler* pn = normals[i].firstSampler.octaveSamplers[k].get();
                if (pn) for (int j = 0; j < 256; j++) perm[(size_t)(octBase[i] + k) * 256 + j] = (uint32_t)pn->permutation[j];
                pn = normals[i].secondSampler.octaveSamplers[k].get();
                if (pn) for (int j = 0; j < 256; j++) perm[(size_t)(octBase[i] + n[i] + k) * 256 + j] = (uint32_t)pn->permutation[j];
            }
        }
    }
    // ===== DFC CPU 采样函数（无虚调用直排求值；形态 A：val[i]=节点索引）=====
    // C 运行时缓冲（采样函数读；由 split()/collectPerm() 填充）
    // splitCoord 必须 per-thread（buildInterpGrid 的 swap 与采样线程共享实例成员会冲突）；static + inline（multi-TU LNK2005）。
    inline static thread_local std::vector<float> splitCoord;
    std::vector<uint32_t> perm;

    // ---- A 数据表（镜像 GLSL const，C++17 inline）----
    static constexpr double GRADIENTS[16][3] = {
        { 1,  1,  0}, {-1,  1,  0}, { 1, -1,  0}, {-1, -1,  0},
        { 1,  0,  1}, {-1,  0,  1}, { 1,  0, -1}, {-1,  0, -1},
        { 0,  1,  1}, { 0, -1,  1}, { 0,  1, -1}, { 0, -1, -1},
        { 1,  1,  0}, { 0, -1,  1}, {-1,  1,  0}, { 0, -1, -1}
    };
    static const int DF_NODES = 163;
    static constexpr int DF_TYPE[163] = {0, 0, 18, 0, 0, 18, 0, 0, 18, 0, 0, 0, 7, 6, 7, 0, 4, 6, 7, 6, 21, 6, 4, 6, 7, 6, 21, 2, 13, 7, 6, 0, 0, 4, 6, 7, 6, 21, 7, 14, 7, 3, 6, 0, 0, 2, 6, 18, 6, 0, 2, 7, 6, 0, 2, 10, 6, 7, 2, 22, 22, 9, 0, 0, 2, 7, 6, 6, 16, 6, 8, 7, 8, 2, 11, 7, 0, 2, 6, 16, 0, 0, 7, 6, 16, 6, 6, 8, 2, 22, 0, 0, 0, 2, 7, 6, 7, 6, 0, 2, 7, 6, 18, 6, 10, 6, 12, 9, 16, 6, 8, 0, 2, 7, 2, 7, 6, 6, 0, 2, 7, 6, 12, 7, 0, 17, 9, 17, 6, 7, 6, 6, 7, 6, 20, 5, 7, 15, 1, 2, 17, 5, 0, 0, 0, 2, 7, 6, 17, 5, 2, 17, 5, 10, 2, 17, 5, 10, 9, 7, 6, 17, 8};
    static constexpr int DF_A1[163] = {-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, 10, 9, -1, 28, 15, 17, 14, 19, 8, 36, 9, 10, 9, 25, 3, 27, 26, 21, -1, -1, 55, 32, 10, 31, 36, 30, 38, 7, 4, 40, -1, -1, 5, 44, -1, 46, -1, 6, 49, 49, -1, 7, 54, 53, 52, 8, 58, 58, 59, -1, -1, 11, 63, 62, 61, 67, 57, 48, 43, 42, 12, 73, 7, -1, 13, 76, 78, -1, -1, 81, 80, 83, 79, 75, 86, 14, 88, -1, -1, -1, 16, 92, 91, 90, 89, -1, 17, 98, 9, -1, 101, 103, 104, 105, 97, 107, 108, 87, -1, 18, 111, 19, 11, 11, 113, -1, 20, 118, 118, 121, 117, -1, 123, 110, 42, 6, 5, 4, 3, 2, 1, 133, 0, 0, 136, -1, 21, 138, 1, -1, -1, -1, 22, 144, 143, 138, 2, 23, 138, 3, 152, 24, 138, 4, 156, 153, 80, 149, 141, 137};
    static constexpr int DF_A2[163] = {-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10, 12, 13, -1, 1, 16, 10, 18, -1, 20, 1, 22, 23, 24, -1, -1, -1, 28, 29, -1, -1, 1, 33, 34, 35, -1, 37, -1, 39, -1, 41, -1, -1, -1, 45, -1, 47, -1, -1, 50, 51, -1, -1, -1, 55, 56, -1, 9, 10, 60, -1, -1, -1, 64, 65, 66, -1, 68, 69, 70, 71, -1, -1, 74, -1, -1, 77, -1, -1, -1, 42, 82, -1, 84, 85, 70, -1, 15, -1, -1, -1, -1, 93, 94, 95, 96, -1, -1, 99, 100, -1, 102, -1, 95, -1, 106, -1, 57, 109, -1, -1, 112, -1, 114, 115, 116, -1, -1, 119, 120, -1, 122, -1, 124, 125, 72, 127, 128, 129, 130, 131, 132, -1, -1, 135, -1, -1, -1, 139, -1, -1, -1, -1, -1, 145, 146, 147, -1, -1, 150, -1, -1, -1, 154, -1, -1, 157, 158, 159, 142, 161};
    static constexpr int DF_A3[163] = {-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 123, -1, 126, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, -1, -1, -1, -1, -1, -1, -1, 9, -1, -1, 9, -1, -1, -1, 9, -1, -1, -1, -1, -1, 160, -1};
    static constexpr float DF_F0[163] = {0.64000000000000001f, 0.1171875f, -64.0f, -0.1171875f, -0.078125f, 240.0f, 0.078125f, 4.0f, -64.0f, 0.0f, 1.0f, -1.0f, 0.0f, 0.0f, 0.0f, -0.5037500262260437f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 10.0f, -10.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 5.0f, 0.37f, 0.0f, 0.0f, -10.0f, 0.0f, -0.050000000000000003f, 0.0f, 0.0f, 0.0f, -0.40000000000000002f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -0.076499999999999999f, -0.011499999999999996f, 0.0f, 0.0f, 0.0f, 0.0f, -1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.27000000000000002f, 0.0f, 0.0f, -1.0f, 1.5f, -0.64000000000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.083000000000000004f, -0.94999999999999996f, -0.35000000000000003f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 8.0f, 0.0f, 0.0f, 0.0f, -64.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -1.0f, 0.0f, 0.0f, 2.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.55000000000000004f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -1000000.0f, -1000000.0f, 0.0f, -1000000.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -60.0f, 0.0f, 64.0f, -0.075000000000000011f, -0.025000000000000001f, 0.0f, 0.0f, 0.0f, -60.0f, 0.0f, 0.0f, -60.0f, 0.0f, 0.0f, 0.0f, -60.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -1000000.0f, 0.0f};
    static constexpr float DF_F1[163] = {0.0f, 0.0f, -40.0f, 0.0f, 0.0f, 256.0f, 0.0f, 0.0f, 320.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 30.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 320.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.029999999999999999f, 0.0f, 1.5625f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 321.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 321.0f, 0.0f, 0.0f, 321.0f, 0.0f, 0.0f, 0.0f, 321.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    static constexpr float DF_F2[163] = {0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 1.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.29999999999999999f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 8.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    static constexpr float DF_F3[163] = {0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -1.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -40.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    static const int TOP_ROOT = 162;          // 形态 A：根节点索引 = DF_NODES-1
    static const int N_INTERP = 5;
    static constexpr int INTERP_ROOTS[5] = {134, 140, 148, 151, 155};   // 每 interp 的 delegate 根节点索引
    static constexpr int NOISE_SLOT_COUNT = 25;
    static constexpr int NOISE_SLOT_BASE[25] = {0, 8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128, 136, 144, 152, 160, 168, 176, 184, 192};
    static constexpr int NOISE_SLOT_STRIDE[25] = {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1};
    static constexpr int COORD_SLOT_TABLE[4] = {0, 1, 2, 2};
    static const int NORMAL_INSTANCES = 200;
    static constexpr int NORMAL_PACK[600] = {9, 0, 0, 9, 18, 108, 9, 36, 216, 9, 54, 324, 9, 72, 432, 9, 90, 540, 9, 108, 648, 9, 126, 756, 5, 144, 864, 5, 154, 924, 5, 164, 984, 5, 174, 1044, 5, 184, 1104, 5, 194, 1164, 5, 204, 1224, 5, 214, 1284, 6, 224, 1344, 6, 236, 1416, 6, 248, 1488, 6, 260, 1560, 6, 272, 1632, 6, 284, 1704, 6, 296, 1776, 6, 308, 1848, 16, 320, 1920, 16, 352, 2112, 16, 384, 2304, 16, 416, 2496, 16, 448, 2688, 16, 480, 2880, 16, 512, 3072, 16, 544, 3264, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 896, 5696, 3, 902, 5732, 3, 908, 5768, 3, 914, 5804, 3, 920, 5840, 3, 926, 5876, 3, 932, 5912, 3, 938, 5948, 1, 944, 5984, 1, 946, 5996, 1, 948, 6008, 1, 950, 6020, 1, 952, 6032, 1, 954, 6044, 1, 956, 6056, 1, 958, 6068, 1, 960, 6080, 1, 962, 6092, 1, 964, 6104, 1, 966, 6116, 1, 968, 6128, 1, 970, 6140, 1, 972, 6152, 1, 974, 6164, 1, 976, 6176, 1, 978, 6188, 1, 980, 6200, 1, 982, 6212, 1, 984, 6224, 1, 986, 6236, 1, 988, 6248, 1, 990, 6260, 1, 992, 6272, 1, 994, 6284, 1, 996, 6296, 1, 998, 6308, 1, 1000, 6320, 1, 1002, 6332, 1, 1004, 6344, 1, 1006, 6356, 1, 1008, 6368, 1, 1010, 6380, 1, 1012, 6392, 1, 1014, 6404, 1, 1016, 6416, 1, 1018, 6428, 1, 1020, 6440, 1, 1022, 6452, 1, 1024, 6464, 1, 1026, 6476, 1, 1028, 6488, 1, 1030, 6500, 1, 1032, 6512, 1, 1034, 6524, 1, 1036, 6536, 1, 1038, 6548, 1, 1040, 6560, 1, 1042, 6572, 1, 1044, 6584, 1, 1046, 6596, 1, 1048, 6608, 1, 1050, 6620, 1, 1052, 6632, 1, 1054, 6644, 9, 1056, 6656, 9, 1074, 6764, 9, 1092, 6872, 9, 1110, 6980, 9, 1128, 7088, 9, 1146, 7196, 9, 1164, 7304, 9, 1182, 7412, 1, 1200, 7520, 1, 1202, 7532, 1, 1204, 7544, 1, 1206, 7556, 1, 1208, 7568, 1, 1210, 7580, 1, 1212, 7592, 1, 1214, 7604, 1, 1216, 7616, 1, 1218, 7628, 1, 1220, 7640, 1, 1222, 7652, 1, 1224, 7664, 1, 1226, 7676, 1, 1228, 7688, 1, 1230, 7700, 1, 1232, 7712, 1, 1234, 7724, 1, 1236, 7736, 1, 1238, 7748, 1, 1240, 7760, 1, 1242, 7772, 1, 1244, 7784, 1, 1246, 7796, 1, 1248, 7808, 1, 1250, 7820, 1, 1252, 7832, 1, 1254, 7844, 1, 1256, 7856, 1, 1258, 7868, 1, 1260, 7880, 1, 1262, 7892, 2, 1264, 7904, 2, 1268, 7928, 2, 1272, 7952, 2, 1276, 7976, 2, 1280, 8000, 2, 1284, 8024, 2, 1288, 8048, 2, 1292, 8072, 1, 1296, 8096, 1, 1298, 8108, 1, 1300, 8120, 1, 1302, 8132, 1, 1304, 8144, 1, 1306, 8156, 1, 1308, 8168, 1, 1310, 8180, 1, 1312, 8192, 1, 1314, 8204, 1, 1316, 8216, 1, 1318, 8228, 1, 1320, 8240, 1, 1322, 8252, 1, 1324, 8264, 1, 1326, 8276, 1, 1328, 8288, 1, 1330, 8300, 1, 1332, 8312, 1, 1334, 8324, 1, 1336, 8336, 1, 1338, 8348, 1, 1340, 8360, 1, 1342, 8372, 1, 1344, 8384, 1, 1346, 8396, 1, 1348, 8408, 1, 1350, 8420, 1, 1352, 8432, 1, 1354, 8444, 1, 1356, 8456, 1, 1358, 8468, 1, 1360, 8480, 1, 1362, 8492, 1, 1364, 8504, 1, 1366, 8516, 1, 1368, 8528, 1, 1370, 8540, 1, 1372, 8552, 1, 1374, 8564, 1, 1376, 8576, 1, 1378, 8588, 1, 1380, 8600, 1, 1382, 8612, 1, 1384, 8624, 1, 1386, 8636, 1, 1388, 8648, 1, 1390, 8660};
    static constexpr float NORMAL_PACK_F[400] = {0.50097847358121328f, 1.4999999999999998f, 0.50097847358121328f, 1.4999999999999998f, 0.50097847358121328f, 1.4999999999999998f, 0.50097847358121328f, 1.4999999999999998f, 0.50097847358121328f, 1.4999999999999998f, 0.50097847358121328f, 1.4999999999999998f, 0.50097847358121328f, 1.4999999999999998f, 0.50097847358121328f, 1.4999999999999998f, 0.5161290322580645f, 1.3888888888888888f, 0.5161290322580645f, 1.3888888888888888f, 0.5161290322580645f, 1.3888888888888888f, 0.5161290322580645f, 1.3888888888888888f, 0.5161290322580645f, 1.3888888888888888f, 0.5161290322580645f, 1.3888888888888888f, 0.5161290322580645f, 1.3888888888888888f, 0.5161290322580645f, 1.3888888888888888f, 0.50793650793650791f, 1.25f, 0.50793650793650791f, 1.25f, 0.50793650793650791f, 1.25f, 0.50793650793650791f, 1.25f, 0.50793650793650791f, 1.25f, 0.50793650793650791f, 1.25f, 0.50793650793650791f, 1.25f, 0.50793650793650791f, 1.25f, 0.50000762951094835f, 1.5686274509803919f, 0.50000762951094835f, 1.5686274509803919f, 0.50000762951094835f, 1.5686274509803919f, 0.50000762951094835f, 1.5686274509803919f, 0.50000762951094835f, 1.5686274509803919f, 0.50000762951094835f, 1.5686274509803919f, 0.50000762951094835f, 1.5686274509803919f, 0.50000762951094835f, 1.5686274509803919f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.5714285714285714f, 1.25f, 0.5714285714285714f, 1.25f, 0.5714285714285714f, 1.25f, 0.5714285714285714f, 1.25f, 0.5714285714285714f, 1.25f, 0.5714285714285714f, 1.25f, 0.5714285714285714f, 1.25f, 0.5714285714285714f, 1.25f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 0.50097847358121328f, 1.4814814814814814f, 0.50097847358121328f, 1.4814814814814814f, 0.50097847358121328f, 1.4814814814814814f, 0.50097847358121328f, 1.4814814814814814f, 0.50097847358121328f, 1.4814814814814814f, 0.50097847358121328f, 1.4814814814814814f, 0.50097847358121328f, 1.4814814814814814f, 0.50097847358121328f, 1.4814814814814814f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 0.66666666666666663f, 1.1111111111111109f, 0.66666666666666663f, 1.1111111111111109f, 0.66666666666666663f, 1.1111111111111109f, 0.66666666666666663f, 1.1111111111111109f, 0.66666666666666663f, 1.1111111111111109f, 0.66666666666666663f, 1.1111111111111109f, 0.66666666666666663f, 1.1111111111111109f, 0.66666666666666663f, 1.1111111111111109f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f, 1.0f, 0.83333333333333326f};
    static constexpr float NORMAL_AMPS[536] = {1.0f, 1.0f, 2.0f, 2.0f, 2.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 2.0f, 2.0f, 2.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 2.0f, 2.0f, 2.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 2.0f, 2.0f, 2.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 2.0f, 2.0f, 2.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 2.0f, 2.0f, 2.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 2.0f, 2.0f, 2.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 2.0f, 2.0f, 2.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 0.0f, 1.0f, 1.0f, 1.0f, 1.0f, 0.0f, 1.0f, 1.0f, 1.0f, 1.0f, 0.0f, 1.0f, 1.0f, 1.0f, 1.0f, 0.0f, 1.0f, 1.0f, 1.0f, 1.0f, 0.0f, 1.0f, 1.0f, 1.0f, 1.0f, 0.0f, 1.0f, 1.0f, 1.0f, 1.0f, 0.0f, 1.0f, 1.0f, 1.0f, 1.0f, 0.0f, 1.0f, 1.0f, 1.0f, 2.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 2.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 2.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 2.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 2.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 2.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 2.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 2.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 0.40000000000000002f, 0.5f, 1.0f, 0.40000000000000002f, 0.5f, 1.0f, 0.40000000000000002f, 0.5f, 1.0f, 0.40000000000000002f, 0.5f, 1.0f, 0.40000000000000002f, 0.5f, 1.0f, 0.40000000000000002f, 0.5f, 1.0f, 0.40000000000000002f, 0.5f, 1.0f, 0.40000000000000002f, 0.5f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 0.5f, 1.0f, 2.0f, 1.0f, 2.0f, 1.0f, 0.0f, 2.0f, 0.0f, 0.5f, 1.0f, 2.0f, 1.0f, 2.0f, 1.0f, 0.0f, 2.0f, 0.0f, 0.5f, 1.0f, 2.0f, 1.0f, 2.0f, 1.0f, 0.0f, 2.0f, 0.0f, 0.5f, 1.0f, 2.0f, 1.0f, 2.0f, 1.0f, 0.0f, 2.0f, 0.0f, 0.5f, 1.0f, 2.0f, 1.0f, 2.0f, 1.0f, 0.0f, 2.0f, 0.0f, 0.5f, 1.0f, 2.0f, 1.0f, 2.0f, 1.0f, 0.0f, 2.0f, 0.0f, 0.5f, 1.0f, 2.0f, 1.0f, 2.0f, 1.0f, 0.0f, 2.0f, 0.0f, 0.5f, 1.0f, 2.0f, 1.0f, 2.0f, 1.0f, 0.0f, 2.0f, 0.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f, 1.0f};
    static constexpr int NORMAL_AMP_OFF[200] = {0, 9, 18, 27, 36, 45, 54, 63, 72, 77, 82, 87, 92, 97, 102, 107, 112, 118, 124, 130, 136, 142, 148, 154, 160, 176, 192, 208, 224, 240, 256, 272, 288, 288, 288, 288, 288, 288, 288, 288, 288, 291, 294, 297, 300, 303, 306, 309, 312, 313, 314, 315, 316, 317, 318, 319, 320, 321, 322, 323, 324, 325, 326, 327, 328, 329, 330, 331, 332, 333, 334, 335, 336, 337, 338, 339, 340, 341, 342, 343, 344, 345, 346, 347, 348, 349, 350, 351, 352, 353, 354, 355, 356, 357, 358, 359, 360, 361, 362, 363, 364, 365, 366, 367, 368, 377, 386, 395, 404, 413, 422, 431, 440, 441, 442, 443, 444, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 455, 456, 457, 458, 459, 460, 461, 462, 463, 464, 465, 466, 467, 468, 469, 470, 471, 472, 474, 476, 478, 480, 482, 484, 486, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 500, 501, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511, 512, 513, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529, 530, 531, 532, 533, 534, 535};
    static const int OLD_INSTANCES = 200;
    static constexpr int OLD_PACK[400] = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 576, 3456, 616, 3736, 656, 4016, 696, 4296, 736, 4576, 776, 4856, 816, 5136, 856, 5416, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};

    // ---- DFC 闭包（每 interp delegate + 顶层；D25：各用闭包子集，消除孤儿 delegate 死计算）----
    // 逐 interp 扁平拼接：CLOSURE_OFF[k]/CLOSURE_LEN[k]/CLOSURE_ROOT_POS[k] 选取第 k 个 interp 的闭包，
    // CLOSURE_VAL_SLOTS[k] = 该闭包的 val 槽数（liveness 峰值），CLOSURE_MAX_SLOTS = 全闭包最大槽数
    // （eval_df_base 的 val[] 上界）。闭包内节点用线性槽（CLOSURE_SLOT 映射闭包内位置→槽）；子节点
    // a1/a2/a3 已是闭包内位置（map_a）。顶层闭包（eval_df 用）单独 CLOSURE_T/TOP_* 系列。数据与
    // eval_df_glsl 的 CTYPE_N/CAx_N/CFx_N/SLOT_OF_N + CTYPE_T/... 严格同源（同一 _compute_val_layout）。
    static const int N_CLOSURE = 5;
    static constexpr int CLOSURE_OFF[5] = {0, 134, 155, 175, 192};
    static constexpr int CLOSURE_LEN[5] = {134, 21, 20, 17, 18};
    static constexpr int CLOSURE_VAL_SLOTS[5] = {18, 7, 6, 6, 6};
    static constexpr int CLOSURE_ROOT_POS[5] = {133, 20, 19, 16, 17};
    static const int CLOSURE_MAX_SLOTS = 18;
    static constexpr int CLOSURE_TYPE[210] = {0, 18, 0, 0, 18, 0, 0, 18, 0, 0, 0, 7, 6, 7, 0, 4, 6, 7, 6, 21, 6, 4, 6, 7, 6, 21, 2, 13, 7, 6, 0, 0, 4, 6, 7, 6, 21, 7, 14, 7, 3, 6, 0, 0, 2, 6, 18, 6, 0, 2, 7, 6, 0, 2, 10, 6, 7, 2, 22, 22, 9, 0, 0, 2, 7, 6, 6, 16, 6, 8, 7, 8, 2, 11, 7, 0, 2, 6, 16, 0, 0, 7, 6, 16, 6, 6, 8, 2, 22, 0, 0, 0, 2, 7, 6, 7, 6, 0, 2, 7, 6, 18, 6, 10, 6, 12, 9, 16, 6, 8, 0, 2, 7, 2, 7, 6, 6, 0, 2, 7, 6, 12, 7, 0, 17, 9, 17, 6, 7, 6, 6, 7, 6, 20, 0, 0, 18, 0, 0, 0, 7, 6, 7, 0, 4, 6, 7, 6, 21, 6, 2, 13, 1, 2, 17, 0, 0, 0, 0, 4, 0, 0, 4, 6, 7, 6, 2, 10, 1, 0, 0, 2, 7, 6, 17, 0, 0, 0, 0, 4, 6, 0, 0, 4, 6, 7, 6, 2, 10, 1, 2, 17, 0, 0, 0, 0, 4, 6, 7, 0, 0, 4, 6, 7, 6, 2, 10, 1, 2, 17};
    static constexpr int CLOSURE_A1[210] = {-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10, 9, 8, -1, 28, 14, 16, 13, 18, 7, 36, 8, 9, 8, 24, 3, 26, 25, 20, -1, -1, 55, 31, 9, 30, 35, 29, 37, 6, 4, 39, -1, -1, 5, 43, -1, 45, -1, 6, 48, 48, -1, 7, 53, 52, 51, 8, 57, 57, 58, -1, -1, 11, 62, 61, 60, 66, 56, 47, 42, 41, 12, 72, 6, -1, 13, 75, 77, -1, -1, 80, 79, 82, 78, 74, 85, 14, 87, -1, -1, -1, 16, 91, 90, 89, 88, -1, 17, 97, 8, -1, 100, 102, 103, 104, 96, 106, 107, 86, -1, 18, 110, 19, 10, 10, 112, -1, 20, 117, 117, 120, 116, -1, 122, 109, 41, 5, 4, 3, 2, 1, 0, 132, -1, -1, -1, -1, -1, -1, 5, 4, 3, -1, 28, 9, 11, 8, 13, 2, 3, 16, -1, 21, 18, -1, -1, -1, -1, 36, -1, -1, 55, 6, 3, 5, 7, 11, -1, -1, -1, 22, 15, 14, 13, -1, -1, -1, -1, 36, 2, -1, -1, 55, 7, 3, 6, 7, 12, -1, 23, 14, -1, -1, -1, -1, 36, 2, 3, -1, -1, 55, 8, 3, 7, 7, 13, -1, 24, 15};
    static constexpr int CLOSURE_A2[210] = {-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 9, 11, 12, -1, 1, 15, 9, 17, -1, 19, 1, 21, 22, 23, -1, -1, -1, 27, 28, -1, -1, 1, 32, 33, 34, -1, 36, -1, 38, -1, 40, -1, -1, -1, 44, -1, 46, -1, -1, 49, 50, -1, -1, -1, 54, 55, -1, 9, 10, 59, -1, -1, -1, 63, 64, 65, -1, 67, 68, 69, 70, -1, -1, 73, -1, -1, 76, -1, -1, -1, 41, 81, -1, 83, 84, 69, -1, 15, -1, -1, -1, -1, 92, 93, 94, 95, -1, -1, 98, 99, -1, 101, -1, 94, -1, 105, -1, 56, 108, -1, -1, 111, -1, 113, 114, 115, -1, -1, 118, 119, -1, 121, -1, 123, 124, 71, 126, 127, 128, 129, 130, 131, -1, -1, -1, -1, -1, -1, -1, 4, 6, 7, -1, 1, 10, 4, 12, -1, 14, -1, -1, -1, -1, 19, -1, -1, -1, -1, 1, -1, -1, 1, 7, 8, 9, -1, -1, -1, -1, -1, -1, 16, 17, 18, -1, -1, -1, -1, 1, 4, -1, -1, 1, 8, 9, 10, -1, -1, -1, -1, 15, -1, -1, -1, -1, 1, 4, 5, -1, -1, 1, 9, 10, 11, -1, -1, -1, -1, 16};
    static constexpr int CLOSURE_A3[210] = {-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 122, -1, 125, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2};
    static constexpr float CLOSURE_F0[210] = {0.1171875f, -64.0f, -0.1171875f, -0.078125f, 240.0f, 0.078125f, 4.0f, -64.0f, 0.0f, 1.0f, -1.0f, 0.0f, 0.0f, 0.0f, -0.5037500262260437f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 10.0f, -10.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 5.0f, 0.37f, 0.0f, 0.0f, -10.0f, 0.0f, -0.050000000000000003f, 0.0f, 0.0f, 0.0f, -0.40000000000000002f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -0.076499999999999999f, -0.011499999999999996f, 0.0f, 0.0f, 0.0f, 0.0f, -1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.27000000000000002f, 0.0f, 0.0f, -1.0f, 1.5f, -0.64000000000000001f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.083000000000000004f, -0.94999999999999996f, -0.35000000000000003f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 8.0f, 0.0f, 0.0f, 0.0f, -64.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -1.0f, 0.0f, 0.0f, 2.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.55000000000000004f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -1000000.0f, -1000000.0f, 0.0f, -1000000.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.1171875f, -0.1171875f, -64.0f, 0.0f, 1.0f, -1.0f, 0.0f, 0.0f, 0.0f, -0.5037500262260437f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -60.0f, 0.1171875f, 4.0f, 0.0f, 1.0f, 0.0f, 10.0f, -10.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -0.075000000000000011f, -0.025000000000000001f, 0.0f, 0.0f, 0.0f, -60.0f, 0.1171875f, 4.0f, 0.0f, 1.0f, 0.0f, 0.0f, 10.0f, -10.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -60.0f, 0.1171875f, 4.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 10.0f, -10.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -60.0f};
    static constexpr float CLOSURE_F1[210] = {0.0f, -40.0f, 0.0f, 0.0f, 256.0f, 0.0f, 0.0f, 320.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 30.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 320.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.029999999999999999f, 0.0f, 1.5625f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 320.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 321.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 321.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 321.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 321.0f};
    static constexpr float CLOSURE_F2[210] = {0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 1.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.29999999999999999f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 8.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    static constexpr float CLOSURE_F3[210] = {0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -1.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -40.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -1.5f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    static constexpr int CLOSURE_SLOT[210] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 11, 12, 13, 14, 12, 13, 11, 12, 7, 11, 7, 11, 7, 11, 13, 11, 7, 11, 12, 13, 14, 12, 9, 11, 9, 7, 9, 7, 11, 7, 9, 12, 13, 9, 12, 9, 13, 14, 13, 9, 14, 15, 14, 9, 13, 14, 15, 13, 14, 15, 16, 17, 15, 14, 13, 14, 13, 12, 7, 12, 14, 12, 6, 14, 15, 6, 14, 15, 16, 15, 14, 15, 6, 12, 6, 13, 6, 14, 15, 16, 17, 15, 14, 6, 13, 14, 16, 13, 8, 14, 8, 13, 8, 13, 6, 8, 6, 8, 9, 12, 8, 9, 8, 9, 8, 10, 12, 10, 8, 10, 8, 9, 8, 6, 7, 5, 4, 3, 2, 1, 0, 0, 0, 0, 1, 2, 3, 4, 5, 4, 1, 5, 6, 1, 2, 1, 2, 0, 1, 0, 1, 2, 0, 0, 0, 1, 2, 2, 3, 4, 5, 3, 1, 1, 2, 1, 2, 3, 4, 5, 3, 2, 0, 0, 0, 1, 2, 3, 2, 3, 4, 5, 3, 1, 1, 2, 1, 2, 3, 0, 0, 0, 1, 2, 3, 2, 2, 3, 4, 5, 3, 1, 1, 2, 1, 2, 3};
    static const int TOP_CLOSURE_LEN = 21;
    static const int VAL_SLOTS_TOP = 8;
    static const int TOP_ROOT_POS = 20;
    static constexpr int TOP_TYPE[21] = {0, 0, 18, 0, 0, 0, 5, 7, 15, 5, 0, 5, 5, 10, 5, 10, 9, 7, 6, 17, 8};
    static constexpr int TOP_A1[21] = {-1, -1, -1, -1, -1, -1, 0, 0, 7, 1, -1, 2, 3, 12, 4, 14, 13, 5, 11, 9, 8};
    static constexpr int TOP_A2[21] = {-1, -1, -1, -1, -1, -1, -1, 6, -1, -1, -1, -1, -1, -1, -1, -1, 15, 16, 17, 10, 19};
    static constexpr int TOP_A3[21] = {-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 18, -1};
    static constexpr float TOP_F0[21] = {0.64000000000000001f, 0.1171875f, -64.0f, -0.1171875f, -0.078125f, 1.5f, 0.0f, 0.0f, 0.0f, 0.0f, 64.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, -1000000.0f, 0.0f};
    static constexpr float TOP_F1[21] = {0.0f, 0.0f, -40.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    static constexpr float TOP_F2[21] = {0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    static constexpr float TOP_F3[21] = {0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    static constexpr int TOP_SLOT[21] = {0, 1, 1, 1, 1, 1, 2, 3, 0, 2, 3, 4, 5, 6, 5, 7, 5, 6, 1, 4, 1};
    // ---- 原语（float32 语义；CpuBackend.ws_scale 为 double 版，此处 float 版区分）----
    static float ws_scaleF(int kind, float v) {
        if (kind == 1) {
            if (v < -0.75f) return 0.5f;
            if (v < -0.5f) return 0.75f;
            if (v < 0.5f) return 1.0f;
            return v < 0.75f ? 2.0f : 3.0f;
        }
        if (v < -0.5f) return 0.75f;
        if (v < 0.0f) return 1.0f;
        return v < 0.5f ? 1.5f : 2.0f;
    }
    int mapPermD(int octBase, int v) { return (int)perm[(size_t)octBase * 256 + (v & 255)]; }
    static float perlinFadeF(float v) { return v * v * v * (v * (v * 6.0f - 15.0f) + 10.0f); }
    static float lerpF(float d, float s, float e) { return s + d * (e - s); }
    float gradDotF(int hash, float x, float y, float z) {
        float gx = (float)GRADIENTS[hash & 15][0];
        float gy = (float)GRADIENTS[hash & 15][1];
        float gz = (float)GRADIENTS[hash & 15][2];
        return gx * x + gy * y + gz * z;
    }
    float pn_sample3_f32(int octBase, int sx, int sy, int sz, float lx, float ly, float lz) {
        int i0 = mapPermD(octBase, sx); int j = mapPermD(octBase, sx + 1);
        int k = mapPermD(octBase, i0 + sy); int l = mapPermD(octBase, i0 + sy + 1);
        int m = mapPermD(octBase, j + sy); int nn = mapPermD(octBase, j + sy + 1);
        float d  = gradDotF(mapPermD(octBase, k + sz),     lx,     ly,     lz);
        float e  = gradDotF(mapPermD(octBase, m + sz),     lx - 1.0f, ly,     lz);
        float f  = gradDotF(mapPermD(octBase, l + sz),     lx,     ly - 1.0f, lz);
        float g  = gradDotF(mapPermD(octBase, nn + sz),    lx - 1.0f, ly - 1.0f, lz);
        float h  = gradDotF(mapPermD(octBase, k + sz + 1), lx,     ly,     lz - 1.0f);
        float o  = gradDotF(mapPermD(octBase, m + sz + 1), lx - 1.0f, ly,     lz - 1.0f);
        float p  = gradDotF(mapPermD(octBase, l + sz + 1), lx,     ly - 1.0f, lz - 1.0f);
        float q  = gradDotF(mapPermD(octBase, nn + sz + 1), lx - 1.0f, ly - 1.0f, lz - 1.0f);
        float r = perlinFadeF(lx); float s = perlinFadeF(ly); float t = perlinFadeF(lz);
        float x0 = lerpF(r, d, e); float x1 = lerpF(r, f, g);
        float x2 = lerpF(r, h, o); float x3 = lerpF(r, p, q);
        float y0 = lerpF(s, x0, x1); float y1 = lerpF(s, x2, x3);
        return lerpF(t, y0, y1);
    }
    // old_blended 5 参数 sample：读 7 值拆分坐标 [ix,iy,iz,gx,gy(h-n),gz,fadeY(h)]，y-fade 用 fadeY（第 7 值）
    float pn_section_f32(int octBase, int sIdx, int splitOffset) {
        int b = sIdx * splitTotal + splitOffset;
        int sx = (int)splitCoord[b + 0];
        int sy = (int)splitCoord[b + 1];
        int sz = (int)splitCoord[b + 2];
        float lx = splitCoord[b + 3];
        float ly = splitCoord[b + 4];
        float lz = splitCoord[b + 5];
        float fadeY = splitCoord[b + 6];
        int i0 = mapPermD(octBase, sx); int j = mapPermD(octBase, sx + 1);
        int k = mapPermD(octBase, i0 + sy); int l = mapPermD(octBase, i0 + sy + 1);
        int m = mapPermD(octBase, j + sy); int nn = mapPermD(octBase, j + sy + 1);
        float d  = gradDotF(mapPermD(octBase, k + sz),     lx,     ly,     lz);
        float e  = gradDotF(mapPermD(octBase, m + sz),     lx - 1.0f, ly,     lz);
        float f  = gradDotF(mapPermD(octBase, l + sz),     lx,     ly - 1.0f, lz);
        float g  = gradDotF(mapPermD(octBase, nn + sz),    lx - 1.0f, ly - 1.0f, lz);
        float h  = gradDotF(mapPermD(octBase, k + sz + 1), lx,     ly,     lz - 1.0f);
        float o  = gradDotF(mapPermD(octBase, m + sz + 1), lx - 1.0f, ly,     lz - 1.0f);
        float p  = gradDotF(mapPermD(octBase, l + sz + 1), lx,     ly - 1.0f, lz - 1.0f);
        float q  = gradDotF(mapPermD(octBase, nn + sz + 1), lx - 1.0f, ly - 1.0f, lz - 1.0f);
        float r = perlinFadeF(lx); float s = perlinFadeF(fadeY); float t = perlinFadeF(lz);
        float x0 = lerpF(r, d, e); float x1 = lerpF(r, f, g);
        float x2 = lerpF(r, h, o); float x3 = lerpF(r, p, q);
        float y0 = lerpF(s, x0, x1); float y1 = lerpF(s, x2, x3);
        return lerpF(t, y0, y1);
    }
    float y_clamped_gradient(int y, float fromY, float toY, float fromV, float toV) {
        if (toY == fromY) return 0.0f;
        float t = std::min(1.0f, std::max(0.0f, ((float)y - fromY) / (toY - fromY)));
        return fromV + t * (toV - fromV);
    }

    // ---- 数据驱动噪声（读取 A 表 + splitCoord + perm）----
    float normal_noise(int noiseIdx, int sIdx) {
        int b3 = noiseIdx * 3;
        int n = NORMAL_PACK[b3 + 0];
        int octBase = NORMAL_PACK[b3 + 1];
        int splitBase = NORMAL_PACK[b3 + 2];
        float persistence = NORMAL_PACK_F[noiseIdx * 2 + 0];
        float amplitude = NORMAL_PACK_F[noiseIdx * 2 + 1];
        int ampOff = NORMAL_AMP_OFF[noiseIdx];
        float d = 0.0f; float f = persistence;
        for (int i = 0; i < n; i++) {
            int b = sIdx * splitTotal + splitBase + i * 6;
            int ix = (int)splitCoord[b + 0]; int iy = (int)splitCoord[b + 1]; int iz = (int)splitCoord[b + 2];
            float gx = splitCoord[b + 3]; float gy = splitCoord[b + 4]; float gz = splitCoord[b + 5];
            float ns = pn_sample3_f32(octBase + i, ix, iy, iz, gx, gy, gz);
            d += NORMAL_AMPS[ampOff + i] * ns * f;
            f /= 2.0f;   // persistence 每 octave /2（红线）
        }
        float d2 = 0.0f; f = persistence;
        for (int i = 0; i < n; i++) {
            int b = sIdx * splitTotal + splitBase + 6 * n + i * 6;
            int ix = (int)splitCoord[b + 0]; int iy = (int)splitCoord[b + 1]; int iz = (int)splitCoord[b + 2];
            float gx = splitCoord[b + 3]; float gy = splitCoord[b + 4]; float gz = splitCoord[b + 5];
            float ns = pn_sample3_f32(octBase + n + i, ix, iy, iz, gx, gy, gz);
            d2 += NORMAL_AMPS[ampOff + i] * ns * f;
            f /= 2.0f;
        }
        return (d + d2) * amplitude;
    }
    float interp_noise(int idx, int sIdx) {
        int octBase = OLD_PACK[idx * 2 + 0];
        int splitBase = OLD_PACK[idx * 2 + 1];
        float n = 0.0f; float o = 1.0f;
        for (int q = 0; q < 8; q++) {
            n += pn_section_f32(octBase + 32 + q, sIdx, splitBase + (32 + q) * 7) / o;
            o /= 2.0f;
        }
        float qq = (n / 10.0f + 1.0f) / 2.0f;
        bool bl = qq >= 1.0f; bool bl2 = qq <= 0.0f;
        float l = 0.0f; float mm = 0.0f; o = 1.0f;
        for (int r = 0; r < 16; r++) {
            if (!bl) l += pn_section_f32(octBase + r, sIdx, splitBase + r * 7) / o;         // 独立早停 1
            if (!bl2) mm += pn_section_f32(octBase + 16 + r, sIdx, splitBase + (16 + r) * 7) / o;   // 独立早停 2
            o /= 2.0f;   // 除法每圈执行（红线）
        }
        float w = std::min(1.0f, std::max(0.0f, qq));
        return (l / 512.0f + w * (mm / 512.0f - l / 512.0f)) / 128.0f;
    }

    // ---- spline（数据驱动表 + 显式栈 stage 机；D23 边界嵌套递归到 v0/v1 子帧）----
    float spline_coord(int coordType, int corner, int sIdx, int ix, int iy, int iz) {
        int slot = COORD_SLOT_TABLE[coordType];
        float v = normal_noise(NOISE_SLOT_BASE[slot] + corner * NOISE_SLOT_STRIDE[slot], sIdx);
        if (coordType == 0) v = (v);
        if (coordType == 1) v = (v);
        if (coordType == 2) v = (-3.0f * (-0.3333333333333333f + std::fabs((-0.6666666666666666f + std::fabs((v))))));
        if (coordType == 3) v = (v);
        return v;
    }
    int spline_find_range(float x, int locBegin, int n) {
        int mn = 0; int i = n;
        while (i > 0) {
            int j = i / 2; int k = mn + j;
            if (x < splineLocs[locBegin + k]) { i = j; }
            else { mn = k + 1; i -= j + 1; }
        }
        return mn - 1;
    }
    float spline_hermite(float coord, float lo, float span, float nv, float ov, float d0, float d1) {
        float kd = (coord - lo) / span;
        float p = d0 * span - (ov - nv);
        float q = -d1 * span + (ov - nv);
        return (nv + kd * (ov - nv)) + kd * (1.0f - kd) * (p + kd * (q - p));
    }
    float spline_eval(int rootNode, int corner, int sIdx, int ix, int iy, int iz) {
        int st_node[64]; int st_i[64]; int st_stage[64];
        float st_coord[64]; float st_v0[64]; float st_v1[64];
        int sp = 0;
        st_node[0] = rootNode; st_stage[0] = 0; sp = 1;
        float outVal = 0.0f;
        while (sp > 0) {
            int f = sp - 1;
            int node = st_node[f];
            int p = node * 5;
            int ct = splineNodePack[p + 0];
            int n = splineNodePack[p + 1];
            int locB = splineNodePack[p + 2];
            int derB = splineNodePack[p + 3];
            int valB = splineNodePack[p + 4];
            if (st_stage[f] == 0) {
                float coord = spline_coord(ct, corner, sIdx, ix, iy, iz);
                int i = spline_find_range(coord, locB, n);
                st_coord[f] = coord; st_i[f] = i;
                if (i < 0) {
                    // D23：左边界外推遇嵌套 value 必须递归求值（非 0.0）
                    if (splineValKind[valB] == 0) {
                        outVal = splineValF[valB] + splineDers[derB] * (coord - splineLocs[locB]);
                        sp--;
                    } else {
                        st_stage[f] = 4;   // 等边界 v0 子帧回填
                        st_node[sp] = splineValNode[valB]; st_stage[sp] = 0; sp++;
                    }
                } else if (i >= n - 1) {
                    // D23：右边界外推
                    if (splineValKind[valB + n - 1] == 0) {
                        outVal = splineValF[valB + n - 1] + splineDers[derB + n - 1] * (coord - splineLocs[locB + n - 1]);
                        sp--;
                    } else {
                        st_stage[f] = 5;   // 等边界 vn 子帧回填
                        st_node[sp] = splineValNode[valB + n - 1]; st_stage[sp] = 0; sp++;
                    }
                } else {
                    st_stage[f] = 1;
                    if (splineValKind[valB + i] == 0) {
                        st_v0[f] = splineValF[valB + i];
                        st_stage[f] = 2;
                        if (splineValKind[valB + i + 1] == 0) {
                            st_v1[f] = splineValF[valB + i + 1];
                            float lo = splineLocs[locB + i];
                            outVal = spline_hermite(coord, lo, splineLocs[locB + i + 1] - lo, st_v0[f], st_v1[f], splineDers[derB + i], splineDers[derB + i + 1]);
                            sp--;
                        } else {
                            st_stage[f] = 3;
                            st_node[sp] = splineValNode[valB + i + 1]; st_stage[sp] = 0; sp++;
                        }
                    } else {
                        st_node[sp] = splineValNode[valB + i]; st_stage[sp] = 0; sp++;
                    }
                }
            } else if (st_stage[f] == 4) {
                // D23：边界 v0 子帧回填 → 左侧外推
                float coord = st_coord[f];
                outVal += splineDers[derB] * (coord - splineLocs[locB]);
                sp--;
            } else if (st_stage[f] == 5) {
                // D23：边界 vn 子帧回填 → 右侧外推
                float coord = st_coord[f];
                outVal += splineDers[derB + n - 1] * (coord - splineLocs[locB + n - 1]);
                sp--;
            } else if (st_stage[f] == 1) {
                // 等 v0 子帧回填
                st_v0[f] = outVal;
                st_stage[f] = 2;
                int i = st_i[f];
                if (splineValKind[valB + i + 1] == 0) {
                    st_v1[f] = splineValF[valB + i + 1];
                    float lo = splineLocs[locB + i];
                    outVal = spline_hermite(st_coord[f], lo, splineLocs[locB + i + 1] - lo, st_v0[f], st_v1[f], splineDers[derB + i], splineDers[derB + i + 1]);
                    sp--;
                } else {
                    st_stage[f] = 3;
                    st_node[sp] = splineValNode[valB + i + 1]; st_stage[sp] = 0; sp++;
                }
            } else if (st_stage[f] == 2) {
                // 瞬态（v0 子帧回填后 v1 也齐）：完成 Hermite
                st_v1[f] = outVal;
                int i = st_i[f];
                float lo = splineLocs[locB + i];
                outVal = spline_hermite(st_coord[f], lo, splineLocs[locB + i + 1] - lo, st_v0[f], st_v1[f], splineDers[derB + i], splineDers[derB + i + 1]);
                sp--;
            } else if (st_stage[f] == 3) {
                // 等 v1 子帧回填 → Hermite 完成
                float v1 = outVal;
                int i = st_i[f];
                float lo = splineLocs[locB + i];
                outVal = spline_hermite(st_coord[f], lo, splineLocs[locB + i + 1] - lo, st_v0[f], v1, splineDers[derB + i], splineDers[derB + i + 1]);
                sp--;
            }
        }
        return outVal;
    }

    // ---- grid 缓存（path C：每 interp 每 chunk 的 5×49×5 去重网格 + 三线性）----
    // thread_local（方案 i，对齐 production density.h tlSlots）：per-thread GridSlot 缓冲按 interpIdx 索引。
    // GridSlot 含 key + grid + edgeCX/CZ + edgeCol（gx=4 列，跨 chunk 复用）。static 成员加 inline（multi-TU LNK2005 教训）。
    struct GridSlot {
        int64_t key = INT64_MIN;
        float grid[49][5][5];
        int edgeCX = INT32_MIN, edgeCZ = INT32_MIN;
        float edgeCol[49][5];
    };
    static inline std::vector<GridSlot>& gridSlots() {
        static thread_local std::vector<GridSlot> slots;
        if (slots.size() < (size_t)N_INTERP) slots.resize((size_t)N_INTERP);
        return slots;
    }

    // 构建某 interp 在某 chunk 的网格。关键正确性（勿违）：eval_df_base 的取值绑定 sIdx 的
    // split 位置（非实参坐标），故每个网格节点必须用「该节点的 split」求值。grid 节点 (gx,gy,gz)
    // 恰在 4/8 网格线上，是 cell (gx,gy,gz) 的 (dx,dy,dz)=(0,0,0) 角点 → corner 恒 0（split(节点坐标)
    // 生成节点自身 split）。节点值唯一性已证：verif_grid_cache_correctness.md；用单实例 corner=0 等价
    // production arg->sample(nodePos)。
    // 优化：① per-cell split 去重（方案 a，不改生成器接口/布局翻转）——网格节点是其 cell 的 (0,0,0) 角点，
    //   故逐 interior cell 调一次 split(cell 角点坐标) 即得该 cell corner0 节点值，无需像旧实现那样按节点整树重算；
    //   单 splitCoord 缓冲逐 cell 复用（corner=0 下 per-cell 槽无跨节点共享，存 768×splitTotal 属冗余存储）。
    // ② edgeCol——gx=4 列 == 右邻 chunk 的 gx=0 列（x 相同），存 gx=4 列供右邻复用其 gx=0 列（~20% 节点免算）。
    // 注意：buildInterpGrid 会覆盖 splitCoord，需还原外层 eval_df 的非 interp 路径（block 位置）split。
    void buildInterpGrid(int interpIdx, int chunkX, int chunkZ) {
        GridSlot& s = gridSlots()[interpIdx];
        std::vector<float> saved = splitCoord;   // 保留外层 (block 位置) 的 split（反 interp 路径）
        if ((size_t)splitCoord.size() < (size_t)splitTotal) splitCoord.assign((size_t)splitTotal, 0.0f);

        // edgeCol 复用条件：左邻 chunk (chunkX-1,chunkZ) 已建 → 本 chunk 的 gx=0 列 == 左邻 gx=4 列（x 相同）。
        // 严格左邻标记 + 复用才拷贝；否则退回全建（正确性优先）。
        const bool reuseLeft = (s.edgeCX == chunkX - 1 && s.edgeCZ == chunkZ);
        if (reuseLeft) {
            for (int gy = 0; gy < 49; gy++)
                for (int gz = 0; gz < 5; gz++)
                    s.grid[gy][gz][0] = s.edgeCol[gy][gz];
        }

        // per-cell split 去重（corner=0）：interior cell (cx,cy,cz) 的 (0,0,0) 角点 = 网格节点 (cx,cy,cz)。
        // 逐 cell 算一次 split(cell 角点坐标) → 该 cell 8 角点展开，corner=0 即节点自身。gx=0 列若已从 edgeCol 复用则跳过。
        for (int gy = 0; gy < 48; gy++) {
            for (int gz = 0; gz < 4; gz++) {
                for (int gx = 0; gx < 4; gx++) {
                    if (gx == 0 && reuseLeft) continue;   // 已从 edgeCol 复用
                    int nx = chunkX * 16 + gx * 4;
                    int ny = minY + gy * 8;
                    int nz = chunkZ * 16 + gz * 4;
                    split(nx, ny, nz, splitCoord.data());     // 该 cell 的 8 角点展开（corner=0 = 节点自身）
                    s.grid[gy][gz][gx] = eval_df_base(interpIdx, 0, 0, nx, ny, nz);
                }
            }
        }
        // 边界列/行：gx=4、gz=4、gy=48（跨 chunk/维度 cell 的 (0,0,0) 角点，直接 split 求值）。
        for (int gy = 0; gy < 49; gy++) {
            for (int gz = 0; gz < 5; gz++) {
                for (int gx = 0; gx < 5; gx++) {
                    if (gx < 4 && gy < 48 && gz < 4) continue;   // interior（已由 per-cell/edgeCol 填充）
                    if (gx == 0 && reuseLeft) continue;          // 已从 edgeCol 复用
                    int nx = chunkX * 16 + gx * 4;
                    int ny = minY + gy * 8;
                    int nz = chunkZ * 16 + gz * 4;
                    split(nx, ny, nz, splitCoord.data());
                    s.grid[gy][gz][gx] = eval_df_base(interpIdx, 0, 0, nx, ny, nz);
                }
            }
        }
        splitCoord.swap(saved);    // 还原（grid 值已存入 s.grid，无需保留节点 split；swap 免 <utility> 依赖）

        // 存 edgeCol（gx=4 列）供右邻 chunk (chunkX+1,...) 复用其 gx=0 列；edgeCol 存网格节点原始值（未三线性）。
        s.edgeCX = chunkX; s.edgeCZ = chunkZ;
        for (int gy = 0; gy < 49; gy++)
            for (int gz = 0; gz < 5; gz++)
                s.edgeCol[gy][gz] = s.grid[gy][gz][4];
        s.key = ((int64_t)chunkX << 32) | (chunkZ & 0xFFFFFFFFLL);
    }

    // 用 grid 缓存三线性采样 interp（先支持 sIdx=0 单点/整 chunk；sIdx 批量语义后续对齐，
    // sIdx!=0 由 interp_N 回退原 8 角点重算）。三线性逻辑与现有 interp_N 一致。
    float sampleInterpGrid(int interpIdx, int ix, int iy, int iz) {
        int chunkX = floorDiv(ix, 16); int chunkZ = floorDiv(iz, 16);
        int64_t key = ((int64_t)chunkX << 32) | (chunkZ & 0xFFFFFFFFLL);
        GridSlot& s = gridSlots()[interpIdx];
        if (s.key != key) buildInterpGrid(interpIdx, chunkX, chunkZ);
        int gx = ix - chunkX * 16; int gy = iy - minY; int gz = iz - chunkZ * 16;
        int cx = gx / 4; int cy = gy / 8; int cz = gz / 4;
        float fx = (float)(gx % 4) / 4.0f; float fy = (float)(gy % 8) / 8.0f; float fz = (float)(gz % 4) / 4.0f;
        float d000 = s.grid[cy + 0][cz + 0][cx + 0];
        float d100 = s.grid[cy + 0][cz + 0][cx + 1];
        float d010 = s.grid[cy + 1][cz + 0][cx + 0];
        float d110 = s.grid[cy + 1][cz + 0][cx + 1];
        float d001 = s.grid[cy + 0][cz + 1][cx + 0];
        float d101 = s.grid[cy + 0][cz + 1][cx + 1];
        float d011 = s.grid[cy + 1][cz + 1][cx + 0];
        float d111 = s.grid[cy + 1][cz + 1][cx + 1];
        float d00 = d000 + (d100 - d000) * fx; float d10 = d010 + (d110 - d010) * fx;
        float d01 = d001 + (d101 - d001) * fx; float d11 = d011 + (d111 - d011) * fx;
        float d0 = d00 + (d10 - d00) * fy; float d1 = d01 + (d11 - d01) * fy;
        return d0 + (d1 - d0) * fz;
    }

    // ---- 解释器（D25：每 interp 只遍历自身 delegate 闭包，消除其他 interp 的 dead delegate 计算）----
    // 闭包数组（CLOSURE_*）由 gen 从 _compute_val_layout 导出，与 GLSL CTYPE_N/CAx_N/CFx_N/SLOT_OF_N 同源。
    // val[k] = 闭包内线性槽（CLOSURE_SLOT 映射闭包内位置→槽）；子节点 a1/a2/a3 已是闭包内位置。
    float eval_df_base(int interpIdx, int corner, int sIdx, int ix, int iy, int iz) {
        float val[CLOSURE_MAX_SLOTS];
        int base = CLOSURE_OFF[interpIdx];
        int len = CLOSURE_LEN[interpIdx];
        for (int ci = 0; ci < len; ci++) {
            int gi = base + ci;
            int t = CLOSURE_TYPE[gi];
            int a1 = CLOSURE_A1[gi]; int a2 = CLOSURE_A2[gi]; int a3 = CLOSURE_A3[gi];
            float f0 = CLOSURE_F0[gi]; float f1 = CLOSURE_F1[gi]; float f2 = CLOSURE_F2[gi]; float f3 = CLOSURE_F3[gi];
            float r = 0.0f;
            if (t == 0) r = f0;                                            // DF_CONSTANT
            else if (t == 1) r = (float)iy;                               // DF_Y
            else if (t == 2 || t == 19) r = normal_noise(NOISE_SLOT_BASE[a1] + corner * NOISE_SLOT_STRIDE[a1], sIdx);  // DF_NOISE/DF_SHIFTED_NOISE
            else if (t == 3) r = interp_noise(NOISE_SLOT_BASE[a1] + corner * NOISE_SLOT_STRIDE[a1], sIdx);            // DF_OLD_BLENDED
            else if (t == 4) {                                            // DF_SPLINE
                if (a2 == 1) r = spline_eval(a1, corner, sIdx, (ix >> 2) << 2, 0, (iz >> 2) << 2);
                else r = spline_eval(a1, corner, sIdx, ix, iy, iz);
            }
            else if (t == 18) r = y_clamped_gradient(iy, f0, f1, f2, f3); // DF_Y_CLAMPED
            else if (t == 10) r = std::fabs(val[CLOSURE_SLOT[base + a1]]);          // DF_ABS
            else if (t == 11) { float v = val[CLOSURE_SLOT[base + a1]]; r = v * v; }           // DF_SQUARE
            else if (t == 12) { float v = val[CLOSURE_SLOT[base + a1]]; r = v * v * v; }       // DF_CUBE
            else if (t == 13) { float v = val[CLOSURE_SLOT[base + a1]]; r = (v > 0.0f ? v : v * 0.5f); }    // DF_HALF_NEG
            else if (t == 14) { float v = val[CLOSURE_SLOT[base + a1]]; r = (v > 0.0f ? v : v * 0.25f); }   // DF_QUARTER_NEG
            else if (t == 15) { float v = val[CLOSURE_SLOT[base + a1]]; float c = (v > 1.0f ? 1.0f : (v < -1.0f ? -1.0f : v)); r = c / 2.0f - c * c * c / 24.0f; }  // DF_SQUEEZE
            else if (t == 16) r = (val[CLOSURE_SLOT[base + a1]] > f1 ? f1 : (val[CLOSURE_SLOT[base + a1]] < f0 ? f0 : val[CLOSURE_SLOT[base + a1]]));  // DF_CLAMP
            else if (t == 17) { float inp = val[CLOSURE_SLOT[base + a1]]; r = (inp >= f0 && inp < f1) ? val[CLOSURE_SLOT[base + a2]] : val[CLOSURE_SLOT[base + a3]]; }  // DF_RANGE_CHOICE
            else if (t == 22) { float v = val[CLOSURE_SLOT[base + a1]]; float d = ws_scaleF((int)f0, v); r = d * std::fabs(normal_noise(NOISE_SLOT_BASE[a2] + corner * NOISE_SLOT_STRIDE[a2], sIdx)); }  // DF_WEIRD
            else if (t == 20 || t == 21) r = val[CLOSURE_SLOT[base + a1]];                     // DF_BLEND_DENSITY/DF_FLAT_CACHE
            else if (t == 6) r = val[CLOSURE_SLOT[base + a1]] + val[CLOSURE_SLOT[base + a2]]; // DF_ADD
            else if (t == 7) r = val[CLOSURE_SLOT[base + a1]] * val[CLOSURE_SLOT[base + a2]]; // DF_MUL
            else if (t == 8) r = std::min(val[CLOSURE_SLOT[base + a1]], val[CLOSURE_SLOT[base + a2]]);  // DF_MIN
            else if (t == 9) r = std::max(val[CLOSURE_SLOT[base + a1]], val[CLOSURE_SLOT[base + a2]]);  // DF_MAX
            else r = 0.0f;                                                // DF_INTERP(5)：delegate 树不含，防御 0
            val[CLOSURE_SLOT[base + ci]] = r;
        }
        return val[CLOSURE_SLOT[base + CLOSURE_ROOT_POS[interpIdx]]];
    }
    // ---- 阶段 A：软流 K 路（interp 闭包交错）----
    // K 个独立点交错遍历同一 CLOSURE 闭包表（共享 CLOSURE_*），每点独立 val[k]/sIdx/坐标。
    // 内层 k 循环使 K 个独立点的同一 op 的 load/算交叠（MLP：K 个独立 load 在飞行）。
    // ⚠️ 当前保留 split（normal_noise 仍读 splitCoord）；待阶段 B 去 split 消除 split()/splitTop() 慢。
    // K 上限 16（vbuf 静态）；out[k] 每点 interp 闭包根值。
    void eval_df_base_soft(int interpIdx, int corner, const int sIdx[], int K,
                           const int ix[], const int iy[], const int iz[], float out[]) const {
        static thread_local float vbuf[16][CLOSURE_MAX_SLOTS];   // K≤16
        int base = CLOSURE_OFF[interpIdx];
        int len = CLOSURE_LEN[interpIdx];
        for (int ci = 0; ci < len; ci++) {
            int gi = base + ci;
            int t = CLOSURE_TYPE[gi];
            int a1 = CLOSURE_A1[gi]; int a2 = CLOSURE_A2[gi]; int a3 = CLOSURE_A3[gi];
            float f0 = CLOSURE_F0[gi]; float f1 = CLOSURE_F1[gi]; float f2 = CLOSURE_F2[gi]; float f3 = CLOSURE_F3[gi];
            int slot = CLOSURE_SLOT[gi];
            for (int k = 0; k < K; k++) {
                float r = 0.0f;
                if (t == 0) r = f0;                                                      // DF_CONSTANT
                else if (t == 1) r = (float)iy[k];                                     // DF_Y
                else if (t == 2 || t == 19) r = normal_noise(NOISE_SLOT_BASE[a1] + corner * NOISE_SLOT_STRIDE[a1], sIdx[k]);
                else if (t == 3) r = interp_noise(NOISE_SLOT_BASE[a1] + corner * NOISE_SLOT_STRIDE[a1], sIdx[k]);
                else if (t == 4) {                                                      // DF_SPLINE
                    if (a2 == 1) r = spline_eval(a1, corner, sIdx[k], (ix[k] >> 2) << 2, 0, (iz[k] >> 2) << 2);
                    else r = spline_eval(a1, corner, sIdx[k], ix[k], iy[k], iz[k]);
                }
                else if (t == 18) r = y_clamped_gradient(iy[k], f0, f1, f2, f3);
                else if (t == 10) r = std::fabs(vbuf[k][CLOSURE_SLOT[base + a1]]);
                else if (t == 11) { float v = vbuf[k][CLOSURE_SLOT[base + a1]]; r = v * v; }
                else if (t == 12) { float v = vbuf[k][CLOSURE_SLOT[base + a1]]; r = v * v * v; }
                else if (t == 13) { float v = vbuf[k][CLOSURE_SLOT[base + a1]]; r = (v > 0.0f ? v : v * 0.5f); }
                else if (t == 14) { float v = vbuf[k][CLOSURE_SLOT[base + a1]]; r = (v > 0.0f ? v : v * 0.25f); }
                else if (t == 15) { float v = vbuf[k][CLOSURE_SLOT[base + a1]]; float c = (v > 1.0f ? 1.0f : (v < -1.0f ? -1.0f : v)); r = c / 2.0f - c * c * c / 24.0f; }
                else if (t == 16) { float v = vbuf[k][CLOSURE_SLOT[base + a1]]; r = (v > f1 ? f1 : (v < f0 ? f0 : v)); }
                else if (t == 17) { float inp = vbuf[k][CLOSURE_SLOT[base + a1]]; r = (inp >= f0 && inp < f1) ? vbuf[k][CLOSURE_SLOT[base + a2]] : vbuf[k][CLOSURE_SLOT[base + a3]]; }
                else if (t == 22) { float v = vbuf[k][CLOSURE_SLOT[base + a1]]; float d = ws_scaleF((int)f0, v); r = d * std::fabs(normal_noise(NOISE_SLOT_BASE[a2] + corner * NOISE_SLOT_STRIDE[a2], sIdx[k])); }
                else if (t == 20 || t == 21) r = vbuf[k][CLOSURE_SLOT[base + a1]];
                else if (t == 6) r = vbuf[k][CLOSURE_SLOT[base + a1]] + vbuf[k][CLOSURE_SLOT[base + a2]];
                else if (t == 7) r = vbuf[k][CLOSURE_SLOT[base + a1]] * vbuf[k][CLOSURE_SLOT[base + a2]];
                else if (t == 8) r = std::min(vbuf[k][CLOSURE_SLOT[base + a1]], vbuf[k][CLOSURE_SLOT[base + a2]]);
                else if (t == 9) r = std::max(vbuf[k][CLOSURE_SLOT[base + a1]], vbuf[k][CLOSURE_SLOT[base + a2]]);
                vbuf[k][slot] = r;
            }
        }
        int root = CLOSURE_ROOT_POS[interpIdx];
        for (int k = 0; k < K; k++) out[k] = vbuf[k][CLOSURE_SLOT[base + root]];
    }

    float interp_N(int interpIdx, int sIdx, int ix, int iy, int iz) {
        if (sIdx == 0) return sampleInterpGrid(interpIdx, ix, iy, iz);   // path C：grid 缓存（sIdx=0）
        int chunkX = floorDiv(ix, 16); int chunkZ = floorDiv(iz, 16);
        int gx = ix - chunkX * 16; int gy = iy - minY; int gz = iz - chunkZ * 16;
        int cx = gx / 4; int cy = gy / 8; int cz = gz / 4;
        float fx = (float)(gx % 4) / 4.0f; float fy = (float)(gy % 8) / 8.0f; float fz = (float)(gz % 4) / 4.0f;
        float d[8];
        for (int c = 0; c < 8; c++) {
            int dx = c & 1; int dy = (c >> 1) & 1; int dz = (c >> 2) & 1;
            int ax = chunkX * 16 + (cx + dx) * 4;
            int ay = minY + (cy + dy) * 8;
            int az = chunkZ * 16 + (cz + dz) * 4;
            d[c] = eval_df_base(interpIdx, c, sIdx, ax, ay, az);
        }
        float d00 = d[0] + (d[1] - d[0]) * fx; float d10 = d[2] + (d[3] - d[2]) * fx;
        float d01 = d[4] + (d[5] - d[4]) * fx; float d11 = d[6] + (d[7] - d[6]) * fx;
        float d0 = d00 + (d10 - d00) * fy; float d1 = d01 + (d11 - d01) * fy;
        return d0 + (d1 - d0) * fz;
    }
    float eval_df(int sIdx, int ix, int iy, int iz) {
        float val[VAL_SLOTS_TOP];
        for (int ci = 0; ci < TOP_CLOSURE_LEN; ci++) {
            int t = TOP_TYPE[ci];
            int a1 = TOP_A1[ci]; int a2 = TOP_A2[ci]; int a3 = TOP_A3[ci];
            float f0 = TOP_F0[ci]; float f1 = TOP_F1[ci]; float f2 = TOP_F2[ci]; float f3 = TOP_F3[ci];
            float r = 0.0f;
            if (t == 5) { r = interp_N(a1, sIdx, ix, iy, iz); val[TOP_SLOT[ci]] = r; continue; }   // DF_INTERP → interp_N
            if (t == 0) r = f0;
            else if (t == 1) r = (float)iy;
            else if (t == 2 || t == 19) r = normal_noise(NOISE_SLOT_BASE[a1], sIdx);    // corner=0
            else if (t == 3) r = interp_noise(NOISE_SLOT_BASE[a1], sIdx);
            else if (t == 4) {
                if (a2 == 1) r = spline_eval(a1, 0, sIdx, (ix >> 2) << 2, 0, (iz >> 2) << 2);
                else r = spline_eval(a1, 0, sIdx, ix, iy, iz);
            }
            else if (t == 18) r = y_clamped_gradient(iy, f0, f1, f2, f3);
            else if (t == 10) r = std::fabs(val[TOP_SLOT[a1]]);
            else if (t == 11) { float v = val[TOP_SLOT[a1]]; r = v * v; }
            else if (t == 12) { float v = val[TOP_SLOT[a1]]; r = v * v * v; }
            else if (t == 13) { float v = val[TOP_SLOT[a1]]; r = (v > 0.0f ? v : v * 0.5f); }
            else if (t == 14) { float v = val[TOP_SLOT[a1]]; r = (v > 0.0f ? v : v * 0.25f); }
            else if (t == 15) { float v = val[TOP_SLOT[a1]]; float c = (v > 1.0f ? 1.0f : (v < -1.0f ? -1.0f : v)); r = c / 2.0f - c * c * c / 24.0f; }
            else if (t == 16) r = (val[TOP_SLOT[a1]] > f1 ? f1 : (val[TOP_SLOT[a1]] < f0 ? f0 : val[TOP_SLOT[a1]]));
            else if (t == 17) { float inp = val[TOP_SLOT[a1]]; r = (inp >= f0 && inp < f1) ? val[TOP_SLOT[a2]] : val[TOP_SLOT[a3]]; }
            else if (t == 22) { float v = val[TOP_SLOT[a1]]; float d = ws_scaleF((int)f0, v); r = d * std::fabs(normal_noise(NOISE_SLOT_BASE[a2], sIdx)); }
            else if (t == 20 || t == 21) r = val[TOP_SLOT[a1]];
            else if (t == 6) r = val[TOP_SLOT[a1]] + val[TOP_SLOT[a2]];
            else if (t == 7) r = val[TOP_SLOT[a1]] * val[TOP_SLOT[a2]];
            else if (t == 8) r = std::min(val[TOP_SLOT[a1]], val[TOP_SLOT[a2]]);
            else if (t == 9) r = std::max(val[TOP_SLOT[a1]], val[TOP_SLOT[a2]]);
            val[TOP_SLOT[ci]] = r;
        }
        return val[TOP_SLOT[TOP_ROOT_POS]];
    }
    float eval_density(int sIdx, int ix, int iy, int iz) {
        return eval_df(sIdx, ix, iy, iz);
    }

    // ---- 便捷入口：单点采样（prepare splitCoord + eval_density sIdx=0）----
    void prepare(int x, int y, int z) {
        splitCoord.assign((size_t)splitTotal, 0.0f);
        split(x, y, z, splitCoord.data());
    }
    float sample(int x, int y, int z) {
        if ((size_t)splitCoord.size() < (size_t)splitTotal) splitCoord.assign((size_t)splitTotal, 0.0f);
        // grid 命中（同 chunk）时 interp 走 grid，非 interp 只读 @c0 → splitTop（整树 split 的 1/8），
        // 避免每点整树 split()。splitTop 只覆盖 interp delegate 的 @c0 + 顶层 spline 坐标；这是
        // eval_density(sIdx=0) 非 interp 路径所需的全部 split（见 gen_cpu 的 splitTop 注释）。
        // grid miss 时 buildInterpGrid 内部仍用全量 split() 建网格并把 splitCoord.swap(saved) 还原为
        // 本行的 splitTop 结果，@c0 非 interp 读值保持不变。
        splitTop(x, y, z, splitCoord.data());
        return eval_density(0, x, y, z);
    }

};
