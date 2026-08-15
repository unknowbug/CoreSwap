// gpu_density_engine.cpp —— GPU 密度引擎实现（I2，PIMPL）
#include "gpu_density_engine.h"
#include "vulkan_runtime.h"
#include "cpu_backend.h"   // 生成器产物（gen_final_density.py 输出到 gpu-assets/cpu_backend.h，CMake include 指向该目录）
#include <cstdio>
#include <cstring>
#include <vector>
#include <mutex>

struct GpuDensityEngine::Impl {
    CpuBackend backend;             // CPU 拆分 + perm/spline 数据（生成器产出）
    std::unique_ptr<VkRuntime> rt;  // Vulkan 运行时（组件）
    VkRuntime::Buffer coordBuf, permBuf, splitBuf, outBuf, valBuf;
    VkRuntime::Buffer npBuf, locBuf, derBuf, vfBuf, vkBuf, vnBuf;
    VkDescriptorSet ds = VK_NULL_HANDLE;
    int curCap = 0;                 // 当前 buffer 容量（N），不足时重建
    // seed 级常量数据（init 时从 backend 复制）
    std::vector<uint32_t> perm;
    VkDeviceSize permSize = 0;
    std::vector<int32_t> splineNodePack, splineValKind, splineValNode;
    std::vector<float> splineLocs, splineDers, splineValF;
    // I6/P2-4：fill() 共享 buffer 上传+dispatch 无互斥——worldgen 池 worker 多线程并发调
    // fillOneChunkCore → 多线程同时 fill() 会驱动层崩溃（block_probe I7 实测 0xC0000005 @ nvtfi）。
    // 串行化 GPU 访问（正确性优先；吞吐影响后续评估——批量大时锁竞争可接受）。
    std::mutex fillMtx;

    void ensureBuffers(int n);
};

void GpuDensityEngine::Impl::ensureBuffers(int n) {
    if (n <= curCap && coordBuf.buffer) return;
    VkDevice dev = rt->device();
    VkRuntime::destroyBuffer(dev, coordBuf); VkRuntime::destroyBuffer(dev, permBuf);
    VkRuntime::destroyBuffer(dev, splitBuf); VkRuntime::destroyBuffer(dev, outBuf); VkRuntime::destroyBuffer(dev, valBuf);
    VkRuntime::destroyBuffer(dev, npBuf); VkRuntime::destroyBuffer(dev, locBuf);
    VkRuntime::destroyBuffer(dev, derBuf); VkRuntime::destroyBuffer(dev, vfBuf);
    VkRuntime::destroyBuffer(dev, vkBuf); VkRuntime::destroyBuffer(dev, vnBuf);
    coordBuf = rt->createBuffer((VkDeviceSize)n * 3 * sizeof(int32_t));
    permBuf = rt->createBuffer(permSize);
    splitBuf = rt->createBuffer((VkDeviceSize)n * backend.splitTotal * sizeof(float));
    outBuf = rt->createBuffer((VkDeviceSize)n * sizeof(float));
    valBuf = rt->createBuffer((VkDeviceSize)n * backend.perSample * sizeof(float));
    npBuf = rt->createBuffer((VkDeviceSize)splineNodePack.size() * sizeof(int32_t));
    locBuf = rt->createBuffer((VkDeviceSize)splineLocs.size() * sizeof(float));
    derBuf = rt->createBuffer((VkDeviceSize)splineDers.size() * sizeof(float));
    vfBuf = rt->createBuffer((VkDeviceSize)splineValF.size() * sizeof(float));
    vkBuf = rt->createBuffer((VkDeviceSize)splineValKind.size() * sizeof(int32_t));
    vnBuf = rt->createBuffer((VkDeviceSize)splineValNode.size() * sizeof(int32_t));
    // 上传常量（perm + spline 表，seed 级）
    rt->upload(permBuf, perm.data(), permSize);
    rt->upload(npBuf, splineNodePack.data(), splineNodePack.size() * sizeof(int32_t));
    rt->upload(locBuf, splineLocs.data(), splineLocs.size() * sizeof(float));
    rt->upload(derBuf, splineDers.data(), splineDers.size() * sizeof(float));
    rt->upload(vfBuf, splineValF.data(), splineValF.size() * sizeof(float));
    rt->upload(vkBuf, splineValKind.data(), splineValKind.size() * sizeof(int32_t));
    rt->upload(vnBuf, splineValNode.data(), splineValNode.size() * sizeof(int32_t));
    // descriptor set（binding 0,1,3,4,5 + splineBindBase..+5；P2-2 从生成器取）
    int wb[11] = {0, 1, 3, 4, 5,
                  backend.splineBindBase + 0, backend.splineBindBase + 1, backend.splineBindBase + 2,
                  backend.splineBindBase + 3, backend.splineBindBase + 4, backend.splineBindBase + 5};
    VkRuntime::Buffer bufs[11] = {coordBuf, permBuf, outBuf, splitBuf, valBuf, npBuf, locBuf, derBuf, vfBuf, vkBuf, vnBuf};
    VkDeviceSize sizes[11] = {
        (VkDeviceSize)n * 3 * sizeof(int32_t), permSize, (VkDeviceSize)n * sizeof(float),
        (VkDeviceSize)n * backend.splitTotal * sizeof(float), (VkDeviceSize)n * backend.perSample * sizeof(float),
        (VkDeviceSize)splineNodePack.size() * sizeof(int32_t), (VkDeviceSize)splineLocs.size() * sizeof(float),
        (VkDeviceSize)splineDers.size() * sizeof(float), (VkDeviceSize)splineValF.size() * sizeof(float),
        (VkDeviceSize)splineValKind.size() * sizeof(int32_t), (VkDeviceSize)splineValNode.size() * sizeof(int32_t)};
    ds = rt->makeDescriptorSet<11>(bufs, wb, sizes, 11);
    curCap = n;
}

GpuDensityEngine::GpuDensityEngine(uint64_t seed, const std::string& spvPath) : m(std::make_unique<Impl>()) {
    auto& im = *m;
    // 1. CpuBackend：noise samplers + perm + spline 数据（与 e2e 相同初始化）
    im.backend.init(seed);
    im.backend.collectPerm(im.perm);
    // 2. Vulkan 初始化 + pipeline 编译（~70-100s，一次性）
    im.rt = std::make_unique<VkRuntime>();
    im.rt->init();
    std::fprintf(stderr, "[GPU] compiling pipeline (one-time ~70-100s)...\n");
    im.rt->createPipeline(spvPath);
    std::fprintf(stderr, "[GPU] pipeline ready, splitTotal=%d perSample=%d splineBindBase=%d\n",
                 im.backend.splitTotal, im.backend.perSample, im.backend.splineBindBase);
    // 3. seed 级常量（perm + spline 表）
    im.permSize = (VkDeviceSize)im.perm.size() * sizeof(uint32_t);
    im.splineNodePack.assign(im.backend.splineNodePack.begin(), im.backend.splineNodePack.end());
    im.splineLocs.assign(im.backend.splineLocs.begin(), im.backend.splineLocs.end());
    im.splineDers.assign(im.backend.splineDers.begin(), im.backend.splineDers.end());
    im.splineValF.assign(im.backend.splineValF.begin(), im.backend.splineValF.end());
    im.splineValKind.assign(im.backend.splineValKind.begin(), im.backend.splineValKind.end());
    im.splineValNode.assign(im.backend.splineValNode.begin(), im.backend.splineValNode.end());
}

GpuDensityEngine::~GpuDensityEngine() {
    if (m && m->rt) {
        VkDevice dev = m->rt->device();
        VkRuntime::destroyBuffer(dev, m->coordBuf); VkRuntime::destroyBuffer(dev, m->permBuf);
        VkRuntime::destroyBuffer(dev, m->splitBuf); VkRuntime::destroyBuffer(dev, m->outBuf); VkRuntime::destroyBuffer(dev, m->valBuf);
        VkRuntime::destroyBuffer(dev, m->npBuf); VkRuntime::destroyBuffer(dev, m->locBuf);
        VkRuntime::destroyBuffer(dev, m->derBuf); VkRuntime::destroyBuffer(dev, m->vfBuf);
        VkRuntime::destroyBuffer(dev, m->vkBuf); VkRuntime::destroyBuffer(dev, m->vnBuf);
        m->rt->destroy();
    }
}

void GpuDensityEngine::fill(const int32_t* coords, int n, float* out) {
    auto& im = *m;
    std::lock_guard<std::mutex> lk(im.fillMtx);  // I6/P2-4：串行化 GPU 访问（多线程 fillOneChunkCore 并发）
    im.ensureBuffers(n);
    // CPU 拆分坐标（double 精度 → int32 格点 + float 小数）
    std::vector<float> splitCoord((size_t)n * im.backend.splitTotal);
    for (int s = 0; s < n; s++)
        im.backend.split(coords[3*s+0], coords[3*s+1], coords[3*s+2], splitCoord.data() + (size_t)s * im.backend.splitTotal);
    im.rt->upload(im.coordBuf, coords, (VkDeviceSize)n * 3 * sizeof(int32_t));
    im.rt->upload(im.splitBuf, splitCoord.data(), (VkDeviceSize)n * im.backend.splitTotal * sizeof(float));
    im.rt->dispatch(im.ds, (uint32_t)n);
    im.rt->readback(im.outBuf, out, (VkDeviceSize)n * sizeof(float));
}

float GpuDensityEngine::sample(int x, int y, int z) {
    int32_t c[3] = {x, y, z};
    float v = 0.0f;
    fill(c, 1, &v);
    return v;
}

int GpuDensityEngine::splitTotal() const { return m->backend.splitTotal; }
int GpuDensityEngine::perSample() const { return m->backend.perSample; }
int GpuDensityEngine::splineBindBase() const { return m->backend.splineBindBase; }
