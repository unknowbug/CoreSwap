cmake_minimum_required(VERSION 3.20)
project(core_swap CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# ⚠️ 构建铁律：严格 MSVC（用户禁用 MinGW——MinGW -static 下 thread_local 退化 → 跨线程共享缓存 → 堆损坏 0xC0000005）
# MSVC：源码为 UTF-8（含中文注释）；NOMINMAX 防 windows.h min/max 宏冲突；/EHsc 异常展开
if(MSVC)
    add_compile_options(/utf-8 /DNOMINMAX /EHsc)
endif()

# CoreSwap 各核心模块
add_subdirectory(worldgen)
# add_subdirectory(ai)   # 实体 AI/寻路 模块（第二刀，开发中）
