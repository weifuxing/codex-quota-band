# 黄山派 Codex额度原型

本地实验性候选，目标是“蓝牙连接 → 显示真实额度 → 断线缓存 → 自动重连”。使用标配 390×450 AMOLED；当前按套餐显示单圈或双圈额度，触摸底部按钮切换主额度；环形进度按各自剩余额度变色，重置时间为 xhxxm / xdxxh。Windows 负责采集，不需要手机端。

这是独立的 SiFli/RT-Thread/LVGL 固件，不是手环 RPK。现有 Android、手环和 WSS 链路仍保留。协议见 [BLE Quota v2](../contract/ble-quota-v2.md)。

## 构建

本机 SDK 位于 `D:\UGit\OpenSiFli\sifli-sdk`。构建产物在 `project/build_sf32lb52-lchspi-ulp_hcpu/`，SDK 本身不作修改。

```powershell
.\huangshan-app\scripts\test.ps1
.\huangshan-app\scripts\build.ps1
```

构建使用 SDK 自带 Python/SCons 和 ARM GCC。此环境缺少的 PyYAML/rich 已放入仓库的忽略目录 `out/huangshan-python-deps`；在新环境中应先按 SDK 官方指南完成环境安装。必须调用 `AddLCPU` 嵌入小核控制器镜像，仅启用 BLUETOOTH 配置无法让早期芯片版本启动控制器。

已提交的中文字体由 SDK 内的 OFL Noto Sans SC 生成，不依赖 Windows 商业字体。需要改文案时运行：

```powershell
python .\huangshan-app\tools\generate_font.py --sdk D:\UGit\OpenSiFli\sifli-sdk
```

## Windows

可以用以下命令生成独立的本地 Windows 候选：

```powershell
.\windows-native\scripts\build-huangshan.ps1 -UseMirror
```

新增托盘开关“黄山派蓝牙同步”，开关状态在应用数据目录保存；启用后连接已配对设备。使用 `--huangshan-ble` 可为当前进程启用，`--ble-probe` 只检查适配器、配对和 GATT 服务。

本机原配置的 gnullvm 工具链未安装，原型使用已安装的 MSVC 工具链。build.rs 同时支持原有 windres 和 Windows SDK rc.exe，不修改原来的 Rust 工具链声明。直接候选 EXE 不等于安装包。

## 首次连接

1. 刷入经本地构建的 bootloader、main 和 ftab，使用对应板级分区地址及回读校验，不整片擦除。
2. 在 Windows 蓝牙设置中添加 `CodexQuota-HSPI`，输入屏幕显示的 6 位 PIN。
3. 启用 Windows 托盘中的“黄山派蓝牙同步”。之后无需手动重复配对。

只有 Windows 运行、可获得额度且蓝牙可达时才会同步；断线时保持 RAM 缓存和原来的真实确认时间。

## 真机诊断

UART 默认 1,000,000 baud。只读取应用自身的受限摘要：

```powershell
python .\huangshan-app\tools\serial_status.py --port COM4
```

`CQ_STATUS` 中 state 为 0=暂无数据、1=已同步、2=缓存、3=离线缓存；five/weekly 的 254 表示套餐不适用、255 表示未知。ble 为 0=等待上电、1=收到上电、2=注册服务、3=广播已启动、4=已连接、9=广播或注册失败。

此命令只筛选额度摘要，不保存完整 UART 日志。应用不记录 PIN。原固件备份保留在本地忽略目录 `out/huangshan-original`，不加入 Git 或发布包。当前真机结论以 [current-status](../docs/current-status.md) 为准。

## 字体与真机状态

界面固定使用 AMOLED 深色配色，沿用 CodexQuota 状态色。18/22px 文字和60px数字字体均来自 SDK 内的 OFL Noto Sans SC；100% 使用紧凑数字档。新增文案后可分别用 --size 18、--size 22 和 --size 60 --digits 重新生成。

2026-10-14 已通过 COM4 覆盖刷写及回读校验，已有 Windows 系统配对自动恢复；用户确认仅周额度套餐的单圈页面和英文重置时间显示正常。双圈选择逻辑及颜色阈值通过自动验证，当前账号不提供5小时窗口，双圈触摸尚未完成对应真实数据的设备验证。

Windows 当前运行 out/huangshan-candidate/windows/CodexQuota.exe 本地便携候选，不是安装包；需要手动启动时运行 windows-native/scripts/run-huangshan.ps1。未修改原安装和原来的登录启动配置。

## 按键切换布局

短按 KEY2 功能键（PA43）在“同心环”和“环形图＋数据行”之间切换。同心布局的重置时间位于环内；数据行布局在环下列出明细。两种样式都根据套餐自动显示单圈或双圈，双圈时支持触摸切换中央主额度。

开机默认同心布局，选择在当前运行期间保持，额度同步不会重置布局。按键由 SDK button library 消抖，只处理 CLICKED；事件进入主线程邮箱，LVGL 操作始终在界面线程执行。电源键不绑定此功能。

2026-10-14 用户已确认 KEY2 连续两次切换均正常。板端状态摘要增加 layout（0=同心，1=数据行）和 clicks，只用于本机验证。

## OLED 保护与唤醒

页面背景为纯黑，正常亮度为驱动百分比30%。连接正常时保持显示，每60秒在中心及周围8个位置移动，最大位移±2px；整个内容和触摸区域一起移动。像素位移只是辅助保护，不能保证消除老化。

持续离线180秒后隐藏页面、将屏幕亮度设为0；短暂恢复连接会重新计时。没有收到有效BLE数据45秒也会视为断线，随后开始180秒计时。额度来源变成缓存但BLE仍正常收包时不会息屏。配对码显示期间保持亮屏。

触摸或短按KEY2可唤醒；唤醒动作只恢复屏幕，保留原布局，不触发额度选择或布局切换。离线唤醒后再显示180秒；重新收到有效BLE数据也会自动唤醒。第一次触摸一直忽略到手指释放，之后恢复普通触摸操作。

这是停止OLED发光，不是整板低功耗休眠：CO5300亮度寄存器设为0，同时画面全黑；触摸、BLE和主循环继续运行。当前板级LCD关电会拉低共用电源，因此不使用LCD POWEROFF，避免触摸无法唤醒。屏幕诊断CQ_SCREEN中的brightness是成功提交给驱动的设置值，不是光学亮度测量。

2026-10-14用户确认当前候选测试通过，包含此前待复核的OLED息屏与唤醒。当前真实账号仅有周窗口，双窗口触摸仍需对应套餐验证；源码推送不代表已发布新的安装包或固件Release。
