// vulkan_runtime.h —— Vulkan compute 运行时封装（I1：从 e2e 抽取，供 e2e/worldgen 集成共用）
// 语义与 dfc_final_backend_e2e.cpp 原内联实现逐位一致（12 binding storage buffer layout、
// host-visible+coherent memory、单 command buffer + fence）。
#pragma once
#include <vulkan/vulkan.h>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#define VKRT_CHECK(fn) do { VkResult _r = (fn); if (_r != VK_SUCCESS) { \
    std::fprintf(stderr, "VK error %d at %s:%d (%s)\n", _r, __FILE__, __LINE__, #fn); \
    std::exit(1); } } while (0)

class VkRuntime {
public:
    struct Buffer {
        VkBuffer buffer = VK_NULL_HANDLE;
        VkDeviceMemory memory = VK_NULL_HANDLE;
        VkDeviceSize size = 0;
    };

    VkRuntime() = default;
    ~VkRuntime() { destroy(); }

    // 设备初始化（instance → physical device[0] → compute queue family → device）
    void init() {
        VkApplicationInfo ai{}; ai.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO; ai.apiVersion = VK_API_VERSION_1_3;
        VkInstanceCreateInfo ii{}; ii.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO; ii.pApplicationInfo = &ai;
        VKRT_CHECK(vkCreateInstance(&ii, nullptr, &m_instance));
        uint32_t dc = 0; vkEnumeratePhysicalDevices(m_instance, &dc, nullptr);
        std::vector<VkPhysicalDevice> phys(dc); vkEnumeratePhysicalDevices(m_instance, &dc, phys.data());
        m_physDev = phys[0];
        VkPhysicalDeviceProperties dp; vkGetPhysicalDeviceProperties(m_physDev, &dp);
        std::printf("[device] %s\n", dp.deviceName);
        uint32_t qc = 0; vkGetPhysicalDeviceQueueFamilyProperties(m_physDev, &qc, nullptr);
        std::vector<VkQueueFamilyProperties> qp(qc); vkGetPhysicalDeviceQueueFamilyProperties(m_physDev, &qc, qp.data());
        m_computeFamily = UINT32_MAX;
        for (uint32_t i = 0; i < qc; i++) if (qp[i].queueFlags & VK_QUEUE_COMPUTE_BIT) { m_computeFamily = i; break; }
        float pri = 1.0f;
        VkDeviceQueueCreateInfo q{}; q.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO; q.queueFamilyIndex = m_computeFamily; q.queueCount = 1; q.pQueuePriorities = &pri;
        VkDeviceCreateInfo di{}; di.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO; di.queueCreateInfoCount = 1; di.pQueueCreateInfos = &q;
        VKRT_CHECK(vkCreateDevice(m_physDev, &di, nullptr, &m_device));
        vkGetDeviceQueue(m_device, m_computeFamily, 0, &m_queue);
        // descriptor set layout：12 个 storage buffer binding（0-11；binding 2 已删 OriginBuf，
        // 但 layout 保留 12 项与 shader 声明兼容——shader 未用的 binding 2 合法）
        VkDescriptorSetLayoutBinding b[12]{};
        for (int i = 0; i < 12; i++) { b[i].binding = i; b[i].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; b[i].descriptorCount = 1; b[i].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT; }
        VkDescriptorSetLayoutCreateInfo lc{}; lc.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO; lc.bindingCount = 12; lc.pBindings = b;
        VKRT_CHECK(vkCreateDescriptorSetLayout(m_device, &lc, nullptr, &m_dsl));
        VkPipelineLayoutCreateInfo plc{}; plc.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO; plc.setLayoutCount = 1; plc.pSetLayouts = &m_dsl;
        VKRT_CHECK(vkCreatePipelineLayout(m_device, &plc, nullptr, &m_pipelineLayout));
    }

    // 从 spv 文件创建 compute pipeline（耗时 = 驱动编译，这是 67-102s 的来源）
    void createPipeline(const std::string& spvPath) {
        std::vector<uint32_t> code = loadSpv(spvPath);
        VkShaderModuleCreateInfo sc{}; sc.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO; sc.codeSize = code.size() * 4; sc.pCode = code.data();
        VkShaderModule sm; VKRT_CHECK(vkCreateShaderModule(m_device, &sc, nullptr, &sm));
        VkComputePipelineCreateInfo cp{}; cp.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
        cp.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO; cp.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT; cp.stage.module = sm; cp.stage.pName = "main"; cp.layout = m_pipelineLayout;
        VKRT_CHECK(vkCreateComputePipelines(m_device, VK_NULL_HANDLE, 1, &cp, nullptr, &m_pipeline));
        vkDestroyShaderModule(m_device, sm, nullptr);
    }

    // X2（260903-05）多 pipeline 扩展：channels 每 channel 一个独立 spv/pipeline（共享 dsl/layout/buffers）。
    VkPipeline createPipelineEx(const std::string& spvPath) {
        std::vector<uint32_t> code = loadSpv(spvPath);
        VkShaderModuleCreateInfo sc{}; sc.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO; sc.codeSize = code.size() * 4; sc.pCode = code.data();
        VkShaderModule sm; VKRT_CHECK(vkCreateShaderModule(m_device, &sc, nullptr, &sm));
        VkComputePipelineCreateInfo cp{}; cp.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
        cp.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO; cp.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT; cp.stage.module = sm; cp.stage.pName = "main"; cp.layout = m_pipelineLayout;
        VkPipeline p; VKRT_CHECK(vkCreateComputePipelines(m_device, VK_NULL_HANDLE, 1, &cp, nullptr, &p));
        vkDestroyShaderModule(m_device, sm, nullptr);
        return p;
    }
    // dispatch 指定 pipeline（其余与 dispatch 同：单 command buffer + fence 同步）
    void dispatchPipeline(VkPipeline pipeline, const VkDescriptorSet& ds, uint32_t n) {
        VkCommandPoolCreateInfo cp{}; cp.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO; cp.queueFamilyIndex = m_computeFamily;
        VkCommandPool pool; VKRT_CHECK(vkCreateCommandPool(m_device, &cp, nullptr, &pool));
        VkCommandBufferAllocateInfo ca{}; ca.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO; ca.commandPool = pool; ca.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY; ca.commandBufferCount = 1;
        VkCommandBuffer cb; VKRT_CHECK(vkAllocateCommandBuffers(m_device, &ca, &cb));
        VkCommandBufferBeginInfo begin{}; begin.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
        VKRT_CHECK(vkBeginCommandBuffer(cb, &begin));
        vkCmdBindPipeline(cb, VK_PIPELINE_BIND_POINT_COMPUTE, pipeline);
        vkCmdBindDescriptorSets(cb, VK_PIPELINE_BIND_POINT_COMPUTE, m_pipelineLayout, 0, 1, &ds, 0, nullptr);
        vkCmdDispatch(cb, (n + 255) / 256, 1, 1);
        VKRT_CHECK(vkEndCommandBuffer(cb));
        VkSubmitInfo si{}; si.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO; si.commandBufferCount = 1; si.pCommandBuffers = &cb;
        VkFenceCreateInfo fc{}; fc.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
        VkFence fence; VKRT_CHECK(vkCreateFence(m_device, &fc, nullptr, &fence));
        VKRT_CHECK(vkQueueSubmit(m_queue, 1, &si, fence));
        VKRT_CHECK(vkWaitForFences(m_device, 1, &fence, VK_TRUE, UINT64_MAX));
        vkDestroyFence(m_device, fence, nullptr); vkDestroyCommandPool(m_device, pool, nullptr);
    }

    // 创建 host-visible + coherent storage buffer
    Buffer createBuffer(VkDeviceSize size) {
        Buffer buf; buf.size = size;
        VkBufferCreateInfo bci{}; bci.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO; bci.size = size; bci.usage = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT; bci.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
        VKRT_CHECK(vkCreateBuffer(m_device, &bci, nullptr, &buf.buffer));
        VkMemoryRequirements req; vkGetBufferMemoryRequirements(m_device, buf.buffer, &req);
        VkPhysicalDeviceMemoryProperties mp; vkGetPhysicalDeviceMemoryProperties(m_physDev, &mp);
        uint32_t ti = UINT32_MAX; for (uint32_t i = 0; i < mp.memoryTypeCount; i++) if ((req.memoryTypeBits & (1u << i)) && (mp.memoryTypes[i].propertyFlags & (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT))) { ti = i; break; }
        VkMemoryAllocateInfo aI{}; aI.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO; aI.allocationSize = req.size; aI.memoryTypeIndex = ti;
        VKRT_CHECK(vkAllocateMemory(m_device, &aI, nullptr, &buf.memory)); VKRT_CHECK(vkBindBufferMemory(m_device, buf.buffer, buf.memory, 0));
        return buf;
    }

    void upload(const Buffer& buf, const void* data, VkDeviceSize size) {
        void* m; VKRT_CHECK(vkMapMemory(m_device, buf.memory, 0, size, 0, &m)); std::memcpy(m, data, size); vkUnmapMemory(m_device, buf.memory);
    }

    template <size_t N>
    VkDescriptorSet makeDescriptorSet(const Buffer* bufs, const int* bindings, const VkDeviceSize* sizes, int count) {
        VkDescriptorPoolSize ps{}; ps.type = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; ps.descriptorCount = (uint32_t)count;
        VkDescriptorPoolCreateInfo dp{}; dp.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO; dp.maxSets = 1; dp.poolSizeCount = 1; dp.pPoolSizes = &ps;
        VkDescriptorPool pool; VKRT_CHECK(vkCreateDescriptorPool(m_device, &dp, nullptr, &pool));
        VkDescriptorSetAllocateInfo da{}; da.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO; da.descriptorPool = pool; da.descriptorSetCount = 1; da.pSetLayouts = &m_dsl;
        VkDescriptorSet ds; VKRT_CHECK(vkAllocateDescriptorSets(m_device, &da, &ds));
        std::vector<VkDescriptorBufferInfo> dbis(count);
        std::vector<VkWriteDescriptorSet> writes(count);
        for (int i = 0; i < count; i++) {
            dbis[i] = { bufs[i].buffer, 0, sizes[i] };
            writes[i].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET; writes[i].dstSet = ds; writes[i].dstBinding = bindings[i];
            writes[i].descriptorCount = 1; writes[i].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; writes[i].pBufferInfo = &dbis[i];
        }
        vkUpdateDescriptorSets(m_device, (uint32_t)count, writes.data(), 0, nullptr);
        m_pools.push_back(pool);
        return ds;
    }

    // dispatch N 个 work item（256/组），等待完成
    void dispatch(const VkDescriptorSet& ds, uint32_t n) {
        VkCommandPoolCreateInfo cp{}; cp.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO; cp.queueFamilyIndex = m_computeFamily;
        VkCommandPool pool; VKRT_CHECK(vkCreateCommandPool(m_device, &cp, nullptr, &pool));
        VkCommandBufferAllocateInfo ca{}; ca.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO; ca.commandPool = pool; ca.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY; ca.commandBufferCount = 1;
        VkCommandBuffer cb; VKRT_CHECK(vkAllocateCommandBuffers(m_device, &ca, &cb));
        VkCommandBufferBeginInfo begin{}; begin.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
        VKRT_CHECK(vkBeginCommandBuffer(cb, &begin));
        vkCmdBindPipeline(cb, VK_PIPELINE_BIND_POINT_COMPUTE, m_pipeline);
        vkCmdBindDescriptorSets(cb, VK_PIPELINE_BIND_POINT_COMPUTE, m_pipelineLayout, 0, 1, &ds, 0, nullptr);
        vkCmdDispatch(cb, (n + 255) / 256, 1, 1);
        VKRT_CHECK(vkEndCommandBuffer(cb));
        VkSubmitInfo si{}; si.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO; si.commandBufferCount = 1; si.pCommandBuffers = &cb;
        VkFenceCreateInfo fc{}; fc.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
        VkFence fence; VKRT_CHECK(vkCreateFence(m_device, &fc, nullptr, &fence));
        VKRT_CHECK(vkQueueSubmit(m_queue, 1, &si, fence));
        VKRT_CHECK(vkWaitForFences(m_device, 1, &fence, VK_TRUE, UINT64_MAX));
        vkDestroyFence(m_device, fence, nullptr); vkDestroyCommandPool(m_device, pool, nullptr);
    }

    void readback(const Buffer& buf, void* out, VkDeviceSize size) {
        void* m; VKRT_CHECK(vkMapMemory(m_device, buf.memory, 0, size, 0, &m)); std::memcpy(out, m, size); vkUnmapMemory(m_device, buf.memory);
    }

    void destroy() {
        if (m_device) {
            for (auto p : m_pools) vkDestroyDescriptorPool(m_device, p, nullptr);
            if (m_pipeline) vkDestroyPipeline(m_device, m_pipeline, nullptr);
            for (auto p : m_extraPipelines) if (p) vkDestroyPipeline(m_device, p, nullptr);
            vkDestroyPipelineLayout(m_device, m_pipelineLayout, nullptr);
            vkDestroyDescriptorSetLayout(m_device, m_dsl, nullptr);
            vkDestroyDevice(m_device, nullptr);
            m_device = VK_NULL_HANDLE; m_pipeline = VK_NULL_HANDLE;
        }
        if (m_instance) { vkDestroyInstance(m_instance, nullptr); m_instance = VK_NULL_HANDLE; }
    }

    // 显式释放单个 buffer（调用方负责在 rt.destroy() 前释放全部 buffer）
    static void destroyBuffer(VkDevice dev, const Buffer& buf) {
        if (dev && buf.buffer) { vkDestroyBuffer(dev, buf.buffer, nullptr); vkFreeMemory(dev, buf.memory, nullptr); }
    }

    VkDevice device() const { return m_device; }

private:
    static std::vector<uint32_t> loadSpv(const std::string& path) {
        std::FILE* f = std::fopen(path.c_str(), "rb");
        if (!f) { std::fprintf(stderr, "cannot open %s\n", path.c_str()); std::exit(1); }
        std::fseek(f, 0, SEEK_END); long n = std::ftell(f); std::fseek(f, 0, SEEK_SET);
        std::vector<uint32_t> code((size_t)n / 4); std::fread(code.data(), 1, (size_t)n, f); std::fclose(f);
        return code;
    }

    VkInstance m_instance = VK_NULL_HANDLE;
    VkPhysicalDevice m_physDev = VK_NULL_HANDLE;
    VkDevice m_device = VK_NULL_HANDLE;
    VkQueue m_queue = VK_NULL_HANDLE;
    uint32_t m_computeFamily = UINT32_MAX;
    VkDescriptorSetLayout m_dsl = VK_NULL_HANDLE;
    VkPipelineLayout m_pipelineLayout = VK_NULL_HANDLE;
    VkPipeline m_pipeline = VK_NULL_HANDLE;
    std::vector<VkPipeline> m_extraPipelines;   // X2：createPipelineEx 产物（destroy 统一清理）
    std::vector<VkDescriptorPool> m_pools;
};
