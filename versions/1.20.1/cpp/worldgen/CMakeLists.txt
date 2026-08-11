cmake_minimum_required(VERSION 3.20)
project(worldgen CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# ⚠️ 构建铁律：严格 MSVC（用户禁用 MinGW——thread_local 语义 + 堆损坏问题，见顶层 CMakeLists 注释）

# 统一输出目录：cpp/build/bin/（无论模块嵌套多深，产物路径稳定）
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

if(NOT DEFINED ENV{JAVA_HOME})
    message(FATAL_ERROR "JAVA_HOME 未设置，需要指向 JDK 安装目录")
endif()

find_path(JNI_INCLUDE_DIR jni.h PATHS "$ENV{JAVA_HOME}/include" NO_DEFAULT_PATH REQUIRED)
find_path(JNI_MD_INCLUDE_DIR jni_md.h PATHS "$ENV{JAVA_HOME}/include/win32" NO_DEFAULT_PATH REQUIRED)

# worldgen 核心静态库（Phase 3 填实现）
add_library(worldgen_core STATIC src/worldgen.cpp src/md5.cpp src/worldgen_api.cpp include/worldgen.h src/md5.h src/random.h src/xoroshiro.h src/noise.h include/worldgen_api.h)
target_include_directories(worldgen_core PUBLIC include src)

# JNI 桥动态库
add_library(worldgen_jni SHARED src/jni_bridge.cpp)
target_include_directories(worldgen_jni PRIVATE include src ${JNI_INCLUDE_DIR} ${JNI_MD_INCLUDE_DIR})
target_link_libraries(worldgen_jni PRIVATE worldgen_core)
set_target_properties(worldgen_jni PROPERTIES PREFIX "" OUTPUT_NAME "worldgen")

# 3a 噪声探针（验证用）
add_executable(noise_probe src/noise_probe.cpp)
target_include_directories(noise_probe PRIVATE include src)
target_link_libraries(noise_probe PRIVATE worldgen_core)

# 3c 密度场探针（验证用）
add_executable(density_probe src/density_probe.cpp)
target_include_directories(density_probe PRIVATE include src)
target_link_libraries(density_probe PRIVATE worldgen_core)

# 3c router 分量探针（验证用）
add_executable(router_probe src/router_probe.cpp)
target_include_directories(router_probe PRIVATE include src)
target_link_libraries(router_probe PRIVATE worldgen_core)

# 3d ore 矿脉探针（验证用）
add_executable(ore_probe src/ore_probe.cpp)
target_include_directories(ore_probe PRIVATE include src)
target_link_libraries(ore_probe PRIVATE worldgen_core)

# 方块层探针（验证用）
add_executable(block_probe src/block_probe.cpp)
target_include_directories(block_probe PRIVATE include src)
target_link_libraries(block_probe PRIVATE worldgen_core)

# got 导出（差异分析用）
add_executable(got_export src/got_export.cpp)
target_include_directories(got_export PRIVATE include src)
target_link_libraries(got_export PRIVATE worldgen_core)

# tbands 带数组导出（对比 Java RouterProbe TBANDS）
add_executable(tbands_dump src/tbands_dump.cpp)
target_include_directories(tbands_dump PRIVATE include src)
target_link_libraries(tbands_dump PRIVATE worldgen_core)

# ChunkRandom/CheckedRandom 移植单测（参照 = ChunkRandomProbe Java 输出，逐位对拍）
add_executable(chunkrandom_test src/chunkrandom_test.cpp)
target_include_directories(chunkrandom_test PRIVATE include src)
target_link_libraries(chunkrandom_test PRIVATE worldgen_core)

# 吞吐基准（SURFACE 模式 = 实机 populateNoise 覆盖路径；2026-08-11 新增）
add_executable(bench_chunks src/bench_chunks.cpp)
target_include_directories(bench_chunks PRIVATE include src)
target_link_libraries(bench_chunks PRIVATE worldgen_core)

# 崩溃定位：worldgen.dll+0x1b930（读地址 0）——生成 map 文件
if(MSVC)
  target_link_options(worldgen_jni PRIVATE "/MAP:${CMAKE_BINARY_DIR}/bin/worldgen.map")
endif()
