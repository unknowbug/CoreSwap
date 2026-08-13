// dfc_interp_e2e.cpp —— interpolated（cell 三线性插值）端到端：单 pass 8 角点 delegate 采样
#include <vulkan/vulkan.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>
#include <fstream>
#include <cmath>
#include "noise.h"
#include "xoroshiro.h"
#include "density.h"

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
static double maintainPrecision(double v) { return v - (long)(v / 3.3554432E7 + 0.5) * 3.3554432E7; }

static void splitOctave(const wg::PerlinNoiseSampler* pn, double cx, double cy, double cz, float* out) {
    double ox = pn ? pn->originX : 0.0, oy = pn ? pn->originY : 0.0, oz = pn ? pn->originZ : 0.0;
    int ix = (int)std::floor(cx + ox), iy = (int)std::floor(cy + oy), iz = (int)std::floor(cz + oz);
    out[0] = (float)ix; out[1] = (float)iy; out[2] = (float)iz;
    out[3] = (float)(cx + ox - ix); out[4] = (float)(cy + oy - iy); out[5] = (float)(cz + oz - iz);
}

int main() {
    const uint64_t worldSeed = 8576294172403134396ULL;
    wg::XoroshiroRandom base(worldSeed);
    auto rd = base.nextSplitter();
    auto noodle = std::make_shared<wg::DoublePerlinNoiseSampler>(rd.split("minecraft:noodle"), wg::DoublePerlinNoiseSampler::NoiseParameters{-8, {1.0}});

    // CPU 参照：InterpolatedDF(delegate = NoiseDF(noodle))
    auto noiseDF = std::make_shared<wg::NoiseDF>(noodle, 1.0, 1.0);
    wg::InterpolatedDF interp(noiseDF, -64, 384);

    // 采样点：单 chunk（chunkX=45, chunkZ=-27），覆盖 cell 内部不同分数位置
    const uint32_t N = 1024;
    std::vector<int32_t> coords(3 * N);
    for (uint32_t i = 0; i < N; i++) {
        coords[3*i+0] = 720 + (i % 16);          // 720..735（chunk 45）
        coords[3*i+1] = -64 + (i / 16 % 16);     // -64..-49（gy 0..15）
        coords[3*i+2] = -432 + (i / 256);        // -432..-429
    }

    // CPU 预拆分：8 角点 × 12 值/角点
    const int SPLIT_PER_CORNER = 12, CORNER_COUNT = 8;
    std::vector<float> splitCoord((size_t)CORNER_COUNT * SPLIT_PER_CORNER * N);
    double lacunarity = std::pow(2.0, noodle->firstSampler.firstOctave);   // 2^-8 = 1/256
    for (uint32_t s = 0; s < N; s++) {
        int x = coords[3*s+0], y = coords[3*s+1], z = coords[3*s+2];
        int chunkX = (x >= 0 ? x : x - 15) / 16;   // floorDiv(x,16)
        int chunkZ = (z >= 0 ? z : z - 15) / 16;
        int gx = x - chunkX * 16, gy = y - (-64), gz = z - chunkZ * 16;
        int cx = gx / 4, cy = gy / 8, cz = gz / 4;
        // 8 角点（dx,dy,dz ∈ {0,1}）
        int dxs[8] = {0,1,0,1,0,1,0,1}, dys[8] = {0,0,1,1,0,0,1,1}, dzs[8] = {0,0,0,0,1,1,1,1};
        for (int c = 0; c < 8; c++) {
            double ax = (double)(chunkX * 16 + (cx + dxs[c]) * 4);
            double ay = (double)(-64 + (cy + dys[c]) * 8);
            double az = (double)(chunkZ * 16 + (cz + dzs[c]) * 4);
            float* out = &splitCoord[(size_t)s * CORNER_COUNT * SPLIT_PER_CORNER + c * SPLIT_PER_CORNER];
            splitOctave(noodle->firstSampler.octaveSamplers[0].get(),
                        maintainPrecision(ax*1.0*lacunarity), maintainPrecision(ay*1.0*lacunarity), maintainPrecision(az*1.0*lacunarity), &out[0]);
            splitOctave(noodle->secondSampler.octaveSamplers[0].get(),
                        maintainPrecision(ax*1.0181268882175227*lacunarity), maintainPrecision(ay*1.0181268882175227*lacunarity), maintainPrecision(az*1.0181268882175227*lacunarity), &out[6]);
        }
    }

    // perm 收集（noodle 单 octave，first + second）
    std::vector<uint32_t> perm(2 * 256, 0);
    const wg::PerlinNoiseSampler* pn = noodle->firstSampler.octaveSamplers[0].get();
    if (pn) for (int k = 0; k < 256; k++) perm[k] = (uint32_t)pn->permutation[k];
    pn = noodle->secondSampler.octaveSamplers[0].get();
    if (pn) for (int k = 0; k < 256; k++) perm[256 + k] = (uint32_t)pn->permutation[k];

    // ---- Vulkan ----
    VkApplicationInfo appInfo{}; appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO; appInfo.apiVersion = VK_API_VERSION_1_3;
    VkInstanceCreateInfo instCI{}; instCI.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO; instCI.pApplicationInfo = &appInfo;
    VkInstance instance; CHECK_VK(vkCreateInstance(&instCI, nullptr, &instance));
    uint32_t devCount = 0; vkEnumeratePhysicalDevices(instance, &devCount, nullptr);
    std::vector<VkPhysicalDevice> phys(devCount); vkEnumeratePhysicalDevices(instance, &devCount, phys.data());
    VkPhysicalDevice physDev = phys[0];
    VkPhysicalDeviceProperties devProps; vkGetPhysicalDeviceProperties(physDev, &devProps);
    std::printf("[device] %s\n", devProps.deviceName);
    uint32_t qCount = 0; vkGetPhysicalDeviceQueueFamilyProperties(physDev, &qCount, nullptr);
    std::vector<VkQueueFamilyProperties> qProps(qCount); vkGetPhysicalDeviceQueueFamilyProperties(physDev, &qCount, qProps.data());
    uint32_t computeFamily = UINT32_MAX;
    for (uint32_t i = 0; i < qCount; i++) if (qProps[i].queueFlags & VK_QUEUE_COMPUTE_BIT) { computeFamily = i; break; }
    float qPri = 1.0f;
    VkDeviceQueueCreateInfo qCI{}; qCI.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO; qCI.queueFamilyIndex = computeFamily; qCI.queueCount = 1; qCI.pQueuePriorities = &qPri;
    VkDeviceCreateInfo devCI{}; devCI.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO; devCI.queueCreateInfoCount = 1; devCI.pQueueCreateInfos = &qCI;
    VkDevice device; CHECK_VK(vkCreateDevice(physDev, &devCI, nullptr, &device));
    VkQueue queue; vkGetDeviceQueue(device, computeFamily, 0, &queue);

    auto spv = loadSpv("interp.spv");
    VkShaderModuleCreateInfo smCI{}; smCI.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO; smCI.codeSize = spv.size()*4; smCI.pCode = spv.data();
    VkShaderModule shader; CHECK_VK(vkCreateShaderModule(device, &smCI, nullptr, &shader));
    VkDescriptorSetLayoutBinding bindings[5]{};
    for (int b = 0; b < 5; b++) { bindings[b].binding = b; bindings[b].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; bindings[b].descriptorCount = 1; bindings[b].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT; }
    VkDescriptorSetLayoutCreateInfo dslCI{}; dslCI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO; dslCI.bindingCount = 5; dslCI.pBindings = bindings;
    VkDescriptorSetLayout dsl; CHECK_VK(vkCreateDescriptorSetLayout(device, &dslCI, nullptr, &dsl));
    VkPipelineLayoutCreateInfo plCI{}; plCI.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO; plCI.setLayoutCount = 1; plCI.pSetLayouts = &dsl;
    VkPipelineLayout pipelineLayout; CHECK_VK(vkCreatePipelineLayout(device, &plCI, nullptr, &pipelineLayout));
    VkComputePipelineCreateInfo cpCI{}; cpCI.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    cpCI.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO; cpCI.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT; cpCI.stage.module = shader; cpCI.stage.pName = "main"; cpCI.layout = pipelineLayout;
    VkPipeline pipeline;
    CHECK_VK(vkCreateComputePipelines(device, VK_NULL_HANDLE, 1, &cpCI, nullptr, &pipeline));
    std::printf("[dbg] pipeline created\n");

    VkDeviceSize coordSize = coords.size() * sizeof(int32_t);
    VkDeviceSize permSize = perm.size() * sizeof(uint32_t);
    VkDeviceSize splitSize = splitCoord.size() * sizeof(float);
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
    VkBuffer coordBuf, permBuf, splitBuf, outBuf; VkDeviceMemory coordMem, permMem, splitMem, outMem;
    makeBuffer(coordSize, &coordBuf, &coordMem); makeBuffer(permSize, &permBuf, &permMem);
    makeBuffer(splitSize, &splitBuf, &splitMem); makeBuffer(outSize, &outBuf, &outMem);
    { void* m; CHECK_VK(vkMapMemory(device, coordMem, 0, coordSize, 0, &m)); std::memcpy(m, coords.data(), coordSize); vkUnmapMemory(device, coordMem); }
    { void* m; CHECK_VK(vkMapMemory(device, permMem, 0, permSize, 0, &m)); std::memcpy(m, perm.data(), permSize); vkUnmapMemory(device, permMem); }
    { void* m; CHECK_VK(vkMapMemory(device, splitMem, 0, splitSize, 0, &m)); std::memcpy(m, splitCoord.data(), splitSize); vkUnmapMemory(device, splitMem); }

    VkDescriptorPoolSize poolSize{}; poolSize.type = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; poolSize.descriptorCount = 5;
    VkDescriptorPoolCreateInfo dpCI{}; dpCI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO; dpCI.maxSets = 1; dpCI.poolSizeCount = 1; dpCI.pPoolSizes = &poolSize;
    VkDescriptorPool dpool; CHECK_VK(vkCreateDescriptorPool(device, &dpCI, nullptr, &dpool));
    VkDescriptorSetAllocateInfo dsAI{}; dsAI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO; dsAI.descriptorPool = dpool; dsAI.descriptorSetCount = 1; dsAI.pSetLayouts = &dsl;
    VkDescriptorSet ds; CHECK_VK(vkAllocateDescriptorSets(device, &dsAI, &ds));
    VkDescriptorBufferInfo dbis[4]{{coordBuf,0,coordSize},{permBuf,0,permSize},{outBuf,0,outSize},{splitBuf,0,splitSize}};
    VkWriteDescriptorSet writes[4]{};
    int wb[4] = {0, 1, 3, 4};
    for (int b = 0; b < 4; b++) { writes[b].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET; writes[b].dstSet = ds; writes[b].dstBinding = wb[b]; writes[b].descriptorCount = 1; writes[b].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; writes[b].pBufferInfo = &dbis[b]; }
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
        double maxDiff = 0.0, sumDiff = 0.0;
        for (uint32_t i = 0; i < N; i++) {
            wg::NoisePos pos{ coords[3*i+0], coords[3*i+1], coords[3*i+2] };
            double ref = interp.sample(pos);
            double diff = std::fabs((double)out[i] - ref);
            if (diff > maxDiff) maxDiff = diff;
            sumDiff += diff;
        }
        std::printf("[result] N=%u, interpolated 8角点单pass GPU float vs CPU double: maxDiff=%.3e avgDiff=%.3e\n", N, maxDiff, sumDiff / N);
    }

    vkDestroyFence(device, fence, nullptr); vkDestroyCommandPool(device, cmdPool, nullptr); vkDestroyDescriptorPool(device, dpool, nullptr);
    vkFreeMemory(device, coordMem, nullptr); vkFreeMemory(device, permMem, nullptr); vkFreeMemory(device, splitMem, nullptr); vkFreeMemory(device, outMem, nullptr);
    vkDestroyBuffer(device, coordBuf, nullptr); vkDestroyBuffer(device, permBuf, nullptr); vkDestroyBuffer(device, splitBuf, nullptr); vkDestroyBuffer(device, outBuf, nullptr);
    vkDestroyPipeline(device, pipeline, nullptr); vkDestroyPipelineLayout(device, pipelineLayout, nullptr); vkDestroyDescriptorSetLayout(device, dsl, nullptr); vkDestroyShaderModule(device, shader, nullptr);
    vkDestroyDevice(device, nullptr); vkDestroyInstance(instance, nullptr);
    std::printf("[done]\n");
    return 0;
}
