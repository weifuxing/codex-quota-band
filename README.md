<p align="right"><a href="README_EN.md">English</a></p>

# Codex额度 · 黄山派蓝牙看板

Windows → 加密 BLE → 黄山派，在标配390×450 AMOLED上显示真实额度、重置倒计时和连接状态，无需手机。

<p align="center"><img src="assets/icon.svg" alt="Codex额度" width="96"></p>

当前版本：**0.6.5**。main 专注 Windows＋黄山派；原Android、小米手环、AstroBox和局域网同步工程保留在 [codex/legacy-android-band 分支](https://github.com/weifuxing/codex-quota-band/tree/codex/legacy-android-band)。

## 下载与连接

从 [GitHub Releases](https://github.com/weifuxing/codex-quota-band/releases) 下载：

| 文件 | 用途 |
| --- | --- |
| CodexQuota-Setup-0.6.5.exe | Windows 10/11 x64安装包 |
| CodexQuota-Huangshan-0.6.5.zip | 黄山派固件、刷写脚本和说明 |
| SHA256SUMS.txt | 安装包和固件包校验值 |

1. 安装Windows程序，确保电脑支持BLE且已登录Codex，可获得官方额度。
2. 解压固件包，按包内README先备份再刷写，仅适用`sf32lb52-lchspi-ulp`及标配屏幕。
3. 在Windows蓝牙设置添加`CodexQuota-HSPI`，输入黄山派屏幕显示的6位PIN。
4. 托盘中的“黄山派蓝牙同步”默认开启，也可关闭；“刷新额度”主动确认，“开机启动”控制自启动。

软件没有手机配对窗口、局域网服务、手环桥接或任务Hook功能。上游额度每45秒确认一次，本地每5秒读取，BLE自动重连。断线缓存保留真实确认时间，过期数据不会伪装成实时。

## 黄山派蓝牙直连

黄山派通过已配对、经身份验证和加密的 BLE 连接电脑，日常使用无需 Android 或小米运动健康。本仓库 main 仅包含 Windows 软件与黄山派固件。

### 运行界面

<table>
  <tr>
    <th align="center">仅周额度 · 单圈</th>
    <th align="center">双额度 · 同心环</th>
    <th align="center">双额度 · 数据行</th>
  </tr>
  <tr>
    <td align="center"><img src="docs/huangshan-ui/weekly-concentric.png" alt="黄山派仅周额度单圈界面，重置时间位于环内" width="230"></td>
    <td align="center"><img src="docs/huangshan-ui/dual-concentric.png" alt="黄山派同心双环界面，外圈5小时额度、内圈周额度，底部可触摸选择" width="230"></td>
    <td align="center"><img src="docs/huangshan-ui/dual-rows.png" alt="黄山派双环和数据行界面，分别显示两项额度与重置时间" width="230"></td>
  </tr>
</table>

以上为 390×450 界面的示例数据预览，非真机拍摄；实际数值由套餐和同步结果决定。KEY2 切换同心环与数据行布局；当前账号的单圈界面已通过真机验证，双窗口触摸仍待对应套餐验证。

### 显示与操作

- 套餐只有周额度时显示单圈，同时提供 5 小时和周额度时显示同心双环；未知数据与套餐不适用分别处理。
- 剩余额度低于 20% 为红色，20–49% 为黄色，50% 及以上为绿色。5 小时重置倒计时使用 `xhxxm`，周额度使用 `xdxxh`。
- 短按 KEY2 在同心环与“环形图＋数据行”布局之间切换；双窗口时可触摸切换中央额度。开机默认同心布局。
- 断线保留本次开机的 RAM 缓存，并显示离线状态和原始数据年龄；Windows 自动重连。断电重启后等待重新同步。

### OLED 保护

纯黑背景、正常亮度 30%，显示内容每分钟移动一次，最大偏移 ±2px。持续断线 3 分钟后息屏；触摸、短按 KEY2 或重新收到有效蓝牙数据可唤醒。首次触摸或按键只负责唤醒，保持原布局；离线唤醒后重新计时 3 分钟。

无有效蓝牙数据 45 秒会判定断线，随后开始息屏计时。连接正常时保持亮屏，额度来源变为缓存不会单独触发息屏。息屏采用全黑画面和亮度归零，保留触摸共用供电；这是停止显示发光，屏幕电路与蓝牙仍在工作。

### 构建与首次连接

本机使用 [SiFli SDK](https://github.com/OpenSiFli/SiFli-SDK)，板级为 `sf32lb52-lchspi-ulp`。完成 SDK 环境安装后，在仓库根目录运行：

```powershell
.\huangshan-app\scripts\test.ps1
.\huangshan-app\scripts\build.ps1 -SdkRoot D:\UGit\OpenSiFli\sifli-sdk
.\windows-native\scripts\build-huangshan.ps1 -UseMirror
```

1. 按[黄山派工程说明](huangshan-app/README.md)备份受影响区域，并刷入对应的 `bootloader`、`main` 和 `ftab`；保留配对数据，不整片擦除。
2. 在 Windows「设置 → 蓝牙和设备 → 添加设备」中选择 `CodexQuota-HSPI`，输入屏幕显示的 6 位配对码。
3. 运行 `.\windows-native\scripts\run-huangshan.ps1` 启动本地便携候选；或在候选的托盘菜单启用「黄山派蓝牙同步」。之后自动恢复配对连接。

便携候选不会替换原来的安装包或登录启动配置。构建环境、协议和验证范围见[黄山派说明](huangshan-app/README.md)、[BLE 协议](contract/ble-quota-v2.md)和[当前状态](docs/current-status.md)。


## 隐私与开发

只同步额度摘要、重置时间与连接状态。访问令牌只在Windows进程内存中用于官方接口，不传给黄山派，不写日志。无云中转、遥测、广告和自动崩溃上报。

- [开发与构建](docs/development-guide.md)
- [架构](docs/architecture.md)
- [安全说明](docs/security.md)
- [当前状态](docs/current-status.md)

MIT开源项目，非OpenAI或SiFli官方产品。中文字体使用OFL许可。
