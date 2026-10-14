# 构建与验证 · 0.6.5

2026-10-14：main为Windows＋黄山派；旧Android/手环工程及历史验证记录位于codex/legacy-android-band。固件沿用已验收字节，Windows重新构建。

| 产物 | 字节 | SHA-256 |
| --- | --- | --- |
| CodexQuota-Setup-0.6.5.exe | 2090955 | 5D87E08A182FE15E3921E484E22734A932A3195849B1A2EF19B2D9D1231A4D5D |
| 安装包内CodexQuota.exe | 4486144 | 1777334DDAAABF7E11E600188540B0853786027B7909A430F0F79F6B7C7F5BA4 |
| CodexQuota-Huangshan-0.6.5.zip | 568498 | 4F160B8BE1C653EE8D8F957A76EE66A70E70CE6E02D2C55A00D56C0859AF75A9 |
| main.bin | 827704 | CC9CE2DF1EAB78C71C5AF0AC77D0C43FB6502C079142A43C830F92804B122D69 |
| ftab.bin | 11288 | 3145CB3DD6E64DEC8A2B137BADF280E542E1873A83610FAC97CAA4716536AA47 |
| bootloader.bin | 58544 | 9A79DF767876E9561EA60FA0BB9BD4123397A28E77A61928B225826BE03CA233 |

Windows Rust30项（额度/套餐17、迁移3、BLE10）通过，无编译警告；板端C状态、缓存、布局、倒计时、颜色、OLED计时和位移测试通过。图标资源检查、最小PATH独立启动通过。7-Zip从安装包提取EXE，其SHA-256与构建文件一致。NSIS在正常主机环境打包；受限环境32位工具无法启动，已在主机环境完成。

实际运行新便携候选：菜单为黄山派状态、数据新鲜度、两项额度、BLE开关、刷新、连接说明、开机启动和退出。进程入站监听数为0。没有手机配对与手环入口。原安装仍在，故不运行会覆盖现有安装的隔离安装/卸载脚本；原安装与登录启动指向未替换。新版便携候选已启动。

固件此前通过COM4回读校验及用户真机验收。配对、单圈、KEY2布局和OLED保护已确认；账号不含5小时窗口，真实双窗口触摸待对应套餐验证。Flash.ps1做PowerShell语法检查，包内镜像与已验收散列相符；本次没有重复刷写或测量面板寿命/电流。

发布说明见releases/0.6.5.md；附件含SHA256SUMS.txt。原设备备份、账号、配对数据、SDK、构建日志不加入公开源码或Release。
