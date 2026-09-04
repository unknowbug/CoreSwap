// dll_test.c - verify Rust WorldgenRust.dll C ABI exports via LoadLibrary
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <windows.h>

typedef void* (*wg_create_fn)(int64_t, const char*, const char*, const char*, int);
typedef void (*wg_destroy_fn)(void*);
typedef int (*wg_fill_blocks_multi_fn)(void*, const int*, const int*, int32_t**, int, int);
typedef int (*wg_min_y_fn)(void*);
typedef int (*wg_height_fn)(void*);

int main(int argc, char** argv) {
    if (argc < 2) { fprintf(stderr, "usage: dll_test <worldgenDir>\n"); return 1; }
    HMODULE dll = LoadLibraryA("WorldgenRust.dll");
    if (!dll) { fprintf(stderr, "LoadLibrary FAILED err=%lu\n", GetLastError()); return 1; }
    wg_create_fn wg_create = (wg_create_fn)GetProcAddress(dll, "wg_create");
    wg_destroy_fn wg_destroy = (wg_destroy_fn)GetProcAddress(dll, "wg_destroy");
    wg_fill_blocks_multi_fn wg_fill = (wg_fill_blocks_multi_fn)GetProcAddress(dll, "wg_fill_blocks_multi");
    wg_min_y_fn wg_min_y = (wg_min_y_fn)GetProcAddress(dll, "wg_min_y");
    wg_height_fn wg_height = (wg_height_fn)GetProcAddress(dll, "wg_height");
    if (!wg_create || !wg_destroy || !wg_fill || !wg_min_y || !wg_height) {
        fprintf(stderr, "GetProcAddress FAILED (create=%p destroy=%p fill=%p miny=%p h=%p)\n",
                (void*)wg_create, (void*)wg_destroy, (void*)wg_fill, (void*)wg_min_y, (void*)wg_height);
        return 1;
    }
    printf("all exports found\n");

    int64_t seed = -8248318472910187742LL;
    void* h = wg_create(seed, argv[1], NULL, NULL, 0);
    if (!h) { fprintf(stderr, "wg_create FAILED\n"); return 1; }
    printf("wg_create OK, min_y=%d height=%d\n", wg_min_y(h), wg_height(h));

    const int N = 16;
    int* cxs = (int*)malloc(N * sizeof(int));
    int* czs = (int*)malloc(N * sizeof(int));
    int32_t** outs = (int32_t**)malloc(N * sizeof(int32_t*));
    int32_t** bufs = (int32_t**)malloc(N * sizeof(int32_t*));
    for (int i = 0; i < N; i++) bufs[i] = (int32_t*)malloc(16*16*384 * sizeof(int32_t));
    int n = 0;
    for (int cz = -256; cz < -256+4; cz++) {
        for (int cx = -288; cx < -288+4; cx++) {
            cxs[n] = cx; czs[n] = cz; outs[n] = bufs[n]; n++;
        }
    }
    int r = wg_fill(h, cxs, czs, outs, n, 1);
    printf("wg_fill_blocks_multi returned %d (want %d)\n", r, n);
    long long nz = 0;
    for (int i = 0; i < n; i++)
        for (int k = 0; k < 16*16*384; k++) if (bufs[i][k] != 0) nz++;
    printf("non-air blocks: %lld\n", nz);
    wg_destroy(h);
    printf("wg_destroy OK\n");
    for (int i = 0; i < N; i++) free(bufs[i]);
    free(bufs); free(outs); free(cxs); free(czs);
    FreeLibrary(dll);
    return 0;
}
