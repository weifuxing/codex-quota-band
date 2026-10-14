param([switch]$StopOnly)
$ErrorActionPreference='Stop'
$repoRoot=Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$candidate=Join-Path $repoRoot 'out\huangshan-candidate\windows\CodexQuota.exe'
if (-not $StopOnly -and -not (Test-Path -LiteralPath $candidate)) { throw 'Missing Huangshan candidate' }
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class HuangshanCandidateWindow {
    [DllImport("user32.dll",CharSet=CharSet.Unicode)]
    public static extern IntPtr FindWindow(string name,string title);
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr window,out uint processId);
    [DllImport("user32.dll")]
    public static extern bool PostMessage(IntPtr window,uint message,UIntPtr wp,IntPtr lp);
}
'@
$window=[HuangshanCandidateWindow]::FindWindow('CodexQuotaTrayWindow','Codex额度')
if ($window -ne [IntPtr]::Zero) {
    $windowProcessId=0u
    [void][HuangshanCandidateWindow]::GetWindowThreadProcessId($window,[ref]$windowProcessId)
    $closingProcess=Get-Process -Id $windowProcessId
    [void][HuangshanCandidateWindow]::PostMessage($window,0x0010,[UIntPtr]::Zero,[IntPtr]::Zero)
    $deadline=[DateTime]::UtcNow.AddSeconds(8)
    while ([HuangshanCandidateWindow]::FindWindow('CodexQuotaTrayWindow','Codex额度') -ne [IntPtr]::Zero) {
        if ([DateTime]::UtcNow -ge $deadline) { throw 'Existing CodexQuota did not close; no forced termination performed' }
        Start-Sleep -Milliseconds 100
    }
    if (-not $closingProcess.WaitForExit(8000)) { throw 'CodexQuota background shutdown timed out; no forced termination performed' }
}
if ($StopOnly) {
    'CANDIDATE_STOPPED'
} else {
    $process=Start-Process -FilePath $candidate -ArgumentList '--huangshan-ble' -WorkingDirectory (Split-Path -Parent $candidate) -PassThru -WindowStyle Hidden
    Start-Sleep -Seconds 2
    if ($process.HasExited) { throw 'Candidate exited; check the existing instance or startup state' }
    'CANDIDATE_STARTED'
}
