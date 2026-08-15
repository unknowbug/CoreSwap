// 自动生成（DFC CPU 后端），勿手改
#pragma once
#include <vector>
#include <map>
#include <string>
#include <cmath>
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
};
