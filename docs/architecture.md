# 架构

Windows QuotaCollector每45秒低频确认官方额度，每5秒读取本地额度缓存。托盘共享QuotaSnapshot给独立BleWorker，后者只连接已配对的CodexQuota-HSPI，经认证加密GATT每10秒写入20字节摘要。重连退避2/4/8/16/30秒。

Windows没有入站监听、LAN/WSS、手机协议、任务Hook事件处理或通知服务。旧本地缓存模型中的reset_inventory只为兼容既有额度缓存，不通过BLE发送重置卡信息。安装器migration模块仅用于移除旧本产品Hook。

黄山派使用SiFli SDK、RT-Thread、LVGL；BLE回调更新受保护状态，主线程渲染，KEY2/触摸事件通过邮箱传递，避免跨线程调用LVGL。失联保留本次开机RAM缓存与确认时间，窗口过期显示未知。

协议见../contract/ble-quota-v2.md。OLED归零亮度时保留共用触摸电源；不是整板休眠。
