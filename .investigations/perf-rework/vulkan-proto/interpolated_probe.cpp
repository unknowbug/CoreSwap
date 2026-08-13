// interpolated_probe.cpp —— InterpolatedNoiseSampler GPU fp64 vs CPU double 误差验证
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
    std::streamsize n = f.tellg(); f.seekg(0);
    std::vector<uint32_t> code((size_t)n / 4); f.read((char*)code.data(), n);
    return code;
}

// ---- CPU double 参照（对齐 vanilla 1.20.1 InterpolatedNoiseSampler）----
static const int GRADIENTS[16][3] = {
    {1,1,0},{-1,1,0},{1,-1,0},{-1,-1,0},
    {1,0,1},{-1,0,1},{1,0,-1},{-1,0,-1},
    {0,1,1},{0,-1,1},{0,1,-1},{0,-1,-1},
    {1,1,0},{0,-1,1},{-1,1,0},{0,-1,-1},
};
static double perlinFade(double v) { return v*v*v*(v*(v*6.0-15.0)+10.0); }
static double lerpD(double d, double s, double e) { return s + d*(e-s); }
static double maintainPrecision(double v) { return v - std::floor(v / 3.3554432E7 + 0.5) * 3.3554432E7; }

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
    double sample5(double x,double y,double z,double yScale,double yMax) const {
        double d=x+ox, e=y+oy, f=z+oz;
        int i=(int)std::floor(d), j=(int)std::floor(e), k=(int)std::floor(f);
        double g=d-i, h=e-j, l=f-k;
        double n;
        if (yScale != 0.0) {
            double m = (yMax >= 0.0 && yMax < h) ? yMax : h;
            n = std::floor(m / yScale + 1.0E-7F) * yScale;
        } else n = 0.0;
        return section(i, j, k, g, h - n, l, h);
    }
};

static double cpuInterpolated(const CpuPerlin* lower, const CpuPerlin* upper, const CpuPerlin* interp,
                              double x, double y, double z,
                              double scaledXzScale, double scaledYScale, double xzFactor, double yFactor, double smear) {
    double d = x * scaledXzScale;
    double e = y * scaledYScale;
    double f = z * scaledXzScale;
    double g = d / xzFactor;
    double h = e / yFactor;
    double i = f / xzFactor;
    double j = scaledYScale * smear;
    double k = j / yFactor;
    double n = 0.0;
    double o = 1.0;
    for (int p = 0; p < 8; p++) {
        n += interp[p].sample5(maintainPrecision(g*o), maintainPrecision(h*o), maintainPrecision(i*o), k*o, h*o) / o;
        o /= 2.0;
    }
    double q = (n / 10.0 + 1.0) / 2.0;
    bool bl = q >= 1.0, bl2 = q <= 0.0;
    double l = 0.0, m = 0.0;
    o = 1.0;
    for (int r = 0; r < 16; r++) {
        double s = maintainPrecision(d*o), t = maintainPrecision(e*o), u = maintainPrecision(f*o);
        double v = j*o;
        if (!bl) l += lower[r].sample5(s, t, u, v, e*o) / o;
        if (!bl2) m += upper[r].sample5(s, t, u, v, e*o) / o;
        o /= 2.0;
    }
    double qq = std::min(1.0, std::max(0.0, q));
    return (l / 512.0 + qq * (m / 512.0 - l / 512.0)) / 128.0;
}

int main() {
    // 参数（base_3d_noise 默认）
    const double scaledXzScale = 684.412 * 0.25;   // 171.103
    const double scaledYScale = 684.412 * 0.125;   // 85.5515
    const double xzFactor = 80.0, yFactor = 160.0, smear = 8.0;

    // 40 octave（identity perm + origin 0，简化；CPU/GPU 一致即可）
    CpuPerlin lower[16], upper[16], interp[8];
    for (int r = 0; r < 16; r++) {
        for (int i = 0; i < 256; i++) { lower[r].perm[i] = (uint8_t)i; upper[r].perm[i] = (uint8_t)i; }
        lower[r].ox = lower[r].oy = lower[r].oz = 0.0;
        upper[r].ox = upper[r].oy = upper[r].oz = 0.0;
    }
    for (int p = 0; p < 8; p++) {
        for (int i = 0; i < 256; i++) interp[p].perm[i] = (uint8_t)i;
        interp[p].ox = interp[p].oy = interp[p].oz = 0.0;
    }

    // 坐标（远坐标 + y 小数，触发 smear）
    const uint32_t N = 1024;
    std::vector<double> coords(3 * N);
    for (uint32_t i = 0; i < N; i++) {
        coords[3*i+0] = 30000000.0 + (i % 16) * 0.13;
        coords[3*i+1] = 64.0 + (i % 16) * 0.37;
        coords[3*i+2] = 30000000.0 + (i % 16) * 0.11;
    }

    // ---- Vulkan 初始化（启用 fp64）----
    VkApplicationInfo appInfo{}; appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO; appInfo.apiVersion = VK_API_VERSION_1_3;
    VkInstanceCreateInfo instCI{}; instCI.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO; instCI.pApplicationInfo = &appInfo;
    VkInstance instance; CHECK_VK(vkCreateInstance(&instCI, nullptr, &instance));
    uint32_t devCount = 0; vkEnumeratePhysicalDevices(instance, &devCount, nullptr);
    std::vector<VkPhysicalDevice> phys(devCount); vkEnumeratePhysicalDevices(instance, &devCount, phys.data());
    VkPhysicalDevice physDev = phys[0];
    VkPhysicalDeviceProperties devProps; vkGetPhysicalDeviceProperties(physDev, &devProps);
    std::printf("[device] %s\n", devProps.deviceName);
    // 检查 fp64 feature（Vulkan 1.1 core）
    VkPhysicalDeviceFeatures feat{};
    vkGetPhysicalDeviceFeatures(physDev, &feat);
    std::printf("[fp64] %s\n", feat.shaderFloat64 ? "supported" : "NOT supported");
    if (!feat.shaderFloat64) { std::fprintf(stderr, "fp64 not supported, abort\n"); return 1; }

    uint32_t qCount = 0; vkGetPhysicalDeviceQueueFamilyProperties(physDev, &qCount, nullptr);
    std::vector<VkQueueFamilyProperties> qProps(qCount); vkGetPhysicalDeviceQueueFamilyProperties(physDev, &qCount, qProps.data());
    uint32_t computeFamily = UINT32_MAX;
    for (uint32_t i = 0; i < qCount; i++) if (qProps[i].queueFlags & VK_QUEUE_COMPUTE_BIT) { computeFamily = i; break; }
    float qPri = 1.0f;
    VkDeviceQueueCreateInfo qCI{}; qCI.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO; qCI.queueFamilyIndex = computeFamily; qCI.queueCount = 1; qCI.pQueuePriorities = &qPri;
    VkPhysicalDeviceFeatures feat2{};
    feat2.shaderFloat64 = VK_TRUE;
    VkDeviceCreateInfo devCI{}; devCI.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO; devCI.queueCreateInfoCount = 1; devCI.pQueueCreateInfos = &qCI;
    devCI.pEnabledFeatures = &feat2;
    VkDevice device; CHECK_VK(vkCreateDevice(physDev, &devCI, nullptr, &device));
    VkQueue queue; vkGetDeviceQueue(device, computeFamily, 0, &queue);

    auto spv = loadSpv("interpolated.spv");
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

    // params buffer 布局：perm[40][256] + origin[40][3] + 5 double
    VkDeviceSize paramsSize = 40*256*4 + 40*3*8 + 5*8;
    VkDeviceSize coordSize = coords.size() * sizeof(double);
    VkDeviceSize outSize = N * sizeof(double);
    auto makeBuffer = [&](VkDeviceSize size, VkBuffer* buf, VkDeviceMemory* mem) {
        VkBufferCreateInfo bCI{}; bCI.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO; bCI.size = size; bCI.usage = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT; bCI.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
        CHECK_VK(vkCreateBuffer(device, &bCI, nullptr, buf));
        VkMemoryRequirements req; vkGetBufferMemoryRequirements(device, *buf, &req);
        VkPhysicalDeviceMemoryProperties mp; vkGetPhysicalDeviceMemoryProperties(physDev, &mp);
        uint32_t ti = UINT32_MAX; for (uint32_t i = 0; i < mp.memoryTypeCount; i++) if ((req.memoryTypeBits & (1u<<i)) && (mp.memoryTypes[i].propertyFlags & (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT|VK_MEMORY_PROPERTY_HOST_COHERENT_BIT))) { ti = i; break; }
        VkMemoryAllocateInfo aI{}; aI.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO; aI.allocationSize = req.size; aI.memoryTypeIndex = ti;
        CHECK_VK(vkAllocateMemory(device, &aI, nullptr, mem)); CHECK_VK(vkBindBufferMemory(device, *buf, *mem, 0));
    };
    VkBuffer paramsBuf, coordBuf, outBuf; VkDeviceMemory paramsMem, coordMem, outMem;
    makeBuffer(paramsSize, &paramsBuf, &paramsMem); makeBuffer(coordSize, &coordBuf, &coordMem); makeBuffer(outSize, &outBuf, &outMem);

    {
        void* m; CHECK_VK(vkMapMemory(device, paramsMem, 0, paramsSize, 0, &m));
        uint8_t* base = (uint8_t*)m;
        uint32_t* permPtr = (uint32_t*)base;
        for (int o = 0; o < 40; o++) for (int i = 0; i < 256; i++) permPtr[o*256 + i] = (uint32_t)i;  // identity
        double* originPtr = (double*)(base + 40*256*4);
        for (int o = 0; o < 40; o++) { originPtr[o*3+0] = 0.0; originPtr[o*3+1] = 0.0; originPtr[o*3+2] = 0.0; }
        double* scalarPtr = (double*)(base + 40*256*4 + 40*3*8);
        scalarPtr[0] = scaledXzScale; scalarPtr[1] = scaledYScale; scalarPtr[2] = xzFactor; scalarPtr[3] = yFactor; scalarPtr[4] = smear;
        vkUnmapMemory(device, paramsMem);
    }
    { void* m; CHECK_VK(vkMapMemory(device, coordMem, 0, coordSize, 0, &m)); std::memcpy(m, coords.data(), coordSize); vkUnmapMemory(device, coordMem); }

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

    { void* m; CHECK_VK(vkMapMemory(device, outMem, 0, outSize, 0, &m)); std::vector<double> out(N); std::memcpy(out.data(), m, outSize); vkUnmapMemory(device, outMem);
        double maxDiff = 0.0, sumDiff = 0.0; uint32_t maxIdx = 0;
        for (uint32_t i = 0; i < N; i++) {
            double ref = cpuInterpolated(lower, upper, interp, coords[3*i+0], coords[3*i+1], coords[3*i+2],
                                         scaledXzScale, scaledYScale, xzFactor, yFactor, smear);
            double diff = std::fabs(out[i] - ref);
            if (diff > maxDiff) { maxDiff = diff; maxIdx = i; }
            sumDiff += diff;
        }
        std::printf("[result] N=%u, InterpolatedNoiseSampler GPU fp64 vs CPU double: maxDiff=%.3e avgDiff=%.3e\n", N, maxDiff, sumDiff / N);
        std::printf("[result] maxDiff @ i=%u: coord=(%.2f,%.2f,%.2f) gpu=%.15f cpu=%.15f\n",
                    maxIdx, coords[3*maxIdx], coords[3*maxIdx+1], coords[3*maxIdx+2],
                    out[maxIdx], cpuInterpolated(lower, upper, interp, coords[3*maxIdx], coords[3*maxIdx+1], coords[3*maxIdx+2],
                                                 scaledXzScale, scaledYScale, xzFactor, yFactor, smear));
    }

    vkDestroyFence(device, fence, nullptr); vkDestroyCommandPool(device, cmdPool, nullptr); vkDestroyDescriptorPool(device, dpool, nullptr);
    vkFreeMemory(device, paramsMem, nullptr); vkFreeMemory(device, coordMem, nullptr); vkFreeMemory(device, outMem, nullptr);
    vkDestroyBuffer(device, paramsBuf, nullptr); vkDestroyBuffer(device, coordBuf, nullptr); vkDestroyBuffer(device, outBuf, nullptr);
    vkDestroyPipeline(device, pipeline, nullptr); vkDestroyPipelineLayout(device, pipelineLayout, nullptr); vkDestroyDescriptorSetLayout(device, dsl, nullptr); vkDestroyShaderModule(device, shader, nullptr);
    vkDestroyDevice(device, nullptr); vkDestroyInstance(instance, nullptr);
    std::printf("[done]\n");
    return 0;
}
