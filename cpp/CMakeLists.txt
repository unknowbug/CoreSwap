cmake_minimum_required(VERSION 3.20)
project(worldgen CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(NOT DEFINED ENV{JAVA_HOME})
    message(FATAL_ERROR "JAVA_HOME 未设置，需要指向 JDK 安装目录")
endif()

find_path(JNI_INCLUDE_DIR jni.h PATHS "$ENV{JAVA_HOME}/include" NO_DEFAULT_PATH REQUIRED)
find_path(JNI_MD_INCLUDE_DIR jni_md.h PATHS "$ENV{JAVA_HOME}/include/win32" NO_DEFAULT_PATH REQUIRED)

# worldgen 核心静态库（Phase 3 填实现）
add_library(worldgen_core STATIC src/worldgen.cpp include/worldgen.h)
target_include_directories(worldgen_core PUBLIC include)

# JNI 桥动态库
add_library(worldgen_jni SHARED src/jni_bridge.cpp)
target_include_directories(worldgen_jni PRIVATE include ${JNI_INCLUDE_DIR} ${JNI_MD_INCLUDE_DIR})
target_link_libraries(worldgen_jni PRIVATE worldgen_core)
set_target_properties(worldgen_jni PROPERTIES PREFIX "" OUTPUT_NAME "worldgen")
