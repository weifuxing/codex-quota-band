# Codex额度 Windows

仅提供黄山派蓝牙同步、额度采集/缓存、刷新、连接说明、开机启动和退出。没有Android/手环、手机配对、LAN/WSS或任务Hook功能。

默认开启BLE，托盘可持久化关闭。每45秒低频确认官方额度，每5秒读取本地缓存，每10秒BLE发送，断线自动退避重连。--ble-probe为受限只读诊断，--smoke-test验证图标独立启动，--migrate-legacy-setup只供安装器移除本产品旧Hook并备份。

构建：cargo test --workspace；scripts/build-installer.ps1。详见../docs/development-guide.md。历史完整Windows手机/手环服务保留在codex/legacy-android-band。
