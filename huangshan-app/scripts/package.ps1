param([string]$Version='0.6.5')
$ErrorActionPreference='Stop'
$appRoot=Split-Path -Parent $PSScriptRoot
$repoRoot=Split-Path -Parent $appRoot
$build=Join-Path $appRoot 'project\build_sf32lb52-lchspi-ulp_hcpu'
$release=Join-Path $repoRoot "out\release-$Version"
$bundle=Join-Path $release "CodexQuota-Huangshan-$Version"
if (Test-Path -LiteralPath $bundle) { throw "Package directory already exists: $bundle" }
New-Item -ItemType Directory -Force -Path $bundle | Out-Null
$images=@()
foreach ($entry in @(@('main.bin','main.bin','0x12020000'),@('ftab\ftab.bin','ftab.bin','0x12000000'),@('bootloader\bootloader.bin','bootloader.bin','0x12010000'))) {
    $file=Join-Path $bundle $entry[1]
    Copy-Item -LiteralPath (Join-Path $build $entry[0]) -Destination $file
    $images+=@{file=$entry[1];address=$entry[2];bytes=(Get-Item -LiteralPath $file).Length;sha256=(Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash}
}
@{version=$Version;board='sf32lb52-lchspi-ulp';screen='390x450 AMOLED';bleProtocol=2;images=$images} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $bundle 'manifest.json') -Encoding utf8
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'Flash.ps1') -Destination $bundle
Copy-Item -LiteralPath (Join-Path $appRoot 'FIRMWARE.md') -Destination (Join-Path $bundle 'README.md')
Copy-Item -LiteralPath (Join-Path $repoRoot 'LICENSE') -Destination $bundle
Copy-Item -LiteralPath (Join-Path $appRoot 'fonts\OFL.txt') -Destination $bundle
$zip=Join-Path $release "CodexQuota-Huangshan-$Version.zip"
Compress-Archive -LiteralPath $bundle -DestinationPath $zip
Get-FileHash -LiteralPath $zip -Algorithm SHA256
