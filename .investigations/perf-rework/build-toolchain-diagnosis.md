# 构建工具链诊断 + 正确构建路径（2026-08-16）

> 状态：draft（主会话临时排查记录）
> 背景：改 density.h 触发重建时，「编译很慢/卡死」——用户指出「这点代码不该超 15 秒」+「之前 MinGW 闯过祸」，怀疑工具链污染。

## 结论（五条）

1. **MSVC 编译器纯洁**：cl.exe 用 `D:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64\cl.exe`（build.ninja 绝对路径）。/showIncludes 单文件编译 3-6s 正常。

2. **ninja 工具链被 pip-ninja 污染**（= 用户「MinGW 闯祸」同类机制）：
   - `ninja` 命令实际 = `D:\Program Files\Python\Python312\Scripts\ninja.exe`（pip 装，**1.11.1.git.kitware.jobserver-1**）
   - CMake 记录的正版 = `D:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe`（VS 自带，**1.13.2**）
   - **pip-ninja 遮蔽官方 ninja**（PATH 顺序：Python312\Scripts 在前）——非官方工具链混入最前 PATH，正是 MinGW 同款问题。

3. **真凶 = ninja 的工作模式在沙箱挂起，非编译慢**：
   - `cmd /c "call vcvars64.bat && ninja"` 一律卡 120s（无论 pip-ninja 还是 VS 官方 ninja 1.13.2、无论 -j1）
   - **手动 `cl` + `lib` 直链全部成功（单文件 3-6s，整链 30s）**
   - **`cmd /c "call vcvars64.bat && cl ..."`（不经 ninja，只跑 cl）10.8s 秒过** → `cmd /c` 包装本身不卡
   - **结论**：卡死的是 **ninja 自身**（spawn 子进程 + 捕获 /showIncludes 输出管道），沙箱限制「父子进程 + stdio 管道」模式（工具文档明示 EPERM/挂起）。**ninja 版本污染是隐患，但非卡死根因**。

4. **📌 修复（2026-08-16）**：新增 `cpp/build.ps1`（cl + lib 直链，替代 ninja）——编译 worldgen_core.lib 的 4 源文件 + 打包静态库 + 链接常用 exe（block_probe/bench_chunks/...），带 -Target/-All/-Clean 参数。实测：整链 30.7s（含 gpu_density_engine 的 Vulkan/226KB cpu_backend 头），bench_chunks.exe 运行正常（seed 8576 T=1 75.42ms）。**项目构建改为 `pwsh build.ps1`，脱离 ninja 沙箱卡死。**
   - build.ps1 保留 CMakeLists 的构建铁律注释（严格 MSVC / 禁 MinGW / /utf-8 /DNOMINMAX /EHsc）

5. **pip-ninja 污染待处理（非阻断）**：build.ps1 不用 ninja，故构建不受影响；但 PATH 里 Python312\Scripts 的 ninja 仍会遮蔽 VS 官方 ninja，若将来要用 ninja（或其他依赖 ninja 的 CMake 流程）需处理。建议：改 PATH 顺序 / 重命名 pip-ninja / 用 VS 官方 ninja 绝对路径。

## 正确构建路径（已验证可靠，build.ps1 已封装）

```
pwsh versions/1.20.1/cpp/build.ps1                 # 构建 worldgen_core.lib + block_probe + bench_chunks
pwsh versions/1.20.1/cpp/build.ps1 -Target bench_chunks
pwsh versions/1.20.1/cpp/build.ps1 -All
```

**Vulkan lib 路径**：`C:\VulkanSDK\1.4.357.0\Lib\vulkan-1.lib`（**无 x64 子目录**——曾误用 `Lib\x64\vulkan-1.lib` 导致 LNK1181）。

## 教训（错误记录）

1. **非官方工具链混入最前 PATH = MinGW 同类祸**：pip-ninja（Python312\Scripts）遮蔽 VS 官方 ninja，版本不一致（1.11.1 vs 1.13.2）产生怪异行为。**先查 `Get-Command ninja` 来源，再怀疑编译慢**。
2. **「编译慢」先区分「编译算力慢」vs「驱动挂起」**：单文件 cl 3-6s（编译快）vs cmd/ninja 卡 120s（驱动挂起）——**用单文件 cl 计时做基准**，别被工具驱动层的假象误导。
3. **超时设 30s 而非 120s**（用户修正）：编译基准 3-10s，30s 不出即异常，应立即止损排查，而非干等。
4. **ninja 依赖 /showIncludes 管道捕获**——沙箱下该管道挂起，**`cl`+`lib` 直链是可靠替代**。
5. **区分「工具链被污染」vs「工具在沙箱不支持」**：pip-ninja 污染是隐患（要修），ninja 沙箱卡死是环境限制（绕过即可，换 ninja 版本无用）。**先定位是哪种，再动手修。**
