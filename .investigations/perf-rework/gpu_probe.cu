// gpu_probe.cu —— 实测 GPU 数据流开销（PCIe 传输 + kernel 启动 + 单 chunk 往返）
// 目的：回答「CPU FP64 折叠坐标 → GPU 吃小坐标 → GPU 算 density → CPU 判定」的数据流开销是否可忽略（对比 CPU density 47ms/chunk）
// 硬件：RTX 4060 Laptop（PCIe 4.0 x8）
#include <cstdio>
#include <cuda_runtime.h>
#include <chrono>

using namespace std::chrono;

__global__ void emptyKernel() {}

// 模拟 FP32 计算负载（与 density 树遍历/插值的浮点强度同量级）
__global__ void busyKernel(float* out, int n) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) {
        float acc = out[i];
        for (int k = 0; k < 200; k++) acc = acc * 1.0001f + 0.0001f;
        out[i] = acc;
    }
}

#define CHECK(c) do { cudaError_t e = (c); if (e != cudaSuccess) { printf("CUDA error %s: %s\n", #c, cudaGetErrorString(e)); return 1; } } while(0)

int main() {
    cudaDeviceProp prop;
    CHECK(cudaGetDeviceProperties(&prop, 0));
    printf("GPU: %s\n", prop.name);

    // 每 chunk 数据量（分层方案）：
    //   in  : 7350 角点 × 3 float = 88KB（折叠坐标）
    //   out : 98304 块 × 1 float = 384KB（density）
    const int N_OUT = 98304;          // 384KB
    const int N_IN  = 7350 * 3;       // 88KB
    float *h_in = new float[N_IN];
    float *h_out = new float[N_OUT];
    float *d_in, *d_out;
    CHECK(cudaMalloc(&d_in,  N_IN  * sizeof(float)));
    CHECK(cudaMalloc(&d_out, N_OUT * sizeof(float)));
    for (int i = 0; i < N_IN;  i++) h_in[i]  = 0.5f;
    for (int i = 0; i < N_OUT; i++) h_out[i] = 0.5f;

    // warmup
    CHECK(cudaMemcpy(d_in, h_in, N_IN * sizeof(float), cudaMemcpyHostToDevice));
    emptyKernel<<<1,1>>>();
    CHECK(cudaMemcpy(h_out, d_out, N_OUT * sizeof(float), cudaMemcpyDeviceToHost));
    CHECK(cudaDeviceSynchronize());

    // 1. 单 chunk 往返端到端延迟（H2D + kernel + D2H + sync）—— 实时逐 chunk 的关键指标
    const int ITERS = 2000;
    auto t0 = high_resolution_clock::now();
    for (int i = 0; i < ITERS; i++) {
        cudaMemcpy(d_in, h_in, N_IN * sizeof(float), cudaMemcpyHostToDevice);
        emptyKernel<<<1,1>>>();
        cudaMemcpy(h_out, d_out, N_OUT * sizeof(float), cudaMemcpyDeviceToHost);
        cudaDeviceSynchronize();
    }
    auto t1 = high_resolution_clock::now();
    double perRoundtrip = duration_cast<microseconds>(t1 - t0).count() / (double)ITERS;
    printf("[1] 单 chunk 往返(H2D 88KB + kernel + D2H 384KB + sync): %.1f us\n", perRoundtrip);

    // 2. 纯 PCIe 传输带宽（大块 384KB D2H）
    t0 = high_resolution_clock::now();
    for (int i = 0; i < ITERS; i++)
        cudaMemcpy(h_out, d_out, N_OUT * sizeof(float), cudaMemcpyDeviceToHost);
    t1 = high_resolution_clock::now();
    double perD2H = duration_cast<microseconds>(t1 - t0).count() / (double)ITERS;
    double bw = (N_OUT * 4.0) / (perD2H * 1e-6) / 1e9;
    printf("[2] D2H 384KB: %.1f us, 实测带宽 %.1f GB/s\n", perD2H, bw);

    // 3. kernel 启动延迟（空 kernel，异步 pipeline 后 sync 摊薄）
    t0 = high_resolution_clock::now();
    for (int i = 0; i < 20000; i++) emptyKernel<<<1,1>>>();
    CHECK(cudaDeviceSynchronize());
    t1 = high_resolution_clock::now();
    double perLaunch = duration_cast<microseconds>(t1 - t0).count() / 20000.0;
    printf("[3] kernel 启动延迟(pipeline 摊薄): %.2f us\n", perLaunch);

    // 4. busy kernel（模拟 FP32 计算负载）单 chunk 时间
    int blocks = (N_OUT + 255) / 256;
    t0 = high_resolution_clock::now();
    for (int i = 0; i < 200; i++) busyKernel<<<blocks, 256>>>(d_out, N_OUT);
    CHECK(cudaDeviceSynchronize());
    t1 = high_resolution_clock::now();
    double perBusy = duration_cast<microseconds>(t1 - t0).count() / 200.0;
    printf("[4] busy kernel(98304×200 乘加): %.1f us/chunk\n", perBusy);

    // 5. 对比 CPU 47ms/chunk
    printf("\n=== 对比 ===\n");
    printf("CPU density(单线程, spline 扁平化后): 47000 us/chunk\n");
    printf("GPU 单 chunk 往返: %.1f us  (%.1fx 快)\n", perRoundtrip, 47000.0 / perRoundtrip);
    printf("结论: 数据流开销是否可忽略\n");

    cudaFree(d_in); cudaFree(d_out);
    delete[] h_in; delete[] h_out;
    return 0;
}
