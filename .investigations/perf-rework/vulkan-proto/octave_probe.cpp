// octave_probe.cpp —— 多 octave + DoublePerlinNoiseSampler 完整链路验证（GPU float vs CPU double）
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

// ---- CPU double 版（对齐 vanilla 1.20.1）----
static const int GRADIENTS[16][3] = {
    {1,1,0},{-1,1,0},{1,-1,0},{-1,-1,0},
    {1,0,1},{-1,0,1},{1,0,-1},{-1,0,-1},
    {0,1,1},{0,-1,1},{0,1,-1},{0,-1,-1},
    {1,1,0},{0,-1,1},{-1,1,0},{0,-1,-1},
};
static double perlinFade(double v) { return v*v*v*(v*(v*6.0-15.0)+10.0); }
static double lerpD(double d, double s, double e) { return s + d*(e-s); }
static double maintainPrecision(double v) { return v - std::floor(v / 3.3554432E7 + 0.5) * 3.3554432E7; }
static const double DOMAIN_SCALE = 1.0181268882175227;

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
    // 单 octave 采样（含 maintainPrecision + origin + floor）
    double sample(double x, double y, double z) const {
        double d = x + ox, e = y + oy, f = z + oz;
        int i = (int)std::floor(d), j = (int)std::floor(e), k = (int)std::floor(f);
        double g = d - i, h = e - j, l = f - k;
        return section(i, j, k, g, h, l, h);
    }
};

// OctavePerlinNoiseSampler（2 octave）
static double octaveSample(const CpuPerlin* pns, double x, double y, double z,
                           double lacunarity, double persistence, const double amplitudes[2]) {
    double d = 0.0;
    double e = lacunarity;
    double f = persistence;
    for (int i = 0; i < 2; i++) {
        double g = pns[i].sample(maintainPrecision(x*e), maintainPrecision(y*e), maintainPrecision(z*e));
        d += amplitudes[i] * g * f;
        e *= 2.0;
        f /= 2.0;
    }
    return d;
}

int main() {
    // 参数：2 octave，amplitudes=[1,1]，firstOctave=0（lacunarity=1, persistence=2/3），amplitude=1.0
    const double amplitudes[2] = {1.0, 1.0};
    const double lacunarity = 1.0;
    const double persistence = 2.0 / 3.0;   // 2^1 / (2^2 - 1)
    const double amplitude = 1.0;

    CpuPerlin pns[4];   // first[2] + second[2]，都 identity + origin 0
    for (int o = 0; o < 4; o++) {
        for (int i = 0; i < 256; i++) pns[o].perm[i] = (uint8_t)i;
        pns[o].ox = pns[o].oy = pns[o].oz = 0.0;
    }

    // 坐标（远坐标 + 小数）
    const double SCALE = 171.103;
    const double BASE = 30000000.0;
    const uint32_t N = 4096;
    const int TOTAL_OCTAVES = 4;

    // CPU double 完整采样 + 拆分坐标
    std::vector<float> splitCoord(TOTAL_OCTAVES * 6 * N);
    for (uint32_t n = 0; n < N; n++) {
        double ox = (n % 64) * 0.13;
        double x = (BASE + ox) * SCALE;
        double y = (BASE + ox * 0.7) * SCALE * 0.7;
        double z = (BASE + ox * 1.3) * SCALE * 1.3;
        // 每 octave 拆分
        for (int o = 0; o < 2; o++) {
            // first octave o: coord × lacunarity × 2^o
            double eo = lacunarity * std::pow(2.0, o);
            double cx = maintainPrecision(x * eo), cy = maintainPrecision(y * eo), cz = maintainPrecision(z * eo);
            double dx = cx + pns[o].ox, dy = cy + pns[o].oy, dz = cz + pns[o].oz;
            int ix = (int)std::floor(dx), iy = (int)std::floor(dy), iz = (int)std::floor(dz);
            splitCoord[n*24 + o*6 + 0] = (float)ix;
            splitCoord[n*24 + o*6 + 1] = (float)iy;
            splitCoord[n*24 + o*6 + 2] = (float)iz;
            splitCoord[n*24 + o*6 + 3] = (float)(dx - ix);
            splitCoord[n*24 + o*6 + 4] = (float)(dy - iy);
            splitCoord[n*24 + o*6 + 5] = (float)(dz - iz);
        }
        for (int o = 0; o < 2; o++) {
            // second octave o: coord × DOMAIN_SCALE × lacunarity × 2^o
            double eo = DOMAIN_SCALE * lacunarity * std::pow(2.0, o);
            double cx = maintainPrecision(x * eo), cy = maintainPrecision(y * eo), cz = maintainPrecision(z * eo);
            double dx = cx + pns[2+o].ox, dy = cy + pns[2+o].oy, dz = cz + pns[2+o].oz;
            int ix = (int)std::floor(dx), iy = (int)std::floor(dy), iz = (int)std::floor(dz);
            splitCoord[n*24 + (2+o)*6 + 0] = (float)ix;
            splitCoord[n*24 + (2+o)*6 + 1] = (float)iy;
            splitCoord[n*24 + (2+o)*6 + 2] = (float)iz;
            splitCoord[n*24 + (2+o)*6 + 3] = (float)(dx - ix);
            splitCoord[n*24 + (2+o)*6 + 4] = (float)(dy - iy);
            splitCoord[n*24 + (2+o)*6 + 5] = (float)(dz - iz);
        }
    }

    // ---- Vulkan 初始化 ----
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

    // params buffer: perm[4][256] + amplitudes[2] + persistence + amplitude + domainScale
    VkDeviceSize paramsSize = 4*256*4 + 2*4 + 4 + 4 + 4;
    VkDeviceSize coordSize = splitCoord.size() * sizeof(float);
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
    VkBuffer paramsBuf, coordBuf, outBuf; VkDeviceMemory paramsMem, coordMem, outMem;
    makeBuffer(paramsSize, &paramsBuf, &paramsMem); makeBuffer(coordSize, &coordBuf, &coordMem); makeBuffer(outSize, &outBuf, &outMem);
    {
        void* m; CHECK_VK(vkMapMemory(device, paramsMem, 0, paramsSize, 0, &m));
        uint32_t* mp = (uint32_t*)m;
        for (int o = 0; o < 4; o++) for (int i = 0; i < 256; i++) mp[o*256 + i] = pns[o].perm[i];
        float* fp = (float*)(mp + 4*256);
        fp[0] = (float)amplitudes[0]; fp[1] = (float)amplitudes[1];
        fp[2] = (float)persistence;
        fp[3] = (float)amplitude;
        fp[4] = (float)DOMAIN_SCALE;
        vkUnmapMemory(device, paramsMem);
    }
    { void* m; CHECK_VK(vkMapMemory(device, coordMem, 0, coordSize, 0, &m)); std::memcpy(m, splitCoord.data(), coordSize); vkUnmapMemory(device, coordMem); }

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

    { void* m; CHECK_VK(vkMapMemory(device, outMem, 0, outSize, 0, &m)); std::vector<float> out(N); std::memcpy(out.data(), m, outSize); vkUnmapMemory(device, outMem);
        double maxDiff = 0.0, sumDiff = 0.0; uint32_t maxIdx = 0;
        for (uint32_t n = 0; n < N; n++) {
            double ox = (n % 64) * 0.13;
            double x = (BASE + ox) * SCALE;
            double y = (BASE + ox * 0.7) * SCALE * 0.7;
            double z = (BASE + ox * 1.3) * SCALE * 1.3;
            // CPU double 完整：first + second(×1.018)
            double first = octaveSample(&pns[0], x, y, z, lacunarity, persistence, amplitudes);
            double second = octaveSample(&pns[2], x*DOMAIN_SCALE, y*DOMAIN_SCALE, z*DOMAIN_SCALE, lacunarity, persistence, amplitudes);
            double ref = (first + second) * amplitude;
            double gpu = out[n];
            double diff = std::fabs(gpu - ref);
            if (diff > maxDiff) { maxDiff = diff; maxIdx = n; }
            sumDiff += diff;
        }
        std::printf("[result] N=%u, 多octave+Double 完整链路: GPU float vs CPU double maxDiff=%.3e avgDiff=%.3e\n", N, maxDiff, sumDiff / N);
        std::printf("[result] maxDiff @ n=%u: gpu=%.9f cpu=%.9f\n", maxIdx, out[maxIdx],
                    (octaveSample(&pns[0], (BASE+(maxIdx%64)*0.13)*SCALE, (BASE+(maxIdx%64)*0.13*0.7)*SCALE*0.7, (BASE+(maxIdx%64)*0.13*1.3)*SCALE*1.3, lacunarity, persistence, amplitudes)
                     + octaveSample(&pns[2], (BASE+(maxIdx%64)*0.13)*SCALE*DOMAIN_SCALE, (BASE+(maxIdx%64)*0.13*0.7)*SCALE*0.7*DOMAIN_SCALE, (BASE+(maxIdx%64)*0.13*1.3)*SCALE*1.3*DOMAIN_SCALE, lacunarity, persistence, amplitudes)) * amplitude);
    }

    vkDestroyFence(device, fence, nullptr); vkDestroyCommandPool(device, cmdPool, nullptr); vkDestroyDescriptorPool(device, dpool, nullptr);
    vkFreeMemory(device, paramsMem, nullptr); vkFreeMemory(device, coordMem, nullptr); vkFreeMemory(device, outMem, nullptr);
    vkDestroyBuffer(device, paramsBuf, nullptr); vkDestroyBuffer(device, coordBuf, nullptr); vkDestroyBuffer(device, outBuf, nullptr);
    vkDestroyPipeline(device, pipeline, nullptr); vkDestroyPipelineLayout(device, pipelineLayout, nullptr); vkDestroyDescriptorSetLayout(device, dsl, nullptr); vkDestroyShaderModule(device, shader, nullptr);
    vkDestroyDevice(device, nullptr); vkDestroyInstance(instance, nullptr);
    std::printf("[done]\n");
    return 0;
}
