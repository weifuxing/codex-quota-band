param(
    [string]$Toolchain = 'stable-x86_64-pc-windows-msvc',
    [switch]$UseMirror
)
$ErrorActionPreference = 'Stop'
$project = Split-Path -Parent $PSScriptRoot
$repo = Split-Path -Parent $project
$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
$target = Join-Path $project 'target-huangshan'
$bundle = Join-Path $repo 'out\huangshan-candidate\windows'
$previousFlags = $env:RUSTFLAGS
$arguments = @("+$Toolchain")
if ($UseMirror) {
    $arguments += @('--config', 'source.crates-io.replace-with="rsproxy"', '--config', 'source.rsproxy.registry="sparse+https://rsproxy.cn/index/"')
}
Push-Location $project
try {
    $env:RUSTFLAGS = '-C target-feature=+crt-static'
    & $cargo @arguments build --release --bin codex_quota_windows --target-dir $target
    if ($LASTEXITCODE -ne 0) { throw 'Windows BLE candidate build failed' }
    New-Item -ItemType Directory -Force -Path $bundle | Out-Null
    Copy-Item -LiteralPath (Join-Path $target 'release\codex_quota_windows.exe') -Destination (Join-Path $bundle 'CodexQuota.exe')
    & (Join-Path $PSScriptRoot 'test-portable-bundle.ps1') -BundleDirectory $bundle
    if ($LASTEXITCODE -ne 0) { throw 'Windows BLE candidate smoke test failed' }
    Get-FileHash -LiteralPath (Join-Path $bundle 'CodexQuota.exe') -Algorithm SHA256
} finally {
    $env:RUSTFLAGS = $previousFlags
    Pop-Location
}
