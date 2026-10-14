# 开发与构建

仓库只包含Windows与黄山派。旧工程在codex/legacy-android-band。

## Windows

安装Rust stable MSVC、Visual Studio Build Tools C++、Windows SDK和NSIS。默认工具链为stable-x86_64-pc-windows-msvc。

```powershell
Set-Location windows-native
cargo test --workspace
cargo fmt --check
.\scripts\build-installer.ps1
```

build-installer.ps1支持-Toolchain、-UseMirror和-MakeNsis自定义工具路径。运行候选时可指定独立CARGO_TARGET_DIR；不覆盖运行中的EXE。安装包构建会检查ICON/GROUP_ICON和最小系统PATH独立启动。test-installer.ps1在已有安装时拒绝覆盖，应记录该限制。

## 黄山派

```powershell
.\huangshan-app\scripts\test.ps1
.\huangshan-app\scripts\build.ps1 -SdkRoot D:\UGit\OpenSiFli\sifli-sdk
.\huangshan-app\scripts\package.ps1
```

本机SDK的Python为C:\Users\fuxing\.sifli\python_env\sifli-sdk2.4_py3.13_env\Scripts\python.exe，GCC14.2.1；其他环境可用脚本参数修改。依赖按SiFli官方环境安装；本机补充依赖位于忽略目录out/huangshan-python-deps。必须AddLCPU，SDK不作修改。

package.ps1打包已验收的三个镜像、校验manifest、Flash.ps1、说明和许可。Flash.ps1先备份再写入并verify，不擦配对区；详见huangshan-app/FIRMWARE.md。不要把原设备备份加入发布包。
