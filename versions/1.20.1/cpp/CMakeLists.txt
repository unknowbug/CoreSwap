cmake_minimum_required(VERSION 3.20)
project(core_swap CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# MinGW 链接：C++ 运行时静态（自包含，dll 只依赖 msvcrt.dll）+ CRT 动态
# 注意：全 -static 时 MinGW 的 thread_local 会退化为跨线程共享（多线程并发写缓存 → 堆损坏）
if(MINGW)
    add_compile_options(-static-libgcc -static-libstdc++)
    add_link_options(-static-libgcc -static-libstdc++)
endif()

# MSVC：源码为 UTF-8（含中文注释）；NOMINMAX 防 windows.h min/max 宏冲突；/EHsc 异常展开
if(MSVC)
    add_compile_options(/utf-8 /DNOMINMAX /EHsc)
endif()

# CoreSwap 各核心模块
add_subdirectory(worldgen)
# add_subdirectory(ai)   # 实体 AI/寻路 模块（第二刀，开发中）
