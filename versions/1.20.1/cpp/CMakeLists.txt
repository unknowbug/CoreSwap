cmake_minimum_required(VERSION 3.20)
project(core_swap CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# MinGW 静态链接运行时：exe/dll 自包含
if(MINGW)
    add_compile_options(-static)
    add_link_options(-static)
endif()

# CoreSwap 各核心模块
add_subdirectory(worldgen)
# add_subdirectory(ai)   # 实体 AI/寻路 模块（第二刀，开发中）
