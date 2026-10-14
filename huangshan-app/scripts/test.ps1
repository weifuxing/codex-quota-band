$ErrorActionPreference = 'Stop'
$appRoot = Split-Path -Parent $PSScriptRoot
$repoRoot = Split-Path -Parent $appRoot
$out = Join-Path $repoRoot 'out\huangshan-tests'
New-Item -ItemType Directory -Force -Path $out | Out-Null
$programFilesX86 = [Environment]::GetFolderPath('ProgramFilesX86')
$vc = Join-Path $programFilesX86 'Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC'
$version = Get-ChildItem -LiteralPath $vc -Directory | Sort-Object Name -Descending | Select-Object -First 1
$vc = $version.FullName
$kits = Join-Path $programFilesX86 'Windows Kits\10'
$kitVersion = Get-ChildItem -LiteralPath (Join-Path $kits 'Include') -Directory | Sort-Object Name -Descending | Select-Object -First 1
$kit = $kitVersion.Name
$exe = Join-Path $out 'quota_state_test.exe'
Push-Location $out
try {
    & (Join-Path $vc 'bin\Hostx64\x64\cl.exe') /nologo /W4 /WX /std:c11 /utf-8 "/I$vc\include" "/I$kits\Include\$kit\ucrt" "/I$appRoot\src" "/Fe:$exe" "/Fo:$out\" (Join-Path $appRoot 'tests\quota_state_test.c') (Join-Path $appRoot 'src\quota_state.c') /link "/LIBPATH:$vc\lib\x64" "/LIBPATH:$kits\Lib\$kit\ucrt\x64" "/LIBPATH:$kits\Lib\$kit\um\x64"
    if ($LASTEXITCODE -ne 0) { throw 'C state tests failed to compile' }
    & $exe
    if ($LASTEXITCODE -ne 0) { throw 'C state tests failed' }
} finally { Pop-Location }
