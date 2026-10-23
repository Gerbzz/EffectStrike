param(
    [Parameter(Mandatory)][string]$Executable,
    [Parameter(Mandatory)][string]$Project,
    [Parameter(Mandatory)][string]$Output,
    [ValidateRange(10, 300)][int]$TimeoutSeconds = 75
)

$ErrorActionPreference = 'Stop'
$directory = [System.IO.Path]::GetFullPath($Output)
$binary = [System.IO.Path]::GetFullPath($Executable)
$document = [System.IO.Path]::GetFullPath($Project)
New-Item -ItemType Directory -Force -Path $directory | Out-Null
if (!(Test-Path -LiteralPath $binary) -or !(Test-Path -LiteralPath $document)) {
    throw 'The built editor and saved project must exist before desktop capture.'
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

function Request-NativeScreenshot([int]$Port, [string]$Path) {
    $client = [System.Net.Sockets.TcpClient]::new()
    try {
        $client.ReceiveTimeout = 15000
        $client.SendTimeout = 3000
        $connect = $client.ConnectAsync('127.0.0.1', $Port)
        if (!$connect.Wait(3000)) { throw 'Native control channel connection timed out.' }
        $connect.GetAwaiter().GetResult()
        $stream = $client.GetStream()
        $request = @{ id = 1; method = 'ui.screenshot'; params = @{ path = $Path } } | ConvertTo-Json -Depth 4 -Compress
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($request + "`n")
        $stream.Write($bytes, 0, $bytes.Length)
        $reader = [System.IO.StreamReader]::new($stream)
        $line = $reader.ReadLine()
        if ($null -eq $line) { throw 'Native editor closed the screenshot control connection.' }
        $reply = $line | ConvertFrom-Json
        if ($reply.id -ne 1) { throw 'Native screenshot response has the wrong request id.' }
        return $reply
    }
    finally { $client.Dispose() }
}

function Measure-EditorPixels([System.Drawing.Bitmap]$Bitmap) {
    # This generated project has a large blue composition. A sized black window, title bar,
    # or isolated blue toolbar button cannot satisfy the content gate.
    $sampled = 0
    $lit = 0
    $blue = 0
    $colors = [System.Collections.Generic.HashSet[int]]::new()
    for ($y = 50; $y -lt $Bitmap.Height - 15; $y += 12) {
        for ($x = 15; $x -lt $Bitmap.Width - 15; $x += 12) {
            $color = $Bitmap.GetPixel($x, $y)
            $sampled++
            if ([Math]::Max($color.R, [Math]::Max($color.G, $color.B)) -ge 25) { $lit++ }
            if ($color.B -ge 80 -and $color.B - $color.R -ge 25 -and $color.B - $color.G -ge 25) { $blue++ }
            [void]$colors.Add($color.ToArgb())
        }
    }
    return [ordered]@{ sampled = $sampled; non_dark = $lit; blue_comp = $blue; distinct_colors = $colors.Count }
}

$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, 0)
$listener.Start()
$controlPort = $listener.LocalEndpoint.Port
$listener.Stop()
$start = [System.Diagnostics.ProcessStartInfo]::new($binary)
$start.ArgumentList.Add('--control')
$start.ArgumentList.Add([string]$controlPort)
$start.ArgumentList.Add($document)
$start.UseShellExecute = $false
$start.RedirectStandardOutput = $false
$start.RedirectStandardError = $false
$process = [System.Diagnostics.Process]::Start($start)
if ($null -eq $process) { throw 'Windows did not start the editor process.' }
$clock = [System.Diagnostics.Stopwatch]::StartNew()
$bitmap = $null
$status = [ordered]@{
    platform = 'windows-2025'
    project = [System.IO.Path]::GetFileName($document)
    process_running = $true
    main_window_handle = 0
    capture = 'unavailable'
    capture_method = 'native_viewport_screenshot'
    control_port = $controlPort
    timeout_seconds = $TimeoutSeconds
}

try {
    $rect = New-Object StrikeWindowBounds+Rect
    $nextReport = 15
    while ($clock.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
        $process.Refresh()
        if ($process.HasExited) {
            $status['process_running'] = $false
            $status['exit_code'] = $process.ExitCode
            break
        }
        $handle = $process.MainWindowHandle
        $status['main_window_handle'] = $handle.ToInt64()
        if ($handle -ne [IntPtr]::Zero -and [StrikeWindowBounds]::GetWindowRect($handle, [ref]$rect)) {
            $width = $rect.Right - $rect.Left
            $height = $rect.Bottom - $rect.Top
            $status['window_width'] = $width
            $status['window_height'] = $height
            if ($width -ge 400 -and $height -ge 300) {
                if (!$status.Contains('first_sized_at_seconds')) {
                    $status['first_sized_at_seconds'] = [Math]::Round($clock.Elapsed.TotalSeconds, 1)
                }
                if ($null -ne $bitmap) { $bitmap.Dispose(); $bitmap = $null }
                $nativePath = Join-Path $directory 'editor-native-candidate.png'
                try {
                    $reply = Request-NativeScreenshot $controlPort $nativePath
                }
                catch {
                    $status['last_control_error'] = $_.Exception.Message
                    Start-Sleep -Seconds 1
                    continue
                }
                $reply | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $directory 'native-screenshot-response.json')
                if ($reply.ok -ne $true -or !(Test-Path -LiteralPath $nativePath)) {
                    $status['last_control_error'] = [string]$reply.error
                    Start-Sleep -Seconds 1
                    continue
                }
                $process.Refresh()
                if ($process.HasExited) { throw 'Editor exited before native screenshot acceptance.' }
                $bitmap = [System.Drawing.Bitmap]::new($nativePath)
                if ($bitmap.Width -ne $reply.result.width -or $bitmap.Height -ne $reply.result.height -or $bitmap.Width -lt 400 -or $bitmap.Height -lt 300) { throw 'Native screenshot dimensions do not match the editor response.' }
                $status['native_width'] = $bitmap.Width
                $status['native_height'] = $bitmap.Height
                $pixels = Measure-EditorPixels $bitmap
                $status['pixels'] = $pixels
                if ($pixels.non_dark -ge 200 -and $pixels.blue_comp -ge 80 -and $pixels.distinct_colors -ge 8) {
                    Copy-Item -LiteralPath $nativePath -Destination (Join-Path $directory 'editor.png') -Force
                    $status['capture'] = 'content_verified'
                    $status['content_ready_at_seconds'] = [Math]::Round($clock.Elapsed.TotalSeconds, 1)
                    $status | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $directory 'desktop-status.json')
                    return
                }
            }
        }
        if ($clock.Elapsed.TotalSeconds -ge $nextReport) {
            $blueSamples = if ($status.Contains('pixels')) { $status['pixels'].blue_comp } else { 0 }
            Write-Host "Startup $([Math]::Round($clock.Elapsed.TotalSeconds, 1))s: window $($status['window_width'])x$($status['window_height']), blue composition samples $blueSamples"
            $nextReport += 15
        }
        Start-Sleep -Seconds 1
    }
    if ($null -ne $bitmap) {
        $bitmap.Save((Join-Path $directory 'editor-unready.png'), [System.Drawing.Imaging.ImageFormat]::Png)
    }
    $status['elapsed_seconds'] = [Math]::Round($clock.Elapsed.TotalSeconds, 1)
    $status | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $directory 'desktop-status.json')
    $blueSamples = if ($status.Contains('pixels')) { $status['pixels'].blue_comp } else { 0 }
    throw "Editor content was not ready after $($status.elapsed_seconds)s; window $($status.window_width)x$($status.window_height), blue samples $blueSamples. No readable GUI screenshot is claimed."
}
finally {
    if ($null -ne $bitmap) { $bitmap.Dispose() }
    if (!$process.HasExited) { $process.Kill() }
    $process.Dispose()
}
