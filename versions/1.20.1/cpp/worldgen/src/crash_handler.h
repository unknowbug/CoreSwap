// crash_handler.h —— CoreSwap 原生崩溃日志（合并进 MC 崩溃日志）
// 用 AddVectoredExceptionHandler 捕获访问违规（0xC0000005 等 SEH 异常），
// 打印：异常地址/寄存器/模块偏移/调用栈 → stderr（进 latest.log）+ 写 crash-coreswap-*.txt
// 不吞异常（返回 EXCEPTION_CONTINUE_SEARCH）——JVM 的 hs_err 照常生成，我们的日志补充函数/偏移。
#pragma once
#include <windows.h>
#include <dbghelp.h>
#include <cstdio>
#include <cstdint>
#include <ctime>
#include <string>
#include <vector>

namespace wg {

// 线程局部：当前 JNI 调用入口名（fillOneChunk 等），崩溃时打印上下文
extern thread_local const char* g_crashContext;

inline const char* moduleNameAt(const void* addr, uintptr_t& baseOut) {
    HMODULE m = nullptr;
    GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                       (LPCWSTR)addr, &m);
    baseOut = (uintptr_t)m;
    static thread_local char buf[64];
    if (m) {
        DWORD n = GetModuleFileNameA(m, buf, sizeof(buf));
        for (DWORD i = n; i > 0; i--) {
            if (buf[i - 1] == '\\' || buf[i - 1] == '/') {
                return buf + i;
            }
        }
        return buf;
    }
    return "?";
}

// 反汇编前 N 字节（hex）——便于识别崩溃指令
inline void dumpBytes(FILE* f, const uint8_t* p, int n) {
    std::fprintf(f, "bytes=");
    for (int i = 0; i < n; i++) std::fprintf(f, "%02X ", p[i]);
    std::fprintf(f, "\n");
}

inline LONG WINAPI CrashHandler(EXCEPTION_POINTERS* ep) {
    EXCEPTION_RECORD* er = ep->ExceptionRecord;
    CONTEXT* ctx = ep->ContextRecord;
    FILE* f = stderr;
    time_t now = time(nullptr);
    struct tm tmv;
    localtime_s(&tmv, &now);
    char ts[32];
    strftime(ts, sizeof(ts), "%Y%m%d-%H%M%S", &tmv);

    // 也写一个独立文件（游戏目录 = CWD）
    std::string path = "crash-coreswap-" + std::string(ts) + ".txt";
    FILE* ffile = fopen(path.c_str(), "w");

    auto log = [&](const char* fmt, ...) {
        va_list ap;
        va_start(ap, fmt);
        vfprintf(f, fmt, ap);
        va_end(ap);
        fflush(f);
        if (ffile) {
            va_start(ap, fmt);
            vfprintf(ffile, fmt, ap);
            va_end(ap);
            fflush(ffile);
        }
    };

    log("[CORESWAP-CRASH] ============ native crash ============\n");
    log("[CORESWAP-CRASH] time=%s context=%s\n", ts, g_crashContext ? g_crashContext : "?");
    log("[CORESWAP-CRASH] code=0x%08X addr=0x%p\n", er->ExceptionCode, er->ExceptionAddress);
    if (er->ExceptionCode == EXCEPTION_ACCESS_VIOLATION) {
        log("[CORESWAP-CRASH] rw=%s 0x%p\n", er->ExceptionInformation[0] ? "write" : "read",
            (void*)er->ExceptionInformation[1]);
    }
    log("[CORESWAP-CRASH] RAX=%p RBX=%p RCX=%p RDX=%p\n", (void*)ctx->Rax, (void*)ctx->Rbx,
        (void*)ctx->Rcx, (void*)ctx->Rdx);
    // 排查内存损坏：0x34001 是 fillOneChunk 里 memset 函数指针的存储位（1.0.16 用户崩溃
    // call 目标=堆地址 0x299AA——被越界写覆盖）。崩溃时打印其当前值（正常应为 msvcrt memset 地址）。
    {
        HMODULE self = GetModuleHandleA("worldgen.dll");
        if (!self) self = GetModuleHandleA("block_probe.exe");
        if (!self) self = GetModuleHandleW(nullptr);
        uintptr_t base = (uintptr_t)self;
        if (base) {
            void* p = (void*)(base + 0x34000);
            uint64_t v = 0;
            if (IsBadReadPtr(p, 8) == FALSE) memcpy(&v, p, 8);
            uint64_t v1 = 0;
            if (IsBadReadPtr((char*)p + 1, 8) == FALSE) memcpy(&v1, (char*)p + 1, 8);
            log("[CORESWAP-CRASH] data[0x34000]=0x%llX data[0x34001]=0x%llX\n", v, v1);
        }
    }
    log("[CORESWAP-CRASH] RSI=%p RDI=%p RBP=%p RSP=%p RIP=%p\n", (void*)ctx->Rsi, (void*)ctx->Rdi,
        (void*)ctx->Rbp, (void*)ctx->Rsp, (void*)ctx->Rip);

    // 崩溃指令前 16 字节（如果可读）
    const uint8_t* ip = (const uint8_t*)ctx->Rip;
    log("[CORESWAP-CRASH] ip=");
    for (int i = -8; i < 8; i++) {
        uint8_t b = 0;
        if (IsBadReadPtr(ip + i, 1) == FALSE) b = *(ip + i);
        log("%02X ", b);
    }
    log("\n");

    // 调用栈：用异常现场的 CONTEXT 做 StackWalk64（比 CaptureStackBackTrace 完整——后者在
    // vectored handler 里被异常分发栈帧污染，只剩 5 帧）。dbghelp 延迟加载。
    typedef BOOL(__stdcall* StackWalk64Fn)(DWORD, HANDLE, HANDLE, LPSTACKFRAME64, PVOID, PREAD_PROCESS_MEMORY_ROUTINE64, PFUNCTION_TABLE_ACCESS_ROUTINE64, PGET_MODULE_BASE_ROUTINE64, PTRANSLATE_ADDRESS_ROUTINE64);
    typedef DWORD64(__stdcall* SymGetModuleBase64Fn)(HANDLE, DWORD64);
    typedef BOOL(__stdcall* SymInitializeFn)(HANDLE, PCSTR, BOOL);
    typedef BOOL(__stdcall* SymFromAddrFn)(HANDLE, DWORD64, PDWORD64, PSYMBOL_INFO);
    static HMODULE dbg = LoadLibraryA("dbghelp.dll");
    static StackWalk64Fn pStackWalk = dbg ? (StackWalk64Fn)GetProcAddress(dbg, "StackWalk64") : nullptr;
    static SymGetModuleBase64Fn pModBase = dbg ? (SymGetModuleBase64Fn)GetProcAddress(dbg, "SymGetModuleBase64") : nullptr;
    static SymInitializeFn pSymInit = dbg ? (SymInitializeFn)GetProcAddress(dbg, "SymInitialize") : nullptr;
    static SymFromAddrFn pSymAddr = dbg ? (SymFromAddrFn)GetProcAddress(dbg, "SymFromAddr") : nullptr;
    static bool symInit = pSymInit ? pSymInit(GetCurrentProcess(), nullptr, TRUE) : false;
    log("[CORESWAP-CRASH] stack (StackWalk64):\n");
    STACKFRAME64 sf = {};
    sf.AddrPC.Offset = ctx->Rip;
    sf.AddrPC.Mode = AddrModeFlat;
    sf.AddrStack.Offset = ctx->Rsp;
    sf.AddrStack.Mode = AddrModeFlat;
    sf.AddrFrame.Offset = ctx->Rbp;
    sf.AddrFrame.Mode = AddrModeFlat;
    for (int i = 0; i < 24; i++) {
        if (!pStackWalk || !pStackWalk(IMAGE_FILE_MACHINE_AMD64, GetCurrentProcess(), GetCurrentThread(), &sf, ctx, nullptr, nullptr, nullptr, nullptr))
            break;
        if (sf.AddrPC.Offset == 0) break;
        uintptr_t base = 0;
        const char* mod = moduleNameAt((void*)sf.AddrPC.Offset, base);
        char fname[256] = "";
        if (symInit && pSymAddr) {
            alignas(SYMBOL_INFO) char symBuf[sizeof(SYMBOL_INFO) + 256 * sizeof(char)];
            SYMBOL_INFO* si = (SYMBOL_INFO*)symBuf;
            si->SizeOfStruct = sizeof(SYMBOL_INFO);
            si->MaxNameLen = 256;
            DWORD64 disp = 0;
            if (pSymAddr(GetCurrentProcess(), sf.AddrPC.Offset, &disp, si)) {
                snprintf(fname, sizeof(fname), " %s+0x%llX", si->Name, disp);
            }
        }
        log("  #%d %s+0x%llX%s\n", i, mod, sf.AddrPC.Offset - base, fname);
    }
    log("[CORESWAP-CRASH] ============ end ============\n");
    if (ffile) fclose(ffile);
    return EXCEPTION_CONTINUE_SEARCH;  // 不吞异常——JVM 照常出 hs_err
}

// 在 wg_create 开头调用一次（幂等）
inline void installCrashHandler() {
    static bool installed = false;
    if (installed) return;
    AddVectoredExceptionHandler(1, CrashHandler);  // 高优先（first=1）
    installed = true;
    // 打印当前 dll 的路径 + 文件大小（验证用户加载的是不是最新版——旧缓存排查）
    HMODULE self = nullptr;
    GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                       (LPCWSTR)(LPCVOID)&installCrashHandler, &self);
    char selfPath[1024] = "";
    if (self && GetModuleFileNameA(self, selfPath, sizeof(selfPath))) {
        WIN32_FILE_ATTRIBUTE_DATA fad = {};
        GetFileAttributesExA(selfPath, GetFileExInfoStandard, &fad);
        std::fprintf(stderr, "[CORESWAP] dll=%s size=%llu\n", selfPath, (unsigned long long)fad.nFileSizeLow);
    }
    std::fprintf(stderr, "[CORESWAP] crash handler installed\n");
}

} // namespace wg
