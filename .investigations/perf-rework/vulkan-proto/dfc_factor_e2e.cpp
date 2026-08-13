// dfc_factor_e2e.cpp —— factor 完整 DF 树端到端验证（spline 嵌套 + registry 引用 + NormalNoise）
#include <vulkan/vulkan.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>
#include <fstream>
#include <sstream>
#include <cmath>
#include <map>
#include "json.h"
#include "density.h"
#include "density_builder.h"
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

static std::string readFile(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    if (!f) throw std::runtime_error("cannot open " + path);
    std::stringstream ss; ss << f.rdbuf(); return ss.str();
}

// 噪声参数表（factor 依赖的 4 个）
static std::map<std::string, wg::DoublePerlinNoiseSampler::NoiseParameters> buildNoiseParams() {
    std::map<std::string, wg::DoublePerlinNoiseSampler::NoiseParameters> m;
    auto add = [&](const char* key, int32_t oct, std::initializer_list<double> amps) {
        m[std::string("minecraft:") + key] = wg::DoublePerlinNoiseSampler::NoiseParameters{oct, std::vector<double>(amps)};
    };
    add("continentalness", -9, {1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0});
    add("offset", -3, {1.0, 1.0, 1.0, 0.0});
    add("erosion", -9, {1.0, 1.0, 0.0, 1.0, 1.0});
    add("ridge", -7, {1.0, 2.0, 1.0, 0.0, 0.0, 0.0});
    return m;
}

static void collectNormal(const wg::DoublePerlinNoiseSampler& dn, int octBase,
                          std::vector<uint32_t>& perm, std::vector<double>& origin) {
    int n = (int)dn.firstSampler.octaveSamplers.size();
    for (int i = 0; i < n; i++) {
        const wg::PerlinNoiseSampler* pn = dn.firstSampler.octaveSamplers[i].get();
        if (pn) {
            for (int k = 0; k < 256; k++) perm[(octBase + i) * 256 + k] = (uint32_t)pn->permutation[k];
            origin[(octBase + i) * 3 + 0] = pn->originX; origin[(octBase + i) * 3 + 1] = pn->originY; origin[(octBase + i) * 3 + 2] = pn->originZ;
        }
        pn = dn.secondSampler.octaveSamplers[i].get();
        if (pn) {
            for (int k = 0; k < 256; k++) perm[(octBase + n + i) * 256 + k] = (uint32_t)pn->permutation[k];
            origin[(octBase + n + i) * 3 + 0] = pn->originX; origin[(octBase + n + i) * 3 + 1] = pn->originY; origin[(octBase + n + i) * 3 + 2] = pn->originZ;
        }
    }
}

int main() {
    const uint64_t worldSeed = 8576294172403134396ULL;
    auto noiseParams = buildNoiseParams();
    wg::DensityBuilder builder(worldSeed, noiseParams);

    // externalLoader：加载 registry 引用
    std::string dfDir = "E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/";
    builder.externalLoader = [&](const std::string& fullRef, const std::string& name) -> wg::DF {
        std::string path = dfDir + name + ".json";
        std::ifstream probe(path);
        if (!probe.good()) return nullptr;
        return builder.parseFile(fullRef, readFile(path));
    };
    // 注册 factor 依赖的 DF（continents/erosion/ridges/ridges_folded + factor）
    for (const char* f : {"continents", "erosion", "ridges", "ridges_folded", "factor"}) {
        builder.registerFunction(std::string("minecraft:overworld/") + f, std::make_shared<wg::DensityBuilder::LazyRef>());
    }
    for (const char* f : {"continents", "erosion", "ridges", "ridges_folded", "factor"}) {
        auto df = builder.parseFile(std::string("minecraft:overworld/") + f, readFile(dfDir + f + ".json"));
        builder.registerFunction(std::string("minecraft:overworld/") + f, df);
    }
    wg::DF factor = builder.getRegistryEntry("minecraft:overworld/factor");
    std::printf("factor DF built\n");

    // 收集 4 个噪声 perm/origin（octBase 布局：continentalness 0, offset 18, erosion 26, ridge 36）
    auto rd = builder.randomDeriverPublic();
    wg::DoublePerlinNoiseSampler continentalness(rd.split("minecraft:continentalness"), noiseParams["minecraft:continentalness"]);
    wg::DoublePerlinNoiseSampler offset(rd.split("minecraft:offset"), noiseParams["minecraft:offset"]);
    wg::DoublePerlinNoiseSampler erosion(rd.split("minecraft:erosion"), noiseParams["minecraft:erosion"]);
    wg::DoublePerlinNoiseSampler ridge(rd.split("minecraft:ridge"), noiseParams["minecraft:ridge"]);
    const int totalOct = 18 + 8 + 10 + 12;
    std::vector<uint32_t> perm(totalOct * 256, 0);
    std::vector<double> origin(totalOct * 3, 0.0);
    collectNormal(continentalness, 0, perm, origin);
    collectNormal(offset, 18, perm, origin);
    collectNormal(erosion, 26, perm, origin);
    collectNormal(ridge, 36, perm, origin);

    // 坐标（近坐标，避免远坐标的坐标精度影响 spline）
    const uint32_t N = 1024;
    std::vector<int32_t> coords(3 * N);
    for (uint32_t i = 0; i < N; i++) {
        coords[3*i+0] = (int32_t)(728 + (i % 32));
        coords[3*i+1] = (int32_t)(-8 + (i / 32 % 16));
        coords[3*i+2] = (int32_t)(-428 + (i / 512));
    }

    // ---- Vulkan 初始化（fp64，4 buffer）----
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

    auto spv = loadSpv("factor.spv");
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
    VkPipeline pipeline; 
    std::fprintf(stderr, "[dbg] before vkCreateComputePipelines\n");
    CHECK_VK(vkCreateComputePipelines(device, VK_NULL_HANDLE, 1, &cpCI, nullptr, &pipeline));
    std::fprintf(stderr, "[dbg] pipeline created\n");

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
            wg::NoisePos pos{coords[3*i+0], coords[3*i+1], coords[3*i+2]};
            double ref = factor->sample(pos);
            double diff = std::fabs((double)out[i] - ref);
            if (diff > maxDiff) { maxDiff = diff; maxIdx = i; }
            sumDiff += diff;
        }
        std::printf("[result] N=%u, factor DFC shader vs CPU double: maxDiff=%.3e avgDiff=%.3e\n", N, maxDiff, sumDiff / N);
        wg::NoisePos p{coords[3*maxIdx], coords[3*maxIdx+1], coords[3*maxIdx+2]};
        std::printf("[result] maxDiff @ (%d,%d,%d): gpu=%.9f cpu=%.9f\n", p.x, p.y, p.z, out[maxIdx], factor->sample(p));
    }

    vkDestroyFence(device, fence, nullptr); vkDestroyCommandPool(device, cmdPool, nullptr); vkDestroyDescriptorPool(device, dpool, nullptr);
    vkFreeMemory(device, coordMem, nullptr); vkFreeMemory(device, permMem, nullptr); vkFreeMemory(device, originMem, nullptr); vkFreeMemory(device, outMem, nullptr);
    vkDestroyBuffer(device, coordBuf, nullptr); vkDestroyBuffer(device, permBuf, nullptr); vkDestroyBuffer(device, originBuf, nullptr); vkDestroyBuffer(device, outBuf, nullptr);
    vkDestroyPipeline(device, pipeline, nullptr); vkDestroyPipelineLayout(device, pipelineLayout, nullptr); vkDestroyDescriptorSetLayout(device, dsl, nullptr); vkDestroyShaderModule(device, shader, nullptr);
    vkDestroyDevice(device, nullptr); vkDestroyInstance(instance, nullptr);
    std::printf("[done]\n");
    return 0;
}
