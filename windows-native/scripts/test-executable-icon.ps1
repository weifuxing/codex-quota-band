param(
    [Parameter(Mandatory = $true)]
    [string]$Executable
)

$ErrorActionPreference = "Stop"
$resolvedExecutable = (Resolve-Path -LiteralPath $Executable).Path
$readobj = Get-Command llvm-readobj -ErrorAction SilentlyContinue
if (-not $readobj) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class CodexQuotaIconResources {
    private delegate bool ResourceName(IntPtr module, IntPtr type, IntPtr name, IntPtr param);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    private static extern IntPtr LoadLibraryEx(string file, IntPtr reserved, uint flags);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode)]
    private static extern bool EnumResourceNames(IntPtr module, IntPtr type, ResourceName callback, IntPtr param);
    [DllImport("kernel32.dll")]
    private static extern bool FreeLibrary(IntPtr module);
    public static bool HasIcons(string file) {
        IntPtr module = LoadLibraryEx(file, IntPtr.Zero, 0x22);
        if (module == IntPtr.Zero) return false;
        try {
            foreach (int type in new int[] {3, 14}) {
                bool found = false;
                ResourceName callback = (m, t, n, p) => { found = true; return false; };
                EnumResourceNames(module, new IntPtr(type), callback, IntPtr.Zero);
                if (!found) return false;
            }
            return true;
        } finally { FreeLibrary(module); }
    }
}
'@
    if (-not [CodexQuotaIconResources]::HasIcons($resolvedExecutable)) {
        throw "Windows executable does not contain ICON and GROUP_ICON resources: $resolvedExecutable"
    }
    Write-Host "Embedded Windows application icon verified: $resolvedExecutable"
    return
}
$resources = & $readobj.Source --coff-resources $resolvedExecutable 2>&1 | Out-String
if ($LASTEXITCODE -ne 0) {
    throw "llvm-readobj failed while checking executable resources"
}

if ($resources -notmatch '(?m)^\s*Type:\s*ICON\s+' -or
    $resources -notmatch '(?m)^\s*Type:\s*GROUP_ICON\s+') {
    throw "Windows executable does not contain ICON and GROUP_ICON resources: $resolvedExecutable"
}

Write-Host "Embedded Windows application icon verified: $resolvedExecutable"
