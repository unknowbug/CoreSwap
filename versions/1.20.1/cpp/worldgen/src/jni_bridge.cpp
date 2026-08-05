#include <jni.h>
#include <cstring>
#include <string>
#include <vector>
#include "worldgen.h"
#include "worldgen_api.h"

extern "C" {

JNIEXPORT jlong JNICALL
Java_wg_WorldGen_nativeProbe(JNIEnv* /*env*/, jclass /*cls*/,
                             jlong seed, jint x, jint z) {
    return static_cast<jlong>(wg::probe(seed, x, z));
}

// ---- CoreSwap worldgen JNI 桥 ----

JNIEXPORT jlong JNICALL
Java_wg_CppWorldgen_init(JNIEnv* env, jclass /*cls*/, jlong seed, jstring worldgenDir) {
    const char* dir = worldgenDir ? env->GetStringUTFChars(worldgenDir, nullptr) : nullptr;
    if (!dir) return 0;
    void* h = wg_create((int64_t)seed, dir);
    env->ReleaseStringUTFChars(worldgenDir, dir);
    return reinterpret_cast<jlong>(h);
}

JNIEXPORT void JNICALL
Java_wg_CppWorldgen_destroy(JNIEnv* /*env*/, jclass /*cls*/, jlong handle) {
    wg_destroy(reinterpret_cast<void*>(handle));
}

// out: double[]，大小 = size*size*pointsPerChunk
JNIEXPORT jint JNICALL
Java_wg_CppWorldgen_fillDensity(JNIEnv* env, jclass /*cls*/, jlong handle,
                                jint minChunkX, jint minChunkZ, jint size,
                                jdoubleArray out) {
    if (!handle || !out) return 0;
    jsize len = env->GetArrayLength(out);
    jsize needed = (jsize)size * size * (jsize)wg_density_points_per_chunk(reinterpret_cast<void*>(handle));
    if (len < needed) return 0;
    double* buf = env->GetDoubleArrayElements(out, nullptr);
    int points = wg_fill_density(reinterpret_cast<void*>(handle), minChunkX, minChunkZ, size, buf);
    env->ReleaseDoubleArrayElements(out, buf, 0);
    return points;
}

JNIEXPORT jint JNICALL
Java_wg_CppWorldgen_densityParams(JNIEnv* env, jclass /*cls*/, jlong handle,
                                  jintArray out4) {
    // out4 = {xzInterval, yInterval, minY, height}
    jint* b = env->GetIntArrayElements(out4, nullptr);
    b[0] = wg_density_xz_interval(reinterpret_cast<void*>(handle));
    b[1] = wg_density_y_interval(reinterpret_cast<void*>(handle));
    b[2] = wg_min_y(reinterpret_cast<void*>(handle));
    b[3] = wg_height(reinterpret_cast<void*>(handle));
    env->ReleaseIntArrayElements(out4, b, 0);
    return 4;
}

// 完整区块生成（方块层）：Java wg.CppWorldgen.fillBlocks(long, int[], int[], int[][], int)
// chunkXs/chunkZs：count 个 chunk 坐标；outs[i] = int[16*16*384]（vanilla raw block id）
// threads <= 0 自适应；返回 count
JNIEXPORT jint JNICALL
Java_wg_CppWorldgen_fillBlocks(JNIEnv* env, jclass /*cls*/, jlong handle,
                               jintArray chunkXs, jintArray chunkZs,
                               jobjectArray outs, jint threads) {
    if (!handle || !chunkXs || !chunkZs || !outs) return 0;
    jsize count = env->GetArrayLength(chunkXs);
    if (count <= 0 || env->GetArrayLength(chunkZs) != count || env->GetArrayLength(outs) != count) return 0;
    int* cxs = env->GetIntArrayElements(chunkXs, nullptr);
    int* czs = env->GetIntArrayElements(chunkZs, nullptr);
    // 安全模式：C++ 内部线程池写「本地 buffer」，主线程再拷回 Java 数组。
    // 直接让 C++ 线程写 GetIntArrayElements 指针在部分 JVM 上不安全（pin 语义 + 跨线程写）。
    constexpr jsize BLOCK_COUNT = 16 * 16 * 384;
    std::vector<std::vector<int32_t>> local((size_t)count, std::vector<int32_t>(BLOCK_COUNT));
    std::vector<int32_t*> bufs((size_t)count);
    for (int i = 0; i < count; i++) bufs[(size_t)i] = local[(size_t)i].data();
    int r = wg_fill_blocks_multi(reinterpret_cast<void*>(handle), cxs, czs, bufs.data(), (int)count, (int)threads);
    // 主线程拷回（r 表示成功生成的 chunk 数）
    for (int i = 0; i < r && i < count; i++) {
        jintArray arr = (jintArray)env->GetObjectArrayElement(outs, i);
        if (!arr) continue;
        jint* dst = env->GetIntArrayElements(arr, nullptr);
        if (dst) {
            std::memcpy(dst, local[(size_t)i].data(), BLOCK_COUNT * sizeof(int32_t));
            env->ReleaseIntArrayElements(arr, dst, 0);
        }
    }
    env->ReleaseIntArrayElements(chunkXs, cxs, 0);
    env->ReleaseIntArrayElements(chunkZs, czs, 0);
    return r;
}

} // extern "C"
