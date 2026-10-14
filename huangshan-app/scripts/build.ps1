param(
    [string]$SdkRoot = "D:\UGit\OpenSiFli\sifli-sdk",
    [string]$Python = "$env:USERPROFILE\.sifli\python_env\sifli-sdk2.4_py3.13_env\Scripts\python.exe",
    [string]$ArmGccBin = "$env:USERPROFILE\.sifli\tools\arm-none-eabi-gcc\14.2.1\bin"
)
$ErrorActionPreference = "Stop"
$SdkRoot = (Resolve-Path -LiteralPath $SdkRoot).Path
$appRoot = Split-Path -Parent $PSScriptRoot
$env:SIFLI_SDK = $SdkRoot
$env:SIFLI_SDK_PATH = $SdkRoot
$env:RTT_ROOT = Join-Path $SdkRoot 'rtos\rtthread'
$env:RTT_EXEC_PATH = $ArmGccBin
$env:RTT_CC = 'gcc'
if (-not $env:PROCESSOR_ARCHITECTURE) { $env:PROCESSOR_ARCHITECTURE = 'AMD64' }
$repoRoot = Split-Path -Parent $appRoot
$localDependencies = Join-Path $repoRoot 'out\huangshan-python-deps'
$env:PYTHONPATH = $localDependencies + [IO.Path]::PathSeparator + (Join-Path $SdkRoot 'tools\build') + [IO.Path]::PathSeparator + $env:PYTHONPATH
$env:PATH = $ArmGccBin + [IO.Path]::PathSeparator + (Split-Path -Parent $Python) + [IO.Path]::PathSeparator + $env:PATH
Push-Location (Join-Path $appRoot 'project')
try {
    & $Python -m SCons --board=sf32lb52-lchspi-ulp -j8
    if ($LASTEXITCODE -ne 0) { throw "Firmware build failed: $LASTEXITCODE" }
} finally { Pop-Location }
