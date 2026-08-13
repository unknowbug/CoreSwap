// batch_probe.cpp —— 批量 dispatch 吞吐测量：GPU 算 base_3d_noise 的 chunk/s
// 复用 octave shader（2+2 octave），批量算 M chunk × 每 chunk 7350 角点，测 GPU kernel 时间 + 吞吐
#include <vulkan/vulkan.h>

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>
#include <fstream>
#include <cmath>
#include <chrono>

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

using namespace std::chrono;

int main() {
    const uint32_t POINTS_PER_CHUNK = 7350;  // InterpolatedDF buildGrid 1225 × 6 实例

    // ---- Vulkan 初始化（同前）----
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

    auto spv = loadSpv("octave.spv");
    VkShaderModuleCreateInfo smCI{}; smCI.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO; smCI.codeSize = spv.size()*4; smCI.pCode = spv.data();
    VkShaderModule shader; CHECK_VK(vkCreateShaderModule(device, &smCI, nullptr, &shader));
    VkDescriptorSetLayoutBinding bindings[3]{};
    for (int b = 0; b < 3; b++) { bindings[b].binding = b; bindings[b].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; bindings[b].descriptorCount = 1; bindings[b].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT; }
    VkDescriptorSetLayoutCreateInfo dslCI{}; dslCI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO; dslCI.bindingCount = 3; dslCI.pBindings = bindings;
    VkDescriptorSetLayout dsl; CHECK_VK(vkCreateDescriptorSetLayout(device, &dslCI, nullptr, &dsl));
    VkPipelineLayoutCreateInfo plCI{}; plCI.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO; plCI.setLayoutCount = 1; plCI.pSetLayouts = &dsl;
    VkPipelineLayout pipelineLayout; CHECK_VK(vkCreatePipelineLayout(device, &plCI, nullptr, &pipelineLayout));
    VkComputePipelineCreateInfo cpCI{}; cpCI.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    cpCI.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO; cpCI.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT; cpCI.stage.module = shader; cpCI.stage.pName = "main"; cpCI.layout = pipelineLayout;
    VkPipeline pipeline; CHECK_VK(vkCreateComputePipelines(device, VK_NULL_HANDLE, 1, &cpCI, nullptr, &pipeline));

    VkDeviceSize paramsSize = 4*256*4 + 2*4 + 4 + 4 + 4;
    auto makeBuffer = [&](VkDeviceSize size, VkBuffer* buf, VkDeviceMemory* mem) {
        VkBufferCreateInfo bCI{}; bCI.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO; bCI.size = size; bCI.usage = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT; bCI.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
        CHECK_VK(vkCreateBuffer(device, &bCI, nullptr, buf));
        VkMemoryRequirements req; vkGetBufferMemoryRequirements(device, *buf, &req);
        VkPhysicalDeviceMemoryProperties mp; vkGetPhysicalDeviceMemoryProperties(physDev, &mp);
        uint32_t ti = UINT32_MAX; for (uint32_t i = 0; i < mp.memoryTypeCount; i++) if ((req.memoryTypeBits & (1u<<i)) && (mp.memoryTypes[i].propertyFlags & (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT|VK_MEMORY_PROPERTY_HOST_COHERENT_BIT))) { ti = i; break; }
        VkMemoryAllocateInfo aI{}; aI.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO; aI.allocationSize = req.size; aI.memoryTypeIndex = ti;
        CHECK_VK(vkAllocateMemory(device, &aI, nullptr, mem)); CHECK_VK(vkBindBufferMemory(device, *buf, *mem, 0));
    };

    // 最大批量：M_MAX chunk
    const uint32_t M_MAX = 256;   // 256 chunk 批量
    const uint32_t N_MAX = POINTS_PER_CHUNK * M_MAX;  // 1881600 点

    VkBuffer paramsBuf, coordBuf, outBuf; VkDeviceMemory paramsMem, coordMem, outMem;
    VkDeviceSize coordSize = (VkDeviceSize)N_MAX * 24 * sizeof(float);  // 4 octave × 6
    VkDeviceSize outSize = (VkDeviceSize)N_MAX * sizeof(float);
    makeBuffer(paramsSize, &paramsBuf, &paramsMem);
    makeBuffer(coordSize, &coordBuf, &coordMem);
    makeBuffer(outSize, &outBuf, &outMem);

    // 填充 params + 坐标（假数据，只测吞吐）
    {
        void* m; CHECK_VK(vkMapMemory(device, paramsMem, 0, paramsSize, 0, &m));
        uint32_t* mp = (uint32_t*)m;
        for (int o = 0; o < 4; o++) for (int i = 0; i < 256; i++) mp[o*256+i] = (uint32_t)i;
        float* fp = (float*)(mp + 4*256); fp[0]=1.0f; fp[1]=1.0f; fp[2]=0.6667f; fp[3]=1.0f; fp[4]=1.01813f;
        vkUnmapMemory(device, paramsMem);
    }
    {
        void* m; CHECK_VK(vkMapMemory(device, coordMem, 0, coordSize, 0, &m));
        float* c = (float*)m;
        for (uint32_t i = 0; i < N_MAX * 24; i++) c[i] = (float)(i % 17) * 0.1f;
        vkUnmapMemory(device, coordMem);
    }

    VkDescriptorPoolSize poolSize{}; poolSize.type = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; poolSize.descriptorCount = 3;
    VkDescriptorPoolCreateInfo dpCI{}; dpCI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO; dpCI.maxSets = 1; dpCI.poolSizeCount = 1; dpCI.pPoolSizes = &poolSize;
    VkDescriptorPool dpool; CHECK_VK(vkCreateDescriptorPool(device, &dpCI, nullptr, &dpool));
    VkDescriptorSetAllocateInfo dsAI{}; dsAI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO; dsAI.descriptorPool = dpool; dsAI.descriptorSetCount = 1; dsAI.pSetLayouts = &dsl;
    VkDescriptorSet ds; CHECK_VK(vkAllocateDescriptorSets(device, &dsAI, &ds));
    VkDescriptorBufferInfo dbi[3]{{paramsBuf,0,paramsSize},{coordBuf,0,coordSize},{outBuf,0,outSize}};
    VkWriteDescriptorSet writes[3]{};
    for (int b = 0; b < 3; b++) { writes[b].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET; writes[b].dstSet = ds; writes[b].dstBinding = b; writes[b].descriptorCount = 1; writes[b].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; writes[b].pBufferInfo = &dbi[b]; }
    vkUpdateDescriptorSets(device, 3, writes, 0, nullptr);

    VkCommandPoolCreateInfo cpPoolCI{}; cpPoolCI.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO; cpPoolCI.queueFamilyIndex = computeFamily;
    VkCommandPool cmdPool; CHECK_VK(vkCreateCommandPool(device, &cpPoolCI, nullptr, &cmdPool));
    VkCommandBufferAllocateInfo cbAI{}; cbAI.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO; cbAI.commandPool = cmdPool; cbAI.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY; cbAI.commandBufferCount = 1;
    VkCommandBuffer cb; CHECK_VK(vkAllocateCommandBuffers(device, &cbAI, &cb));
    VkFenceCreateInfo fenceCI{}; fenceCI.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
    VkFence fence; CHECK_VK(vkCreateFence(device, &fenceCI, nullptr, &fence));

    std::printf("=== 批量 dispatch 吞吐（每 chunk %u 角点，2+2 octave）===\n", POINTS_PER_CHUNK);
    std::printf("%8s %12s %14s %14s\n", "chunks", "点数", "kernel时间", "吞吐(chunk/s)");

    uint32_t chunkCounts[] = {1, 4, 16, 64, 256};
    for (uint32_t M : chunkCounts) {
        uint32_t N = POINTS_PER_CHUNK * M;
        VkCommandBufferBeginInfo begin{}; begin.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
        CHECK_VK(vkBeginCommandBuffer(cb, &begin));
        vkCmdBindPipeline(cb, VK_PIPELINE_BIND_POINT_COMPUTE, pipeline);
        vkCmdBindDescriptorSets(cb, VK_PIPELINE_BIND_POINT_COMPUTE, pipelineLayout, 0, 1, &ds, 0, nullptr);
        vkCmdDispatch(cb, (N + 255) / 256, 1, 1);
        CHECK_VK(vkEndCommandBuffer(cb));
        VkSubmitInfo submit{}; submit.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO; submit.commandBufferCount = 1; submit.pCommandBuffers = &cb;

        // 多次测取平均
        const int ITERS = 20;
        vkQueueSubmit(queue, 1, &submit, fence); vkWaitForFences(device, 1, &fence, VK_TRUE, UINT64_MAX); vkResetFences(device, 1, &fence);  // warmup
        auto t0 = high_resolution_clock::now();
        for (int it = 0; it < ITERS; it++) {
            CHECK_VK(vkQueueSubmit(queue, 1, &submit, fence));
            CHECK_VK(vkWaitForFences(device, 1, &fence, VK_TRUE, UINT64_MAX));
            CHECK_VK(vkResetFences(device, 1, &fence));
        }
        auto t1 = high_resolution_clock::now();
        double perSubmit = duration_cast<microseconds>(t1 - t0).count() / (double)ITERS;
        double chunkPerSec = M / (perSubmit * 1e-6);
        std::printf("%8u %12u %12.1f us %14.0f\n", M, N, perSubmit, chunkPerSec);
    }

    // cleanup
    vkDestroyFence(device, fence, nullptr); vkDestroyCommandPool(device, cmdPool, nullptr); vkDestroyDescriptorPool(device, dpool, nullptr);
    vkFreeMemory(device, paramsMem, nullptr); vkFreeMemory(device, coordMem, nullptr); vkFreeMemory(device, outMem, nullptr);
    vkDestroyBuffer(device, paramsBuf, nullptr); vkDestroyBuffer(device, coordBuf, nullptr); vkDestroyBuffer(device, outBuf, nullptr);
    vkDestroyPipeline(device, pipeline, nullptr); vkDestroyPipelineLayout(device, pipelineLayout, nullptr); vkDestroyDescriptorSetLayout(device, dsl, nullptr); vkDestroyShaderModule(device, shader, nullptr);
    vkDestroyDevice(device, nullptr); vkDestroyInstance(instance, nullptr);
    std::printf("[done]\n");
    return 0;
}
