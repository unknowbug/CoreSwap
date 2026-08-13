// perlin_probe.cpp —— Perlin 噪声 GPU FP32 vs CPU double 精度验证
// 上传 perm+origin → GPU 算 float 噪声 → CPU 用 double 算参照 → 对比差异
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

// ---- CPU double 版 Perlin 噪声（对齐 vanilla 1.20.1）----
static const int GRADIENTS[16][3] = {
    {1,1,0},{-1,1,0},{1,-1,0},{-1,-1,0},
    {1,0,1},{-1,0,1},{1,0,-1},{-1,0,-1},
    {0,1,1},{0,-1,1},{0,1,-1},{0,-1,-1},
    {1,1,0},{0,-1,1},{-1,1,0},{0,-1,-1},
};
static double perlinFade(double v) { return v*v*v*(v*(v*6.0-15.0)+10.0); }
static double lerpD(double d, double s, double e) { return s + d*(e-s); }

struct CpuPerlin {
    uint8_t perm[256];
    double ox, oy, oz;
    int map(int i) const { return perm[i & 0xFF]; }
    double grad(int hash, double x, double y, double z) const {
        const int* g = GRADIENTS[hash & 15];
        return g[0]*x + g[1]*y + g[2]*z;
    }
    double section(int sx,int sy,int sz,double lx,double ly,double lz,double fadeY) const {
        int i=map(sx), j=map(sx+1), k=map(i+sy), l=map(i+sy+1), m=map(j+sy), n=map(j+sy+1);
        double d=grad(map(k+sz),lx,ly,lz), e=grad(map(m+sz),lx-1,ly,lz);
        double f=grad(map(l+sz),lx,ly-1,lz), g=grad(map(n+sz),lx-1,ly-1,lz);
        double h=grad(map(k+sz+1),lx,ly,lz-1), o=grad(map(m+sz+1),lx-1,ly,lz-1);
        double p=grad(map(l+sz+1),lx,ly-1,lz-1), q=grad(map(n+sz+1),lx-1,ly-1,lz-1);
        double r=perlinFade(lx), s=perlinFade(fadeY), t=perlinFade(lz);
        double x0=lerpD(r,d,e), x1=lerpD(r,f,g), x2=lerpD(r,h,o), x3=lerpD(r,p,q);
        double y0=lerpD(s,x0,x1), y1=lerpD(s,x2,x3);
        return lerpD(t,y0,y1);
    }
    double sample(double x,double y,double z) const {
        double d=x+ox, e=y+oy, f=z+oz;
        int i=(int)std::floor(d), j=(int)std::floor(e), k=(int)std::floor(f);
        double g=d-i, h=e-j, l=f-k;
        return section(i,j,k,g,h,l,h);
    }
};

int main() {
    // 噪声参数：identity perm + origin(0,0,0)（最简，精度验证足够）
    CpuPerlin cpu;
    for (int i = 0; i < 256; i++) cpu.perm[i] = (uint8_t)i;
    cpu.ox = cpu.oy = cpu.oz = 0.0;

    // 坐标：0.1 步长的小坐标（非精确表示，能体现 float/double 差异）
    const uint32_t N = 4096;
    std::vector<float> coords(3*N);
    for (uint32_t i = 0; i < N; i++) {
        coords[3*i+0] = (float)((int)(i % 16)) * 0.1f;
        coords[3*i+1] = (float)((int)(i / 16 % 16)) * 0.1f;
        coords[3*i+2] = (float)((int)(i / 256)) * 0.1f;
    }

    // ---- Vulkan 初始化（同 vulkan_min.cpp）----
    VkApplicationInfo appInfo{};
    appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
    appInfo.apiVersion = VK_API_VERSION_1_3;
    VkInstanceCreateInfo instCI{};
    instCI.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
    instCI.pApplicationInfo = &appInfo;
    VkInstance instance;
    CHECK_VK(vkCreateInstance(&instCI, nullptr, &instance));

    uint32_t devCount = 0;
    vkEnumeratePhysicalDevices(instance, &devCount, nullptr);
    std::vector<VkPhysicalDevice> phys(devCount);
    vkEnumeratePhysicalDevices(instance, &devCount, phys.data());
    VkPhysicalDevice physDev = phys[0];
    VkPhysicalDeviceProperties devProps;
    vkGetPhysicalDeviceProperties(physDev, &devProps);
    std::printf("[device] %s\n", devProps.deviceName);

    uint32_t qCount = 0;
    vkGetPhysicalDeviceQueueFamilyProperties(physDev, &qCount, nullptr);
    std::vector<VkQueueFamilyProperties> qProps(qCount);
    vkGetPhysicalDeviceQueueFamilyProperties(physDev, &qCount, qProps.data());
    uint32_t computeFamily = UINT32_MAX;
    for (uint32_t i = 0; i < qCount; i++) if (qProps[i].queueFlags & VK_QUEUE_COMPUTE_BIT) { computeFamily = i; break; }

    float qPri = 1.0f;
    VkDeviceQueueCreateInfo qCI{};
    qCI.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
    qCI.queueFamilyIndex = computeFamily; qCI.queueCount = 1; qCI.pQueuePriorities = &qPri;
    VkDeviceCreateInfo devCI{};
    devCI.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
    devCI.queueCreateInfoCount = 1; devCI.pQueueCreateInfos = &qCI;
    VkDevice device;
    CHECK_VK(vkCreateDevice(physDev, &devCI, nullptr, &device));
    VkQueue queue;
    vkGetDeviceQueue(device, computeFamily, 0, &queue);

    auto spv = loadSpv("perlin.spv");
    VkShaderModuleCreateInfo smCI{};
    smCI.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
    smCI.codeSize = spv.size() * 4; smCI.pCode = spv.data();
    VkShaderModule shader;
    CHECK_VK(vkCreateShaderModule(device, &smCI, nullptr, &shader));

    VkDescriptorSetLayoutBinding bindings[3]{};
    for (int b = 0; b < 3; b++) {
        bindings[b].binding = b;
        bindings[b].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
        bindings[b].descriptorCount = 1;
        bindings[b].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
    }
    VkDescriptorSetLayoutCreateInfo dslCI{};
    dslCI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
    dslCI.bindingCount = 3; dslCI.pBindings = bindings;
    VkDescriptorSetLayout dsl;
    CHECK_VK(vkCreateDescriptorSetLayout(device, &dslCI, nullptr, &dsl));
    VkPipelineLayoutCreateInfo plCI{};
    plCI.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
    plCI.setLayoutCount = 1; plCI.pSetLayouts = &dsl;
    VkPipelineLayout pipelineLayout;
    CHECK_VK(vkCreatePipelineLayout(device, &plCI, nullptr, &pipelineLayout));
    VkComputePipelineCreateInfo cpCI{};
    cpCI.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    cpCI.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    cpCI.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT;
    cpCI.stage.module = shader; cpCI.stage.pName = "main";
    cpCI.layout = pipelineLayout;
    VkPipeline pipeline;
    CHECK_VK(vkCreateComputePipelines(device, VK_NULL_HANDLE, 1, &cpCI, nullptr, &pipeline));

    // buffers
    VkDeviceSize paramsSize = 256*4 + 3*4;   // perm[256] + origin 3 float
    VkDeviceSize coordsSize = coords.size() * sizeof(float);
    VkDeviceSize outSize = N * sizeof(float);
    auto makeBuffer = [&](VkDeviceSize size, VkBuffer* buf, VkDeviceMemory* mem) {
        VkBufferCreateInfo bCI{};
        bCI.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO;
        bCI.size = size; bCI.usage = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT; bCI.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
        CHECK_VK(vkCreateBuffer(device, &bCI, nullptr, buf));
        VkMemoryRequirements req; vkGetBufferMemoryRequirements(device, *buf, &req);
        VkPhysicalDeviceMemoryProperties mp; vkGetPhysicalDeviceMemoryProperties(physDev, &mp);
        uint32_t ti = UINT32_MAX;
        for (uint32_t i = 0; i < mp.memoryTypeCount; i++)
            if ((req.memoryTypeBits & (1u<<i)) && (mp.memoryTypes[i].propertyFlags & (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT|VK_MEMORY_PROPERTY_HOST_COHERENT_BIT))) { ti = i; break; }
        VkMemoryAllocateInfo aI{}; aI.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO; aI.allocationSize = req.size; aI.memoryTypeIndex = ti;
        CHECK_VK(vkAllocateMemory(device, &aI, nullptr, mem));
        CHECK_VK(vkBindBufferMemory(device, *buf, *mem, 0));
    };
    VkBuffer paramsBuf, coordsBuf, outBuf;
    VkDeviceMemory paramsMem, coordsMem, outMem;
    makeBuffer(paramsSize, &paramsBuf, &paramsMem);
    makeBuffer(coordsSize, &coordsBuf, &coordsMem);
    makeBuffer(outSize, &outBuf, &outMem);

    // 上传 params（perm[256] uint + origin 3 float）
    {
        void* m;
        CHECK_VK(vkMapMemory(device, paramsMem, 0, paramsSize, 0, &m));
        uint32_t* mp = (uint32_t*)m;
        for (int i = 0; i < 256; i++) mp[i] = cpu.perm[i];
        float* of = (float*)(mp + 256);
        of[0] = 0.0f; of[1] = 0.0f; of[2] = 0.0f;
        vkUnmapMemory(device, paramsMem);
    }
    { void* m; CHECK_VK(vkMapMemory(device, coordsMem, 0, coordsSize, 0, &m)); std::memcpy(m, coords.data(), coordsSize); vkUnmapMemory(device, coordsMem); }

    // descriptor set
    VkDescriptorPoolSize poolSize{}; poolSize.type = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; poolSize.descriptorCount = 3;
    VkDescriptorPoolCreateInfo dpCI{}; dpCI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO; dpCI.maxSets = 1; dpCI.poolSizeCount = 1; dpCI.pPoolSizes = &poolSize;
    VkDescriptorPool dpool; CHECK_VK(vkCreateDescriptorPool(device, &dpCI, nullptr, &dpool));
    VkDescriptorSetAllocateInfo dsAI{}; dsAI.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO; dsAI.descriptorPool = dpool; dsAI.descriptorSetCount = 1; dsAI.pSetLayouts = &dsl;
    VkDescriptorSet ds; CHECK_VK(vkAllocateDescriptorSets(device, &dsAI, &ds));
    VkDescriptorBufferInfo dbi[3]{
        {paramsBuf, 0, paramsSize}, {coordsBuf, 0, coordsSize}, {outBuf, 0, outSize}
    };
    VkWriteDescriptorSet writes[3]{};
    for (int b = 0; b < 3; b++) {
        writes[b].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        writes[b].dstSet = ds; writes[b].dstBinding = b; writes[b].descriptorCount = 1;
        writes[b].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER; writes[b].pBufferInfo = &dbi[b];
    }
    vkUpdateDescriptorSets(device, 3, writes, 0, nullptr);

    // dispatch
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

    // 读回 + 对比
    { void* m; CHECK_VK(vkMapMemory(device, outMem, 0, outSize, 0, &m)); std::vector<float> out(N); std::memcpy(out.data(), m, outSize); vkUnmapMemory(device, outMem);
        double maxDiff = 0.0, sumDiff = 0.0; uint32_t maxIdx = 0;
        for (uint32_t i = 0; i < N; i++) {
            double ref = cpu.sample(coords[3*i+0], coords[3*i+1], coords[3*i+2]);
            double gpu = out[i];
            double diff = std::fabs(gpu - ref);
            if (diff > maxDiff) { maxDiff = diff; maxIdx = i; }
            sumDiff += diff;
        }
        std::printf("[result] N=%u, GPU float vs CPU double: maxDiff=%.3e avgDiff=%.3e\n", N, maxDiff, sumDiff / N);
        std::printf("[result] maxDiff @ i=%u: coord=(%.3f,%.3f,%.3f) gpu=%.9f cpu=%.9f\n",
                    maxIdx, coords[3*maxIdx], coords[3*maxIdx+1], coords[3*maxIdx+2],
                    out[maxIdx], cpu.sample(coords[3*maxIdx], coords[3*maxIdx+1], coords[3*maxIdx+2]));
        // 打印前 5 个对比
        for (uint32_t i = 0; i < 5; i++) {
            double ref = cpu.sample(coords[3*i+0], coords[3*i+1], coords[3*i+2]);
            std::printf("  i=%u coord=(%.1f,%.1f,%.1f) gpu=%.9f cpu=%.9f diff=%.3e\n",
                        i, coords[3*i], coords[3*i+1], coords[3*i+2], out[i], ref, std::fabs(out[i]-ref));
        }
    }

    // cleanup
    vkDestroyFence(device, fence, nullptr); vkDestroyCommandPool(device, cmdPool, nullptr);
    vkDestroyDescriptorPool(device, dpool, nullptr);
    vkFreeMemory(device, paramsMem, nullptr); vkFreeMemory(device, coordsMem, nullptr); vkFreeMemory(device, outMem, nullptr);
    vkDestroyBuffer(device, paramsBuf, nullptr); vkDestroyBuffer(device, coordsBuf, nullptr); vkDestroyBuffer(device, outBuf, nullptr);
    vkDestroyPipeline(device, pipeline, nullptr); vkDestroyPipelineLayout(device, pipelineLayout, nullptr);
    vkDestroyDescriptorSetLayout(device, dsl, nullptr); vkDestroyShaderModule(device, shader, nullptr);
    vkDestroyDevice(device, nullptr); vkDestroyInstance(instance, nullptr);
    std::printf("[done]\n");
    return 0;
}
