param(
    [Parameter(Mandatory)][string]$Executable,
    [Parameter(Mandatory)][string]$Project,
    [Parameter(Mandatory)][string]$Output
)

$ErrorActionPreference = 'Stop'
$directory = [System.IO.Path]::GetFullPath($Output)
$binary = [System.IO.Path]::GetFullPath($Executable)
$document = [System.IO.Path]::GetFullPath($Project)
New-Item -ItemType Directory -Force -Path $directory | Out-Null
if (!(Test-Path -LiteralPath $binary) -or !(Test-Path -LiteralPath $document)) {
    throw 'The built editor and saved project must exist before desktop capture.'
}

$start = [System.Diagnostics.ProcessStartInfo]::new($binary)
$start.ArgumentList.Add($document)
$start.UseShellExecute = $false
$start.RedirectStandardOutput = $false
$start.RedirectStandardError = $false
$process = [System.Diagnostics.Process]::Start($start)
if ($null -eq $process) { throw 'Windows did not start the editor process.' }

try {
    $handle = [IntPtr]::Zero
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        Start-Sleep -Seconds 1
        $process.Refresh()
        if ($process.HasExited) { throw "Editor exited during startup with code $($process.ExitCode)." }
        $handle = $process.MainWindowHandle
        if ($handle -ne [IntPtr]::Zero) { break }
    }

    $status = [ordered]@{
        platform = 'windows-2025'
        project = 'synthetic.ecproj'
        process_running = !$process.HasExited
        main_window_handle = $handle.ToInt64()
        capture = 'unavailable'
    }
    if ($handle -eq [IntPtr]::Zero) {
        Write-Warning 'The hosted runner did not expose a desktop window; no GUI screenshot is claimed.'
        $status | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $directory 'desktop-status.json')
        return
    }

    Add-Type -AssemblyName System.Drawing
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class StrikeWindowBounds {
    [StructLayout(LayoutKind.Sequential)]
    public struct Rect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr handle, out Rect rectangle);
}
'@
    $rect = New-Object StrikeWindowBounds+Rect
    if (![StrikeWindowBounds]::GetWindowRect($handle, [ref]$rect)) { throw 'Could not read the native editor window bounds.' }
    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -lt 400 -or $height -lt 300) { throw "Editor window is unexpectedly small: ${width}x${height}." }
    $bitmap = [System.Drawing.Bitmap]::new($width, $height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
        $bitmap.Save((Join-Path $directory 'editor.png'), [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
    $status['capture'] = 'captured'
    $status['window_width'] = $width
    $status['window_height'] = $height
    $status | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $directory 'desktop-status.json')
}
finally {
    if (!$process.HasExited) { $process.Kill() }
    $process.Dispose()
}
