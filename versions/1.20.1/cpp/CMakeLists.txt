cmake_minimum_required(VERSION 3.20)
project(worldgen CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# MinGW 静态链接运行时：exe/dll 自包含，用户无需安装 MinGW
if(MINGW)
    add_compile_options(-static)
    add_link_options(-static)
endif()

if(NOT DEFINED ENV{JAVA_HOME})
    message(FATAL_ERROR "JAVA_HOME 未设置，需要指向 JDK 安装目录")
endif()

find_path(JNI_INCLUDE_DIR jni.h PATHS "$ENV{JAVA_HOME}/include" NO_DEFAULT_PATH REQUIRED)
find_path(JNI_MD_INCLUDE_DIR jni_md.h PATHS "$ENV{JAVA_HOME}/include/win32" NO_DEFAULT_PATH REQUIRED)

# worldgen 核心静态库（Phase 3 填实现）
add_library(worldgen_core STATIC src/worldgen.cpp src/md5.cpp include/worldgen.h src/md5.h src/random.h src/xoroshiro.h src/noise.h)
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
