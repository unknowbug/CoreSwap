# Rust 安装流程（Windows，含镜像/代理——解决网络卡住）

> 用户环境自装。Coreswap 项目已有 MSVC（link.exe），Rust `x86_64-pc-windows-msvc` 复用。
> 背景：沙箱下 rustup 下载 stable 工具链疑似被代理/网络卡住（长时间无进展），故用户自行安装。

## 1. 先测网（确认是否官方连不上）
```powershell
Test-NetConnection static.rust-lang.org -Port 443
# 若超时/失败 → 用镜像（第2步）
```

## 2. 设镜像（推荐，绕官方慢/代理）
```powershell
# rsproxy.cn（国内快，推荐）
$env:RUSTUP_DIST_SERVER="https://rsproxy.cn"
$env:RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

# 中科大备用：
# $env:RUSTUP_DIST_SERVER="https://mirrors.ustc.edu.cn/rust-static"
# $env:RUSTUP_UPDATE_ROOT="https://mirrors.ustc.edu.cn/rust-static/rustup"
```

## 3. 下载 + 安装 rustup-init
```powershell
# 镜像版 rustup-init（与第2步镜像对应）
Invoke-WebRequest "https://rsproxy.cn/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile "$env:TEMP\rustup-init.exe"

# 官方版（镜像不行时）：
# Invoke-WebRequest "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile "$env:TEMP\rustup-init.exe"

# 安装（minimal = 最小工具链，够编译）
& "$env:TEMP\rustup-init.exe" --default-toolchain stable --profile minimal -y
```

## 4. 企业代理（rustup 下载卡住时）
```powershell
$env:HTTP_PROXY="http://你的代理:端口"
$env:HTTPS_PROXY="http://你的代理:端口"
# 若用镜像（第2步），一般无需代理；代理 + 镜像可能冲突，二选一。
```

## 5. 验证
```powershell
rustc --version    # 应出版本号（如 rustc 1.84.0）
cargo --version
```

## 6. PATH（找不到 rustc/cargo 时）
```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"
# 或系统环境变量里把 %USERPROFILE%\.cargo\bin 加到 PATH
```

## 7. 编译报 link.exe 找不到（链接器）
你项目已有 VS MSVC（link.exe）。Rust msvc 后端复用。若报错：
```powershell
rustup default stable-x86_64-pc-windows-msvc
# 或先 `call vcvars64.bat`（VS 环境），让 link.exe 在 PATH
```

## 关键
- **先测网**（第1步）→ 官方连不上就**用镜像**（第2步 rsproxy.cn）。
- **镜像**绕官方慢/代理（正对你怀疑的卡住问题）。
- **minimal profile** 够编译，不必全装（省时间/带宽）。
- 装完 `rustc --version` 出版本号 = 成功。

## 装好后（回到本课题）
用 `rustc -O mlp_probe.rs`（`E:\PYTHON\CoreSwap\.investigations\perf-rework\mlp_probe.rs`，已写好）编译，跑：
```
mlp_probe 400000 1   # seq
mlp_probe 400000 2   # soft4
mlp_probe 400000 3   # soft8
```
对比 C++ `mlp_probe.exe` 的 -36%（soft8），确认 Rust 软流增益 + Rust 环境。
