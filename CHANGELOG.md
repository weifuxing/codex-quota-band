# 更新日志

## 0.6.5 · 2026-10-14

- main改为Windows＋黄山派，旧Android/小米手环/手机同步完整工程迁移到codex/legacy-android-band。
- Windows移除手机配对、LAN/WSS、任务通知和Hook安装；保留额度采集、BLE、刷新、连接说明、开机启动与退出。安装升级时备份并移除本产品旧Hook。
- 黄山派支持按套餐单/双圈、KEY2布局切换、触摸选择、RAM断线缓存和自动重连，xhxxm/xdxxh倒计时与额度阈值颜色。
- 纯黑、30%亮度、每分钟±2px位移、断线3分钟息屏，以及触摸/KEY2/有效BLE数据唤醒。
- 发布Windows安装包、黄山派固件ZIP、刷写说明及SHA-256清单；真实双窗口触摸仍待对应套餐验证。

详细验证与边界见[Release说明](docs/releases/0.6.5.md)。历史版本记录见[旧工程分支](https://github.com/weifuxing/codex-quota-band/blob/codex/legacy-android-band/CHANGELOG.md)。
