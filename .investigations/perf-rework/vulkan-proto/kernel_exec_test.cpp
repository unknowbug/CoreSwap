// kernel_exec_test.cpp — 最小 kernel 执行计时（final_density.spv，1 work item）
// 决定性判别：TDR 是「kernel 死循环」还是「kernel 极慢」。
#include <vulkan/vulkan.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>
#include <fstream>
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
int main() {
    VkApplicationInfo ai{}; ai.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO; ai.apiVersion = VK_API_VERSION_1_3;
    VkInstanceCreateInfo ii{}; ii.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO; ii.pApplicationInfo = &ai;
    VkInstance inst; CHECK_VK(vkCreateInstance(&ii, nullptr, &inst));
    uint32_t dc = 0; vkEnumeratePhysicalDevices(inst, &dc, nullptr);
    std::vector<VkPhysicalDevice> phys(dc); vkEnumeratePhysicalDevices(inst, &dc, phys.data());
    VkPhysicalDevice pd = phys[0];
    uint32_t qc = 0; vkGetPhysicalDeviceQueueFamilyProperties(pd, &qc, nullptr);
    std::vector<VkQueueFamilyProperties> qp(qc); vkGetPhysicalDeviceQueueFamilyProperties(pd, &qc, qp.data());
    uint32_t cf = UINT32_MAX;
    for (uint32_t i = 0; i < qc; i++) if (qp[i].queueFlags & VK_QUEUE_COMPUTE_BIT) { cf = i; break; }
    float pri = 1.0f;
    VkDeviceQueueCreateInfo q{}; q.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO; q.queueFamilyIndex = cf; q.queueCount = 1; q.pQueuePriorities = &pri;
    VkDeviceCreateInfo di{}; di.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO; di.queueCreateInfoCount = 1; di.pQueueCreateInfos = &q;
    VkDevice dev; CHECK_VK(vkCreateDevice(pd, &di, nullptr, &dev));
    VkQueue queue; vkGetDeviceQueue(dev, cf, 0, &queue);

    auto spv = loadSpv("final_density.spv");
    VkShaderModuleCreateInfo sc{}; sc.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO; sc.codeSize = spv.size()*4; sc.pCode = spv.data();
    VkShaderModule sm; CHECK_VK(vkCreateShaderModule(dev, &sc, nullptr, &sm));
    VkDescriptorSetLayoutBinding b[6]{};
    for (int i = 0; i < 6; i++) { b[i].binding = i; b[i].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; b[i].descriptorCount = 1; b[i].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT; }
    VkDescriptorSetLayoutCreateInfo lc{}; lc.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO; lc.bindingCount = 6; lc.pBindings = b;
    VkDescriptorSetLayout l; CHECK_VK(vkCreateDescriptorSetLayout(dev, &lc, nullptr, &l));
    VkPipelineLayoutCreateInfo plc{}; plc.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO; plc.setLayoutCount = 1; plc.pSetLayouts = &l;
    VkPipelineLayout pl; CHECK_VK(vkCreatePipelineLayout(dev, &plc, nullptr, &pl));
    VkComputePipelineCreateInfo cp{}; cp.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    cp.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO; cp.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT; cp.stage.module = sm; cp.stage.pName = "main"; cp.layout = pl;
    VkPipeline p;
    auto t0 = std::chrono::steady_clock::now();
    VkResult r = vkCreateComputePipelines(dev, VK_NULL_HANDLE, 1, &cp, nullptr, &p);
    auto t1 = std::chrono::steady_clock::now();
    std::printf("[dbg] pipeline created in %.1fs (rc=%d)\n", std::chrono::duration<double>(t1 - t0).count(), (int)r);
    if (r != VK_SUCCESS) return 1;

    // 最小 buffers（空数据，只测 kernel 是否执行/卡死）
    const uint32_t N = 1;
    VkDeviceSize sz = 4 << 20;  // 4MB 每个（perm 需 335872×4B = 1.3MB）
    auto makeBuf = [&](VkBuffer* buf, VkDeviceMemory* mem) {
        VkBufferCreateInfo bc{}; bc.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO; bc.size = sz; bc.usage = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT; bc.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
        CHECK_VK(vkCreateBuffer(dev, &bc, nullptr, buf));
        VkMemoryRequirements req; vkGetBufferMemoryRequirements(dev, *buf, &req);
        VkPhysicalDeviceMemoryProperties mp; vkGetPhysicalDeviceMemoryProperties(pd, &mp);
        uint32_t ti = UINT32_MAX;
        for (uint32_t i = 0; i < mp.memoryTypeCount; i++) if ((req.memoryTypeBits & (1u<<i)) && (mp.memoryTypes[i].propertyFlags & (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT|VK_MEMORY_PROPERTY_HOST_COHERENT_BIT))) { ti = i; break; }
        VkMemoryAllocateInfo ac{}; ac.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO; ac.allocationSize = req.size; ac.memoryTypeIndex = ti;
        CHECK_VK(vkAllocateMemory(dev, &ac, nullptr, mem)); CHECK_VK(vkBindBufferMemory(dev, *buf, *mem, 0));
    };
    VkBuffer bufs[5]; VkDeviceMemory mems[5];
    for (int i = 0; i < 5; i++) makeBuf(&bufs[i], &mems[i]);
    VkDescriptorPoolSize ps{}; ps.type = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; ps.descriptorCount = 5;
    VkDescriptorPoolCreateInfo dpc{}; dpc.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO; dpc.maxSets = 1; dpc.poolSizeCount = 1; dpc.pPoolSizes = &ps;
    VkDescriptorPool pool; CHECK_VK(vkCreateDescriptorPool(dev, &dpc, nullptr, &pool));
    VkDescriptorSetAllocateInfo dsa{}; dsa.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO; dsa.descriptorPool = pool; dsa.descriptorSetCount = 1; dsa.pSetLayouts = &l;
    VkDescriptorSet ds; CHECK_VK(vkAllocateDescriptorSets(dev, &dsa, &ds));
    VkDescriptorBufferInfo db[5]{};
    int bindings[5] = {0, 1, 3, 4, 5};
    VkWriteDescriptorSet wr[5]{};
    for (int i = 0; i < 5; i++) { db[i] = {bufs[i], 0, sz}; wr[i].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET; wr[i].dstSet = ds; wr[i].dstBinding = bindings[i]; wr[i].descriptorCount = 1; wr[i].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; wr[i].pBufferInfo = &db[i]; }
    vkUpdateDescriptorSets(dev, 5, wr, 0, nullptr);

    VkCommandPoolCreateInfo cpc{}; cpc.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO; cpc.queueFamilyIndex = cf;
    VkCommandPool cmdPool; CHECK_VK(vkCreateCommandPool(dev, &cpc, nullptr, &cmdPool));
    VkCommandBufferAllocateInfo cba{}; cba.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO; cba.commandPool = cmdPool; cba.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY; cba.commandBufferCount = 1;
    VkCommandBuffer cb; CHECK_VK(vkAllocateCommandBuffers(dev, &cba, &cb));
    VkCommandBufferBeginInfo cbb{}; cbb.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    CHECK_VK(vkBeginCommandBuffer(cb, &cbb));
    vkCmdBindPipeline(cb, VK_PIPELINE_BIND_POINT_COMPUTE, p);
    vkCmdBindDescriptorSets(cb, VK_PIPELINE_BIND_POINT_COMPUTE, pl, 0, 1, &ds, 0, nullptr);
    vkCmdDispatch(cb, 1, 1, 1);
    CHECK_VK(vkEndCommandBuffer(cb));
    VkSubmitInfo si{}; si.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO; si.commandBufferCount = 1; si.pCommandBuffers = &cb;
    VkFenceCreateInfo fc{}; fc.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
    VkFence f; CHECK_VK(vkCreateFence(dev, &fc, nullptr, &f));
    std::printf("[dbg] dispatch 1 work item...\n");
    CHECK_VK(vkQueueSubmit(queue, 1, &si, f));
    auto t2 = std::chrono::steady_clock::now();
    VkResult wr2 = vkWaitForFences(dev, 1, &f, VK_TRUE, 60ULL * 1000000000ULL);  // 60s 超时
    auto t3 = std::chrono::steady_clock::now();
    std::printf("[result] kernel wait: rc=%d elapsed=%.3fs\n", (int)wr2, std::chrono::duration<double>(t3 - t2).count());
    return 0;
}
