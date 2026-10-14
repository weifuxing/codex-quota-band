# Huangshan BLE Quota v2

本协议用于 Windows → 黄山派的本地额度摘要，不依赖 Android、小米运动健康或 AstroBox。它独立于现有 WSS 协议；本阶段不传输任务或可用重置卡信息。

## 连接与信任

- 黄山派为 BLE peripheral / GATT server，Windows 为 central / GATT client。
- 广播名称为 `CodexQuota-HSPI`，使用 general discoverable 模式和 1M PHY。
- 服务 UUID：`f984c790-37ca-4d8b-a140-58d6a1b25f10`。
- 额度特征 UUID：`f984c791-37ca-4d8b-a140-58d6a1b25f10`。
- 首次使用 Windows 系统配对，黄山派显示随机 PIN；要求加密和身份验证。广播名称与 UUID 仅用于发现，不能代替系统配对。
- Windows 只选择已配对且名称一致的设备；出现多个候选时停止选择，避免向不确定的板子发送数据。
- 特征仅允许 authenticated Write Request；Windows 使用 WriteWithResponse，并要求 EncryptionAndAuthenticationRequired。
- SDK 的 Bluetooth/HCI 原始跟踪关闭。应用诊断只输出状态、百分比、确认时间与接收计数，不输出地址、密钥或报文。
- 本机 Codex 访问令牌继续只用于 Windows 官方额度确认，不进入 BLE。

## 载荷

每次写入恰好 20 字节，不依赖 MTU 扩大、分包或 JSON。多字节整数均为 little-endian；时间为 UTC Unix 秒。

| 偏移 | 长度 | 字段 | 含义 |
| --- | --- | --- | --- |
| 0 | 1 | version | 固定为 2；未知版本拒绝 |
| 1 | 1 | freshness | 0=暂无额度，1=近期官方确认，2=缓存 |
| 2 | 1 | fiveHourRemaining | 0–100；254=套餐不适用，255=未知 |
| 3 | 1 | weeklyRemaining | 0–100；254=套餐不适用，255=未知 |
| 4 | 4 | sentAt | 发送时电脑时间，也是板端倒计时的时间锚点 |
| 8 | 4 | confirmedAt | 同一额度最近一次官方成功确认时间；0=未知 |
| 12 | 4 | fiveHourResetsAt | 5 小时重置时间；0=未知 |
| 16 | 4 | weeklyResetsAt | 周重置时间；0=未知 |

有效百分比必须来自 current 窗口且有尚未到期的有效重置时间。未知、异常和 pendingSync 窗口不猜测。源码中的标签、重置卡 ID、凭据及任务内容无法进入此固定结构。

## 时间与缓存

Windows 每 10 秒发送摘要；官方额度仍按现有 45 秒节拍确认。发送新包、心跳或重新连接均不能更新 confirmedAt。

板端使用最近的 sentAt 与本机单调运行时钟计算当前时间和缓存年龄，不要求先设置 RTC。confirmedAt 在两分钟内且发送方状态为 current 时才显示“已同步”。Windows 仍连接但官方确认变旧时显示“缓存”；连接断开或 45 秒没有有效摘要时显示“离线缓存”。未知确认时间不显示虚假的缓存年龄。

断线缓存仅保留在 RAM，设备重启后等待新快照。额度重置时间到期后该窗口显示 `--`，等待 Windows 获取新的真实数据，不推算恢复到 100%。

未知版本、长度异常、异常百分比、未来确认时间、不一致 freshness，以及不递增 sentAt 都拒绝，且不覆盖最后一份有效状态。电脑时钟大幅回拨暂不提供跨会话时钟校正。

## 重连

黄山派的广播服务在断线后自动恢复广播。Windows 在失败后释放 GATT/session/device 资源，按 2、4、8、16、30 秒退避重新发现和连接已配对设备；成功写入后恢复 10 秒节拍。不自动取消配对或重新生成密钥。

## 套餐布局与版本兼容

Windows 从完整、结构有效的官方额度窗口定义判断该窗口是否适用，不读取套餐名称或账号资料。未知格式和旧版缓存没有窗口事实时保持 unknown，不把采集失败当成不适用。窗口事实仅保留在本机额度缓存。main不再包含WSS服务。

254 与 255 必须区别处理：套餐只有周额度时显示单圈、无切换按钮；两项限制同时存在时显示双圈，触摸底部按钮切换中央数字及重置倒计时。暂时不可用或断线时保留此前的套餐布局。

5小时重置倒计时为 xhxxm，周重置倒计时为 xdxxh；后一项补零两位。两圈按各自剩余额度变色：低于20%红、20–49%黄、50%及以上绿，未知灰。缓存或离线不改变额度阈值色。

v2 仍为20字节，但 v1 板端会拒绝 v2 包。Windows 与黄山派必须一起升级；GATT UUID 和系统配对保持不变。
