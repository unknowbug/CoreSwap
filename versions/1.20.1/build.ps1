# 构建脚本（1.20.1）：编译 C++ 核心 + JNI DLL + Java JNI 测试
# 用法: powershell -File versions\1.20.1\build.ps1
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent (Split-Path -Parent $root)
$env:JAVA_HOME = "$repoRoot\tools\jdk17\jdk-17.0.20+8"
$env:Path = "$repoRoot\tools\mingw\mingw64\bin;$env:JAVA_HOME\bin;" + $env:Path

Write-Host "== CMake configure =="
cmake -S "$root\cpp" -B "$root\cpp\build" -G "MinGW Makefiles" -DCMAKE_BUILD_TYPE=Release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "== CMake build =="
cmake --build "$root\cpp\build" --config Release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "== javac =="
javac -d "$root\java\jnitest\out" "$root\java\jnitest\wg\WorldGen.java"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "== run JNI test =="
java "-Djava.library.path=$root\cpp\build\bin" -cp "$root\java\jnitest\out" wg.WorldGen 123456789 42 -17

Write-Host "BUILD OK"
