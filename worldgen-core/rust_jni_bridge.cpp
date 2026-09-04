// rust_jni_bridge.cpp — Rust worldgen JNI 桥（worldgen.dll 入口）
// 加载 Rust WorldgenRust.dll（导出 wg_* C ABI），导出 Java_wg_CppWorldgen_* JNI 函数。
// 对齐 C++ jni_bridge.cpp（Java wg.CppWorldgen 的 JNI 声明）。
// 编译：cl /LD rust_jni_bridge.cpp /I<jdk include> /link /OUT:worldgen.dll
#include <jni.h>
#include <windows.h>
#include <cstring>
#include <cstdio>
#include <vector>

// ---- Rust WorldgenRust.dll 的 wg_* C ABI（对齐 worldgen_api.h）----
typedef void* (*wg_create_fn)(int64_t, const char*, const char*, const char*, int);
typedef void (*wg_destroy_fn)(void*);
typedef int (*wg_fill_blocks_multi_fn)(void*, const int*, const int*, int32_t* const*, int, int);
typedef void (*wg_set_beardifier_fn)(void*, int, int, const int*, int, const int*, int);
typedef void (*wg_clear_beardifier_fn)(void*);
typedef int (*wg_density_xz_interval_fn)(void*);
typedef int (*wg_density_y_interval_fn)(void*);
typedef int (*wg_min_y_fn)(void*);
typedef int (*wg_height_fn)(void*);
typedef int (*wg_density_points_per_chunk_fn)(void*);
typedef int (*wg_fill_density_fn)(void*, int, int, int, double*);

static wg_create_fn wg_create;
static wg_destroy_fn wg_destroy;
static wg_fill_blocks_multi_fn wg_fill_blocks_multi;
static wg_set_beardifier_fn wg_set_beardifier;
static wg_clear_beardifier_fn wg_clear_beardifier;
static wg_density_xz_interval_fn wg_density_xz_interval;
static wg_density_y_interval_fn wg_density_y_interval;
static wg_min_y_fn wg_min_y;
static wg_height_fn wg_height;
static wg_density_points_per_chunk_fn wg_density_points_per_chunk;
static wg_fill_density_fn wg_fill_density;

// 加载 Rust dll（DLL 初始化时）
static bool loadRustDll() {
    // 优先同目录 WorldgenRust.dll；否则 -Dcpp.rust.lib 指定
    HMODULE dll = LoadLibraryA("WorldgenRust.dll");
    if (!dll) {
        const char* lib = getenv("CPP_RUST_LIB");
        if (lib) dll = LoadLibraryA(lib);
    }
    if (!dll) { fprintf(stderr, "[RUST-JNI] cannot load WorldgenRust.dll\n"); return false; }
    wg_create = (wg_create_fn)GetProcAddress(dll, "wg_create");
    wg_destroy = (wg_destroy_fn)GetProcAddress(dll, "wg_destroy");
    wg_fill_blocks_multi = (wg_fill_blocks_multi_fn)GetProcAddress(dll, "wg_fill_blocks_multi");
    wg_set_beardifier = (wg_set_beardifier_fn)GetProcAddress(dll, "wg_set_beardifier");
    wg_clear_beardifier = (wg_clear_beardifier_fn)GetProcAddress(dll, "wg_clear_beardifier");
    wg_density_xz_interval = (wg_density_xz_interval_fn)GetProcAddress(dll, "wg_density_xz_interval");
    wg_density_y_interval = (wg_density_y_interval_fn)GetProcAddress(dll, "wg_density_y_interval");
    wg_min_y = (wg_min_y_fn)GetProcAddress(dll, "wg_min_y");
    wg_height = (wg_height_fn)GetProcAddress(dll, "wg_height");
    wg_density_points_per_chunk = (wg_density_points_per_chunk_fn)GetProcAddress(dll, "wg_density_points_per_chunk");
    wg_fill_density = (wg_fill_density_fn)GetProcAddress(dll, "wg_fill_density");
    if (!wg_create || !wg_destroy || !wg_fill_blocks_multi) {
        fprintf(stderr, "[RUST-JNI] missing wg_* exports\n");
        return false;
    }
    fprintf(stderr, "[RUST-JNI] Rust worldgen.dll bridge loaded (WorldgenRust.dll attached)\n");
    return true;
}

extern "C" {

JNIEXPORT jlong JNICALL
Java_wg_CppWorldgen_init(JNIEnv* env, jclass, jlong seed, jstring worldgenDir) {
    if (!wg_create) { if (!loadRustDll()) return 0; }
    const char* dir = worldgenDir ? env->GetStringUTFChars(worldgenDir, nullptr) : nullptr;
    if (!dir) return 0;
    void* h = wg_create((int64_t)seed, dir, nullptr, nullptr, 0);
    env->ReleaseStringUTFChars(worldgenDir, dir);
    return (jlong)h;
}

JNIEXPORT void JNICALL
Java_wg_CppWorldgen_destroy(JNIEnv*, jclass, jlong handle) {
    if (wg_destroy) wg_destroy((void*)handle);
}

JNIEXPORT jint JNICALL
Java_wg_CppWorldgen_fillBlocks(JNIEnv* env, jclass, jlong handle,
                               jintArray chunkXs, jintArray chunkZs,
                               jobjectArray outs, jint threads) {
    if (!handle || !chunkXs || !chunkZs || !outs || !wg_fill_blocks_multi) return 0;
    jsize count = env->GetArrayLength(chunkXs);
    if (count <= 0 || env->GetArrayLength(chunkZs) != count || env->GetArrayLength(outs) != count) return 0;
    int* cxs = env->GetIntArrayElements(chunkXs, nullptr);
    int* czs = env->GetIntArrayElements(chunkZs, nullptr);
    constexpr jsize BLOCK_COUNT = 16 * 16 * 384;
    std::vector<std::vector<int32_t>> local((size_t)count, std::vector<int32_t>(BLOCK_COUNT));
    std::vector<int32_t*> bufs((size_t)count);
    for (int i = 0; i < count; i++) bufs[(size_t)i] = local[(size_t)i].data();
    int r = wg_fill_blocks_multi((void*)handle, cxs, czs, bufs.data(), (int)count, (int)threads);
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

JNIEXPORT void JNICALL
Java_wg_CppWorldgen_setBeardifier(JNIEnv* env, jclass, jlong handle,
                                  jint chunkX, jint chunkZ,
                                  jintArray pieces, jint pieceCount,
                                  jintArray junctions, jint junctionCount) {
    if (!wg_set_beardifier) return;
    jint* p = pieces ? env->GetIntArrayElements(pieces, nullptr) : nullptr;
    jint* j = junctions ? env->GetIntArrayElements(junctions, nullptr) : nullptr;
    wg_set_beardifier((void*)handle, chunkX, chunkZ, p, (int)pieceCount, j, (int)junctionCount);
    if (p) env->ReleaseIntArrayElements(pieces, p, JNI_ABORT);
    if (j) env->ReleaseIntArrayElements(junctions, j, JNI_ABORT);
}

JNIEXPORT jint JNICALL
Java_wg_CppWorldgen_fillDensity(JNIEnv* env, jclass, jlong handle,
                                jint minChunkX, jint minChunkZ, jint size,
                                jdoubleArray out) {
    if (!handle || !out || !wg_fill_density) return 0;
    jsize len = env->GetArrayLength(out);
    jsize needed = (jsize)size * size * (jsize)wg_density_points_per_chunk((void*)handle);
    if (len < needed) return 0;
    double* buf = env->GetDoubleArrayElements(out, nullptr);
    int points = wg_fill_density((void*)handle, minChunkX, minChunkZ, size, buf);
    env->ReleaseDoubleArrayElements(out, buf, 0);
    return points;
}

JNIEXPORT jint JNICALL
Java_wg_CppWorldgen_densityParams(JNIEnv* env, jclass, jlong handle, jintArray out4) {
    jint* b = env->GetIntArrayElements(out4, nullptr);
    b[0] = wg_density_xz_interval((void*)handle);
    b[1] = wg_density_y_interval((void*)handle);
    b[2] = wg_min_y((void*)handle);
    b[3] = wg_height((void*)handle);
    env->ReleaseIntArrayElements(out4, b, 0);
    return 4;
}

} // extern "C"
