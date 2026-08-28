// jni_dll_test.c - verify worldgen.dll (JNI bridge) exports + loads Rust dll
#include <stdio.h>
#include <windows.h>

int main() {
    HMODULE dll = LoadLibraryA("worldgen.dll");
    if (!dll) { fprintf(stderr, "LoadLibrary worldgen.dll FAILED err=%lu\n", GetLastError()); return 1; }
    const char* names[] = {
        "Java_wg_CppWorldgen_init", "Java_wg_CppWorldgen_destroy",
        "Java_wg_CppWorldgen_fillBlocks", "Java_wg_CppWorldgen_setBeardifier",
        "Java_wg_CppWorldgen_fillDensity", "Java_wg_CppWorldgen_densityParams"
    };
    int ok = 1;
    for (int i = 0; i < 6; i++) {
        FARPROC p = GetProcAddress(dll, names[i]);
        printf("%s: %s\n", names[i], p ? "OK" : "MISSING");
        if (!p) ok = 0;
    }
    FreeLibrary(dll);
    return ok ? 0 : 1;
}
