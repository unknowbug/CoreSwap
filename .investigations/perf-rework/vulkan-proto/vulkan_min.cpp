// vulkan_min.cpp —— 最小 Vulkan compute 原型
// 验证：storage buffer in/out 数据流 + FP32 精度（GPU float vs CPU double）
// 构建：glslc compute.comp -o compute.spv && cl vulkan_min.cpp /I <SDK>\Include vulkan-1.lib
#include <vulkan/vulkan.h>

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>
#include <fstream>
#include <cmath>

#define CHECK_VK(fn) do { VkResult _r = (fn); if (_r != VK_SUCCESS) { \
    std::fprintf(stderr, "VK error %d at %s:%d (%s)\n", _r, __FILE__, __LINE__, #fn); \
    std::exit(1); } } while (0)

static std::vector<uint32_t> loadSpv(const char* path) {
    std::ifstream f(path, std::ios::binary | std::ios::ate);
    if (!f) { std::fprintf(stderr, "cannot open %s\n", path); std::exit(1); }
    std::streamsize n = f.tellg();
    f.seekg(0);
    std::vector<uint32_t> code((size_t)n / 4);
    f.read((char*)code.data(), n);
    return code;
}

int main() {
    // 输入数据：N 个坐标点，每个 (x,y,z) 折叠后 float 坐标
    const uint32_t N = 1024;
    std::vector<float> coords(3 * N);
    for (uint32_t i = 0; i < N; i++) {
        coords[3*i+0] = (float)((int)(i % 32) - 16) * 0.125f;
        coords[3*i+1] = (float)((int)(i / 32) - 16) * 0.125f;
        coords[3*i+2] = 0.5f;
    }

    // ---- 1. instance ----
    VkApplicationInfo appInfo{};
    appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
    appInfo.pApplicationName = "vulkan_min";
    appInfo.apiVersion = VK_API_VERSION_1_3;
    VkInstanceCreateInfo instCI{};
    instCI.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
    instCI.pApplicationInfo = &appInfo;
    VkInstance instance;
    CHECK_VK(vkCreateInstance(&instCI, nullptr, &instance));

    // ---- 2. physical device（选第一个，RTX 4060） ----
    uint32_t devCount = 0;
    vkEnumeratePhysicalDevices(instance, &devCount, nullptr);
    std::vector<VkPhysicalDevice> phys(devCount);
    vkEnumeratePhysicalDevices(instance, &devCount, phys.data());
    VkPhysicalDevice physDev = phys[0];
    VkPhysicalDeviceProperties devProps;
    vkGetPhysicalDeviceProperties(physDev, &devProps);
    std::printf("[device] %s\n", devProps.deviceName);

    // compute queue family
    uint32_t qCount = 0;
    vkGetPhysicalDeviceQueueFamilyProperties(physDev, &qCount, nullptr);
    std::vector<VkQueueFamilyProperties> qProps(qCount);
    vkGetPhysicalDeviceQueueFamilyProperties(physDev, &qCount, qProps.data());
    uint32_t computeFamily = UINT32_MAX;
    for (uint32_t i = 0; i < qCount; i++) {
        if (qProps[i].queueFlags & VK_QUEUE_COMPUTE_BIT) { computeFamily = i; break; }
    }
    if (computeFamily == UINT32_MAX) { std::fprintf(stderr, "no compute queue\n"); return 1; }

    // ---- 3. logical device + queue ----
    float qPri = 1.0f;
    VkDeviceQueueCreateInfo qCI{};
    qCI.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
    qCI.queueFamilyIndex = computeFamily;
    qCI.queueCount = 1;
    qCI.pQueuePriorities = &qPri;
    VkDeviceCreateInfo devCI{};
    devCI.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
    devCI.queueCreateInfoCount = 1;
    devCI.pQueueCreateInfos = &qCI;
    VkDevice device;
    CHECK_VK(vkCreateDevice(physDev, &devCI, nullptr, &device));
    VkQueue queue;
    vkGetDeviceQueue(device, computeFamily, 0, &queue);

    // ---- 4. shader module + pipeline ----
    auto spv = loadSpv("compute.spv");
    VkShaderModuleCreateInfo smCI{};
    smCI.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
    smCI.codeSize = spv.size() * 4;
    smCI.pCode = spv.data();
    VkShaderModule shader;
    CHECK_VK(vkCreateShaderModule(device, &smCI, nullptr, &shader));

    VkDescriptorSetLayoutBinding bindings[2]{};
    bindings[0].binding = 0;
    bindings[0].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
    bindings[0].descriptorCount = 1;
    bindings[0].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
    bindings[1] = bindings[0];
    bindings[1].binding = 1;
    VkDescriptorSetLayoutCreateInfo dslCI{};
    dslCI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
    dslCI.bindingCount = 2;
    dslCI.pBindings = bindings;
    VkDescriptorSetLayout dsl;
    CHECK_VK(vkCreateDescriptorSetLayout(device, &dslCI, nullptr, &dsl));

    VkPipelineLayoutCreateInfo plCI{};
    plCI.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
    plCI.setLayoutCount = 1;
    plCI.pSetLayouts = &dsl;
    VkPipelineLayout pipelineLayout;
    CHECK_VK(vkCreatePipelineLayout(device, &plCI, nullptr, &pipelineLayout));

    VkComputePipelineCreateInfo cpCI{};
    cpCI.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    cpCI.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    cpCI.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT;
    cpCI.stage.module = shader;
    cpCI.stage.pName = "main";
    cpCI.layout = pipelineLayout;
    VkPipeline pipeline;
    CHECK_VK(vkCreateComputePipelines(device, VK_NULL_HANDLE, 1, &cpCI, nullptr, &pipeline));

    // ---- 5. buffers ----
    VkDeviceSize inSize = coords.size() * sizeof(float);
    VkDeviceSize outSize = N * sizeof(float);
    auto makeBuffer = [&](VkDeviceSize size, VkBuffer* buf, VkDeviceMemory* mem) {
        VkBufferCreateInfo bCI{};
        bCI.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO;
        bCI.size = size;
        bCI.usage = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT;
        bCI.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
        CHECK_VK(vkCreateBuffer(device, &bCI, nullptr, buf));
        VkMemoryRequirements req;
        vkGetBufferMemoryRequirements(device, *buf, &req);
        VkPhysicalDeviceMemoryProperties memProps;
        vkGetPhysicalDeviceMemoryProperties(physDev, &memProps);
        uint32_t typeIdx = UINT32_MAX;
        for (uint32_t i = 0; i < memProps.memoryTypeCount; i++) {
            if ((req.memoryTypeBits & (1u << i)) &&
                (memProps.memoryTypes[i].propertyFlags & (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT))) {
                typeIdx = i; break;
            }
        }
        VkMemoryAllocateInfo aI{};
        aI.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
        aI.allocationSize = req.size;
        aI.memoryTypeIndex = typeIdx;
        CHECK_VK(vkAllocateMemory(device, &aI, nullptr, mem));
        CHECK_VK(vkBindBufferMemory(device, *buf, *mem, 0));
    };
    VkBuffer inBuf, outBuf;
    VkDeviceMemory inMem, outMem;
    makeBuffer(inSize, &inBuf, &inMem);
    makeBuffer(outSize, &outBuf, &outMem);

    // 上传输入
    void* mapped;
    CHECK_VK(vkMapMemory(device, inMem, 0, inSize, 0, &mapped));
    std::memcpy(mapped, coords.data(), inSize);
    vkUnmapMemory(device, inMem);

    // ---- 6. descriptor set ----
    VkDescriptorPoolSize poolSize{};
    poolSize.type = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
    poolSize.descriptorCount = 2;
    VkDescriptorPoolCreateInfo dpCI{};
    dpCI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO;
    dpCI.maxSets = 1;
    dpCI.poolSizeCount = 1;
    dpCI.pPoolSizes = &poolSize;
    VkDescriptorPool dpool;
    CHECK_VK(vkCreateDescriptorPool(device, &dpCI, nullptr, &dpool));
    VkDescriptorSetAllocateInfo dsAI{};
    dsAI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO;
    dsAI.descriptorPool = dpool;
    dsAI.descriptorSetCount = 1;
    dsAI.pSetLayouts = &dsl;
    VkDescriptorSet ds;
    CHECK_VK(vkAllocateDescriptorSets(device, &dsAI, &ds));

    VkDescriptorBufferInfo dbi[2]{};
    dbi[0].buffer = inBuf;  dbi[0].offset = 0; dbi[0].range = inSize;
    dbi[1].buffer = outBuf; dbi[1].offset = 0; dbi[1].range = outSize;
    VkWriteDescriptorSet writes[2]{};
    writes[0].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
    writes[0].dstSet = ds;
    writes[0].dstBinding = 0;
    writes[0].descriptorCount = 1;
    writes[0].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
    writes[0].pBufferInfo = &dbi[0];
    writes[1] = writes[0];
    writes[1].dstBinding = 1;
    writes[1].pBufferInfo = &dbi[1];
    vkUpdateDescriptorSets(device, 2, writes, 0, nullptr);

    // ---- 7. command buffer + dispatch ----
    VkCommandPoolCreateInfo cpPoolCI{};
    cpPoolCI.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
    cpPoolCI.queueFamilyIndex = computeFamily;
    VkCommandPool cmdPool;
    CHECK_VK(vkCreateCommandPool(device, &cpPoolCI, nullptr, &cmdPool));
    VkCommandBufferAllocateInfo cbAI{};
    cbAI.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
    cbAI.commandPool = cmdPool;
    cbAI.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
    cbAI.commandBufferCount = 1;
    VkCommandBuffer cb;
    CHECK_VK(vkAllocateCommandBuffers(device, &cbAI, &cb));

    VkCommandBufferBeginInfo begin{};
    begin.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    CHECK_VK(vkBeginCommandBuffer(cb, &begin));
    vkCmdBindPipeline(cb, VK_PIPELINE_BIND_POINT_COMPUTE, pipeline);
    vkCmdBindDescriptorSets(cb, VK_PIPELINE_BIND_POINT_COMPUTE, pipelineLayout, 0, 1, &ds, 0, nullptr);
    vkCmdDispatch(cb, (N + 255) / 256, 1, 1);
    CHECK_VK(vkEndCommandBuffer(cb));

    VkSubmitInfo submit{};
    submit.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
    submit.commandBufferCount = 1;
    submit.pCommandBuffers = &cb;
    VkFenceCreateInfo fenceCI{};
    fenceCI.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
    VkFence fence;
    CHECK_VK(vkCreateFence(device, &fenceCI, nullptr, &fence));
    CHECK_VK(vkQueueSubmit(queue, 1, &submit, fence));
    CHECK_VK(vkWaitForFences(device, 1, &fence, VK_TRUE, UINT64_MAX));

    // ---- 8. 读回 + 验证 ----
    CHECK_VK(vkMapMemory(device, outMem, 0, outSize, 0, &mapped));
    std::vector<float> out(N);
    std::memcpy(out.data(), mapped, outSize);
    vkUnmapMemory(device, outMem);

    // CPU double 参照（模拟分层方案的 FP64 侧）
    double maxDiff = 0.0;
    for (uint32_t i = 0; i < N; i++) {
        double x = coords[3*i+0], y = coords[3*i+1], z = coords[3*i+2];
        double ref = x * x + y * y + z * z;      // double
        float  gpu = out[i];                     // GPU float
        double diff = std::fabs((double)gpu - ref);
        if (diff > maxDiff) maxDiff = diff;
    }
    std::printf("[result] N=%u, GPU float vs CPU double maxDiff = %.3e\n", N, maxDiff);
    std::printf("[result] 样例: coord(0)=%.6f,%.6f,%.6f -> gpu=%f cpu=%f\n",
                coords[0], coords[1], coords[2], out[0],
                (float)((double)coords[0]*coords[0] + coords[1]*coords[1] + coords[2]*coords[2]));

    // cleanup
    vkDestroyFence(device, fence, nullptr);
    vkDestroyCommandPool(device, cmdPool, nullptr);
    vkDestroyDescriptorPool(device, dpool, nullptr);
    vkFreeMemory(device, inMem, nullptr);
    vkFreeMemory(device, outMem, nullptr);
    vkDestroyBuffer(device, inBuf, nullptr);
    vkDestroyBuffer(device, outBuf, nullptr);
    vkDestroyPipeline(device, pipeline, nullptr);
    vkDestroyPipelineLayout(device, pipelineLayout, nullptr);
    vkDestroyDescriptorSetLayout(device, dsl, nullptr);
    vkDestroyShaderModule(device, shader, nullptr);
    vkDestroyDevice(device, nullptr);
    vkDestroyInstance(instance, nullptr);
    std::printf("[done]\n");
    return 0;
}
