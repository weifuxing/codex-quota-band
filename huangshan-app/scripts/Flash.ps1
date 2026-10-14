param(
    [Parameter(Mandatory=$true)][string]$Port,
    [Parameter(Mandatory=$true)][string]$SfTool
)
$ErrorActionPreference='Stop'
$tool=(Resolve-Path -LiteralPath $SfTool).Path
$manifest=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'manifest.json') -Raw | ConvertFrom-Json
foreach ($image in $manifest.images) {
    $file=Join-Path $PSScriptRoot $image.file
    if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $image.sha256) { throw "Checksum mismatch: $($image.file)" }
}
$backup=Join-Path $PSScriptRoot ('backup-'+[DateTime]::Now.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $backup | Out-Null
Push-Location $backup
try {
    & $tool -p $Port -c SF32LB52 -b 1000000 --connect-attempts 3 read_flash 'ftab.bin@0x12000000:0x8000' 'bootloader.bin@0x12010000:0x10000' 'main-1m.bin@0x12020000:0x100000'
    if ($LASTEXITCODE -ne 0) { throw 'Backup failed; firmware was not written' }
    foreach ($item in @(@('ftab.bin',32768),@('bootloader.bin',65536),@('main-1m.bin',1048576))) {
        if ((Get-Item -LiteralPath $item[0]).Length -ne $item[1]) { throw 'Backup length mismatch; firmware was not written' }
    }
} finally { Pop-Location }
Push-Location $PSScriptRoot
try {
    & $tool -p $Port -c SF32LB52 -b 1000000 --connect-attempts 3 write_flash --verify 'ftab.bin@0x12000000' 'bootloader.bin@0x12010000' 'main.bin@0x12020000'
    if ($LASTEXITCODE -ne 0) { throw "Flash failed; original regions are in $backup" }
} finally { Pop-Location }
Write-Host "Verified firmware written. Keep original backup: $backup"
