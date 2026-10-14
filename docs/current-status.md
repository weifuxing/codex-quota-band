# 当前状态

更新时间：2026-10-14。Windows＋黄山派0.6.5已[正式发布](https://github.com/weifuxing/codex-quota-band/releases/tag/v0.6.5)。附件为Windows安装包、黄山派固件ZIP与SHA256SUMS.txt，远端大小及SHA-256与本地一致。

main仅维护Windows原生软件和黄山派固件；旧Android/手环/局域网工程已完整推送到codex/legacy-android-band（f25205e）。删除只涉及Git跟踪文件，忽略的本机SDK、构建缓存和旧产物仍保留在本地。

Windows已去除手机扫码配对、LAN/WSS服务器、发现广播、任务/通知同步、Hook安装入口及对应依赖；托盘保留额度、刷新、BLE开关、连接说明、启动和退出。安装器只迁移删除本产品旧Hook，修改前保留hooks.json备份，不影响其他Hook。旧命令行Hook调用直接退出，不读事件。

黄山派BLE v2、RAM断线缓存、重连、按套餐单/双圈、KEY2布局切换及OLED保护已完成此前本机验收。固件字节保持不变；账号仅有周窗口，真实双窗口触摸仍待对应套餐验证。

Windows裁剪后重新执行自动测试、安装包图标与独立启动检查；发布状态及散列见build-verification.md。原Windows安装、自启动和当前运行程序在本次打包阶段仍未替换。用户已授权从main发布Windows安装包和黄山派固件，不发布旧分支APK/RPK。
