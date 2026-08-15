// compile_bench.cpp —— 批量测 vkCreateComputePipelines 编译时间（多 spv）
// 用法: compile_bench.exe a.spv b.spv ...
#include <vulkan/vulkan.h>
#include <cstdio>
#include <cstdlib>
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
int main(int argc, char** argv) {
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
    VkDescriptorSetLayoutBinding b[12]{};
    for (int i = 0; i < 12; i++) { b[i].binding = i; b[i].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; b[i].descriptorCount = 1; b[i].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT; }
    VkDescriptorSetLayoutCreateInfo lc{}; lc.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO; lc.bindingCount = 12; lc.pBindings = b;
    VkDescriptorSetLayout l; CHECK_VK(vkCreateDescriptorSetLayout(dev, &lc, nullptr, &l));
    VkPipelineLayoutCreateInfo plc{}; plc.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO; plc.setLayoutCount = 1; plc.pSetLayouts = &l;
    VkPipelineLayout pl; CHECK_VK(vkCreatePipelineLayout(dev, &plc, nullptr, &pl));
    std::printf("# compile_bench: %d shaders\n", argc - 1);
    for (int a = 1; a < argc; a++) {
        auto spv = loadSpv(argv[a]);
        VkShaderModuleCreateInfo sc{}; sc.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO; sc.codeSize = spv.size()*4; sc.pCode = spv.data();
        VkShaderModule sm; CHECK_VK(vkCreateShaderModule(dev, &sc, nullptr, &sm));
        VkComputePipelineCreateInfo cp{}; cp.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
        cp.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO; cp.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT; cp.stage.module = sm; cp.stage.pName = "main"; cp.layout = pl;
        VkPipeline p;
        auto t0 = std::chrono::steady_clock::now();
        VkResult r = vkCreateComputePipelines(dev, VK_NULL_HANDLE, 1, &cp, nullptr, &p);
        auto t1 = std::chrono::steady_clock::now();
        std::printf("[%s] %.1fs (rc=%d)\n", argv[a], std::chrono::duration<double>(t1 - t0).count(), (int)r);
        if (r == VK_SUCCESS) vkDestroyPipeline(dev, p, nullptr);
        vkDestroyShaderModule(dev, sm, nullptr);
    }
    vkDestroyDevice(dev, nullptr);
    vkDestroyInstance(inst, nullptr);
    return 0;
}
