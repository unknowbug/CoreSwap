// dfc_e2e.cpp —— DFC 生成 shader 的端到端验证：真实 seed 生成 perm/origin → GPU 采样 vs CPU double
#include <vulkan/vulkan.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>
#include <fstream>
#include <cmath>

// 复用 CoreSwap noise.h 的 PerlinNoiseSampler/OctavePerlinNoiseSampler + XoroshiroRandom（用 /I 指定目录）
#include "noise.h"
#include "xoroshiro.h"
#include "md5.h"

#define CHECK_VK(fn) do { VkResult _r = (fn); if (_r != VK_SUCCESS) { \
    std::fprintf(stderr, "VK error %d at %s:%d (%s)\n", _r, __FILE__, __LINE__, #fn); \
    std::exit(1); } } while (0)

static std::vector<uint32_t> loadSpv(const char* path) {
    std::ifstream f(path, std::ios::binary | std::ios::ate);
    if (!f) { std::fprintf(stderr, "cannot open %s\n", path); std::exit(1); }
    std::streamsize n = f.tellg(); f.seekg(0);
    std::vector<uint32_t> code((size_t)n / 4); f.read((char*)code.data(), n);
    return code;
}

// CPU double 参照：InterpolatedNoiseSampler（对齐 vanilla 1.20.1）
static double maintainPrecision(double v) { return v - std::floor(v / 3.3554432E7 + 0.5) * 3.3554432E7; }
static double cpuInterpolated(const wg::OctavePerlinNoiseSampler& lower,
                              const wg::OctavePerlinNoiseSampler& upper,
                              const wg::OctavePerlinNoiseSampler& interp,
                              double x, double y, double z,
                              double scaledXzScale, double scaledYScale, double xzFactor, double yFactor, double smear) {
    double d = x * scaledXzScale;
    double e = y * scaledYScale;
    double f = z * scaledXzScale;
    double g = d / xzFactor, h = e / yFactor, i = f / xzFactor;
    double j = scaledYScale * smear, k = j / yFactor;
    double n = 0.0; double o = 1.0;
    for (int p = 0; p < 8; p++) {
        const wg::PerlinNoiseSampler* pn = interp.getOctave(p);
        if (pn) n += pn->sample(maintainPrecision(g*o), maintainPrecision(h*o), maintainPrecision(i*o), k*o, h*o) / o;
        o /= 2.0;
    }
    double q = (n / 10.0 + 1.0) / 2.0;
    bool bl = q >= 1.0, bl2 = q <= 0.0;
    double l = 0.0, m = 0.0; o = 1.0;
    for (int r = 0; r < 16; r++) {
        double s = maintainPrecision(d*o), t = maintainPrecision(e*o), u = maintainPrecision(f*o);
        double v = j*o;
        if (!bl) { const wg::PerlinNoiseSampler* pn = lower.getOctave(r); if (pn) l += pn->sample(s, t, u, v, e*o) / o; }
        if (!bl2) { const wg::PerlinNoiseSampler* pn = upper.getOctave(r); if (pn) m += pn->sample(s, t, u, v, e*o) / o; }
        o /= 2.0;
    }
    double w = std::min(1.0, std::max(0.0, q));
    return (l / 512.0 + w * (m / 512.0 - l / 512.0)) / 128.0;
}

int main() {
    // 参数（base_3d_noise 默认）
    const double scaledXzScale = 684.412 * 0.25;   // 171.103
    const double scaledYScale = 684.412 * 0.125;   // 85.5515
    const double xzFactor = 80.0, yFactor = 160.0, smear = 8.0;

    // 生成 old_blended 的 40 个 octave（对齐 vanilla：deriver.split("minecraft:terrain")）
    const uint64_t worldSeed = 8576294172403134396ULL;
    wg::XoroshiroRandom deriver(worldSeed);
    auto splitter = deriver.nextSplitter();
    wg::XoroshiroRandom rnd = splitter.split("minecraft:terrain");
    wg::OctavePerlinNoiseSampler lower(rnd, true, -15, wg::OctavePerlinNoiseSampler::rangeClosedAmplitudes(-15, 0));
    wg::OctavePerlinNoiseSampler upper(rnd, true, -15, wg::OctavePerlinNoiseSampler::rangeClosedAmplitudes(-15, 0));
    wg::OctavePerlinNoiseSampler interp(rnd, true, -7, wg::OctavePerlinNoiseSampler::rangeClosedAmplitudes(-7, 0));

    // 收集 perm/origin（shader octBase 布局：0..15 lower, 16..31 upper, 32..39 interp）
    std::vector<uint32_t> perm(40 * 256);
    std::vector<double> origin(40 * 3);
    auto collect = [&](const wg::OctavePerlinNoiseSampler& os, int octBase, int nOct) {
        for (int o = 0; o < nOct; o++) {
            const wg::PerlinNoiseSampler* pn = os.getOctave(o);
            if (!pn) continue;
            for (int i = 0; i < 256; i++) perm[(octBase + o) * 256 + i] = (uint32_t)pn->permutation[i];
            origin[(octBase + o) * 3 + 0] = pn->originX;
            origin[(octBase + o) * 3 + 1] = pn->originY;
            origin[(octBase + o) * 3 + 2] = pn->originZ;
        }
    };
    collect(lower, 0, 16);
    collect(upper, 16, 16);
    collect(interp, 32, 8);

    // 坐标（远坐标 + y 小数）
    const uint32_t N = 1024;
    std::vector<int32_t> coords(3 * N);
    for (uint32_t i = 0; i < N; i++) {
        coords[3*i+0] = (int32_t)(30000000 + (i % 16) * 1);
        coords[3*i+1] = (int32_t)(64 + (i % 16) * 1);
        coords[3*i+2] = (int32_t)(30000000 + (i % 16) * 1);
    }

    // ---- Vulkan 初始化（fp64）----
    VkApplicationInfo appInfo{}; appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO; appInfo.apiVersion = VK_API_VERSION_1_3;
    VkInstanceCreateInfo instCI{}; instCI.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO; instCI.pApplicationInfo = &appInfo;
    VkInstance instance; CHECK_VK(vkCreateInstance(&instCI, nullptr, &instance));
    uint32_t devCount = 0; vkEnumeratePhysicalDevices(instance, &devCount, nullptr);
    std::vector<VkPhysicalDevice> phys(devCount); vkEnumeratePhysicalDevices(instance, &devCount, phys.data());
    VkPhysicalDevice physDev = phys[0];
    VkPhysicalDeviceProperties devProps; vkGetPhysicalDeviceProperties(physDev, &devProps);
    std::printf("[device] %s\n", devProps.deviceName);
    VkPhysicalDeviceFeatures feat{}; vkGetPhysicalDeviceFeatures(physDev, &feat);
    if (!feat.shaderFloat64) { std::fprintf(stderr, "fp64 not supported\n"); return 1; }
    uint32_t qCount = 0; vkGetPhysicalDeviceQueueFamilyProperties(physDev, &qCount, nullptr);
    std::vector<VkQueueFamilyProperties> qProps(qCount); vkGetPhysicalDeviceQueueFamilyProperties(physDev, &qCount, qProps.data());
    uint32_t computeFamily = UINT32_MAX;
    for (uint32_t i = 0; i < qCount; i++) if (qProps[i].queueFlags & VK_QUEUE_COMPUTE_BIT) { computeFamily = i; break; }
    float qPri = 1.0f;
    VkDeviceQueueCreateInfo qCI{}; qCI.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO; qCI.queueFamilyIndex = computeFamily; qCI.queueCount = 1; qCI.pQueuePriorities = &qPri;
    VkPhysicalDeviceFeatures feat2{}; feat2.shaderFloat64 = VK_TRUE;
    VkDeviceCreateInfo devCI{}; devCI.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO; devCI.queueCreateInfoCount = 1; devCI.pQueueCreateInfos = &qCI; devCI.pEnabledFeatures = &feat2;
    VkDevice device; CHECK_VK(vkCreateDevice(physDev, &devCI, nullptr, &device));
    VkQueue queue; vkGetDeviceQueue(device, computeFamily, 0, &queue);

    auto spv = loadSpv("base_3d_noise.spv");
    VkShaderModuleCreateInfo smCI{}; smCI.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO; smCI.codeSize = spv.size()*4; smCI.pCode = spv.data();
    VkShaderModule shader; CHECK_VK(vkCreateShaderModule(device, &smCI, nullptr, &shader));
    VkDescriptorSetLayoutBinding bindings[4]{};
    for (int b = 0; b < 4; b++) { bindings[b].binding = b; bindings[b].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; bindings[b].descriptorCount = 1; bindings[b].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT; }
    VkDescriptorSetLayoutCreateInfo dslCI{}; dslCI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO; dslCI.bindingCount = 4; dslCI.pBindings = bindings;
    VkDescriptorSetLayout dsl; CHECK_VK(vkCreateDescriptorSetLayout(device, &dslCI, nullptr, &dsl));
    VkPipelineLayoutCreateInfo plCI{}; plCI.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO; plCI.setLayoutCount = 1; plCI.pSetLayouts = &dsl;
    VkPipelineLayout pipelineLayout; CHECK_VK(vkCreatePipelineLayout(device, &plCI, nullptr, &pipelineLayout));
    VkComputePipelineCreateInfo cpCI{}; cpCI.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    cpCI.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO; cpCI.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT; cpCI.stage.module = shader; cpCI.stage.pName = "main"; cpCI.layout = pipelineLayout;
    VkPipeline pipeline; CHECK_VK(vkCreateComputePipelines(device, VK_NULL_HANDLE, 1, &cpCI, nullptr, &pipeline));

    VkDeviceSize coordSize = coords.size() * sizeof(int32_t);
    VkDeviceSize permSize = perm.size() * sizeof(uint32_t);
    VkDeviceSize originSize = origin.size() * sizeof(double);
    VkDeviceSize outSize = N * sizeof(float);
    auto makeBuffer = [&](VkDeviceSize size, VkBuffer* buf, VkDeviceMemory* mem) {
        VkBufferCreateInfo bCI{}; bCI.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO; bCI.size = size; bCI.usage = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT; bCI.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
        CHECK_VK(vkCreateBuffer(device, &bCI, nullptr, buf));
        VkMemoryRequirements req; vkGetBufferMemoryRequirements(device, *buf, &req);
        VkPhysicalDeviceMemoryProperties mp; vkGetPhysicalDeviceMemoryProperties(physDev, &mp);
        uint32_t ti = UINT32_MAX; for (uint32_t i = 0; i < mp.memoryTypeCount; i++) if ((req.memoryTypeBits & (1u<<i)) && (mp.memoryTypes[i].propertyFlags & (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT|VK_MEMORY_PROPERTY_HOST_COHERENT_BIT))) { ti = i; break; }
        VkMemoryAllocateInfo aI{}; aI.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO; aI.allocationSize = req.size; aI.memoryTypeIndex = ti;
        CHECK_VK(vkAllocateMemory(device, &aI, nullptr, mem)); CHECK_VK(vkBindBufferMemory(device, *buf, *mem, 0));
    };
    VkBuffer coordBuf, permBuf, originBuf, outBuf; VkDeviceMemory coordMem, permMem, originMem, outMem;
    makeBuffer(coordSize, &coordBuf, &coordMem); makeBuffer(permSize, &permBuf, &permMem);
    makeBuffer(originSize, &originBuf, &originMem); makeBuffer(outSize, &outBuf, &outMem);
    { void* m; CHECK_VK(vkMapMemory(device, coordMem, 0, coordSize, 0, &m)); std::memcpy(m, coords.data(), coordSize); vkUnmapMemory(device, coordMem); }
    { void* m; CHECK_VK(vkMapMemory(device, permMem, 0, permSize, 0, &m)); std::memcpy(m, perm.data(), permSize); vkUnmapMemory(device, permMem); }
    { void* m; CHECK_VK(vkMapMemory(device, originMem, 0, originSize, 0, &m)); std::memcpy(m, origin.data(), originSize); vkUnmapMemory(device, originMem); }

    VkDescriptorPoolSize poolSize{}; poolSize.type = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; poolSize.descriptorCount = 4;
    VkDescriptorPoolCreateInfo dpCI{}; dpCI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO; dpCI.maxSets = 1; dpCI.poolSizeCount = 1; dpCI.pPoolSizes = &poolSize;
    VkDescriptorPool dpool; CHECK_VK(vkCreateDescriptorPool(device, &dpCI, nullptr, &dpool));
    VkDescriptorSetAllocateInfo dsAI{}; dsAI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO; dsAI.descriptorPool = dpool; dsAI.descriptorSetCount = 1; dsAI.pSetLayouts = &dsl;
    VkDescriptorSet ds; CHECK_VK(vkAllocateDescriptorSets(device, &dsAI, &ds));
    VkDescriptorBufferInfo dbi[4]{{coordBuf,0,coordSize},{permBuf,0,permSize},{originBuf,0,originSize},{outBuf,0,outSize}};
    VkWriteDescriptorSet writes[4]{};
    for (int b = 0; b < 4; b++) { writes[b].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET; writes[b].dstSet = ds; writes[b].dstBinding = b; writes[b].descriptorCount = 1; writes[b].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; writes[b].pBufferInfo = &dbi[b]; }
    vkUpdateDescriptorSets(device, 4, writes, 0, nullptr);

    VkCommandPoolCreateInfo cpPoolCI{}; cpPoolCI.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO; cpPoolCI.queueFamilyIndex = computeFamily;
    VkCommandPool cmdPool; CHECK_VK(vkCreateCommandPool(device, &cpPoolCI, nullptr, &cmdPool));
    VkCommandBufferAllocateInfo cbAI{}; cbAI.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO; cbAI.commandPool = cmdPool; cbAI.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY; cbAI.commandBufferCount = 1;
    VkCommandBuffer cb; CHECK_VK(vkAllocateCommandBuffers(device, &cbAI, &cb));
    VkCommandBufferBeginInfo begin{}; begin.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    CHECK_VK(vkBeginCommandBuffer(cb, &begin));
    vkCmdBindPipeline(cb, VK_PIPELINE_BIND_POINT_COMPUTE, pipeline);
    vkCmdBindDescriptorSets(cb, VK_PIPELINE_BIND_POINT_COMPUTE, pipelineLayout, 0, 1, &ds, 0, nullptr);
    vkCmdDispatch(cb, (N + 255) / 256, 1, 1);
    CHECK_VK(vkEndCommandBuffer(cb));
    VkSubmitInfo submit{}; submit.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO; submit.commandBufferCount = 1; submit.pCommandBuffers = &cb;
    VkFenceCreateInfo fenceCI{}; fenceCI.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
    VkFence fence; CHECK_VK(vkCreateFence(device, &fenceCI, nullptr, &fence));
    CHECK_VK(vkQueueSubmit(queue, 1, &submit, fence));
    CHECK_VK(vkWaitForFences(device, 1, &fence, VK_TRUE, UINT64_MAX));

    { void* m; CHECK_VK(vkMapMemory(device, outMem, 0, outSize, 0, &m)); std::vector<float> out(N); std::memcpy(out.data(), m, outSize); vkUnmapMemory(device, outMem);
        double maxDiff = 0.0, sumDiff = 0.0; uint32_t maxIdx = 0;
        for (uint32_t i = 0; i < N; i++) {
            double ref = cpuInterpolated(lower, upper, interp, coords[3*i+0], coords[3*i+1], coords[3*i+2],
                                         scaledXzScale, scaledYScale, xzFactor, yFactor, smear);
            double diff = std::fabs((double)out[i] - ref);
            if (diff > maxDiff) { maxDiff = diff; maxIdx = i; }
            sumDiff += diff;
        }
        std::printf("[result] N=%u, DFC shader(真实seed) vs CPU double: maxDiff=%.3e avgDiff=%.3e\n", N, maxDiff, sumDiff / N);
        std::printf("[result] maxDiff @ i=%u: gpu=%.12f cpu=%.12f\n", maxIdx, out[maxIdx],
                    cpuInterpolated(lower, upper, interp, coords[3*maxIdx], coords[3*maxIdx+1], coords[3*maxIdx+2],
                                    scaledXzScale, scaledYScale, xzFactor, yFactor, smear));
    }

    vkDestroyFence(device, fence, nullptr); vkDestroyCommandPool(device, cmdPool, nullptr); vkDestroyDescriptorPool(device, dpool, nullptr);
    vkFreeMemory(device, coordMem, nullptr); vkFreeMemory(device, permMem, nullptr); vkFreeMemory(device, originMem, nullptr); vkFreeMemory(device, outMem, nullptr);
    vkDestroyBuffer(device, coordBuf, nullptr); vkDestroyBuffer(device, permBuf, nullptr); vkDestroyBuffer(device, originBuf, nullptr); vkDestroyBuffer(device, outBuf, nullptr);
    vkDestroyPipeline(device, pipeline, nullptr); vkDestroyPipelineLayout(device, pipelineLayout, nullptr); vkDestroyDescriptorSetLayout(device, dsl, nullptr); vkDestroyShaderModule(device, shader, nullptr);
    vkDestroyDevice(device, nullptr); vkDestroyInstance(instance, nullptr);
    std::printf("[done]\n");
    return 0;
}
