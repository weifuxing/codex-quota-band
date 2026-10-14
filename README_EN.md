<p align="right"><a href="README.md">简体中文</a></p>

# Codex Quota · Huangshan Pi

Windows → authenticated, encrypted BLE → Huangshan Pi, displaying quota and reset times on the stock390×450 AMOLED without a phone.

Version **0.6.5**. The original Android, Xiaomi band, AstroBox and LAN-sync project is preserved on [codex/legacy-android-band](https://github.com/weifuxing/codex-quota-band/tree/codex/legacy-android-band).

Download the Windows installer `CodexQuota-Setup-0.6.5.exe`, firmware `CodexQuota-Huangshan-0.6.5.zip`, and `SHA256SUMS.txt` from [Releases](https://github.com/weifuxing/codex-quota-band/releases).

Install the Windows app, flash the matching board using the firmware package guide, pair `CodexQuota-HSPI` in Windows Bluetooth settings with the on-screen PIN, and keep the tray Bluetooth sync enabled. The software includes Refresh quota, Bluetooth sync and Startup controls. It has no phone pairing, LAN server, band bridge or task Hook feature.

## Huangshan Pi direct Bluetooth (experimental)

Windows sends quota summaries over authenticated, encrypted BLE to a paired `CodexQuota-HSPI`. No phone is required. The main branch contains only the Windows software and Huangshan firmware.

<table>
  <tr>
    <th align="center">Weekly only</th>
    <th align="center">Concentric rings</th>
    <th align="center">Rings with data rows</th>
  </tr>
  <tr>
    <td align="center"><img src="docs/huangshan-ui/weekly-concentric.png" alt="Huangshan Pi weekly-only ring with reset time inside" width="230"></td>
    <td align="center"><img src="docs/huangshan-ui/dual-concentric.png" alt="Five-hour outer ring and weekly inner ring with touch selection" width="230"></td>
    <td align="center"><img src="docs/huangshan-ui/dual-rows.png" alt="Two quota rings with separate percentage and reset-time rows" width="230"></td>
  </tr>
</table>

These are 390×450 interface previews with sample data, not device photographs. Actual values depend on the plan and sync results. KEY2 switches layouts. The weekly-only interface has been verified on the device; real dual-window touch behavior still needs a matching plan.

- Weekly-only plans show one ring; plans with both windows show two rings. Missing data and a window absent from the plan are handled separately.
- Remaining quota is red below 20%, yellow at 20–49%, and green at 50% or above. Reset times use `xhxxm` for five-hour windows and `xdxxh` for weekly windows.
- Short-press KEY2 to switch between concentric rings and rings with data rows. Touch selects the central quota when both windows are available. Boot defaults to concentric rings.
- Disconnections retain RAM-cached quota with its original confirmation time; Windows reconnects automatically. Power cycling requires another sync.
- OLED protection uses a black background, 30% brightness, and a shift of up to ±2px every minute. Three minutes continuously offline blanks the display; touch, KEY2, or a valid BLE packet wakes it. The first touch/key action only wakes the display. No valid packets for 45 seconds counts as a disconnected link, then the three-minute timer starts.

Blanking hides all content and sets brightness to zero while retaining the shared touch supply. The display circuitry and BLE keep running. A healthy BLE connection keeps the screen on even when the upstream quota is cached.

After installing the SiFli SDK environment, build from the repository root:

```powershell
.\huangshan-app\scripts\test.ps1
.\huangshan-app\scripts\build.ps1 -SdkRoot D:\UGit\OpenSiFli\sifli-sdk
.\windows-native\scripts\build-huangshan.ps1 -UseMirror
```

Back up affected flash regions and flash the board-specific bootloader, main and ftab without erasing bonds. Pair `CodexQuota-HSPI` in Windows Bluetooth settings using the six-digit PIN shown on the screen, then run `.\windows-native\scripts\run-huangshan.ps1`. The portable candidate preserves the previous installed app and login startup configuration.

The user has confirmed device testing of the current candidate. The test account only provides a weekly window, so real dual-window touch behavior remains unverified. See the [board guide](huangshan-app/README.md), [BLE contract](contract/ble-quota-v2.md), and [current status](docs/current-status.md).


## Privacy

Only quota summaries and reset/connection state are sent to the board. Local Codex access tokens stay in Windows process memory and are used only with official endpoints. No cloud relay, telemetry or automatic crash reporting. Community project, MIT code and OFL fonts; not an official OpenAI or SiFli product.
