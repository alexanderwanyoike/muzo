param(
    [Parameter(Mandatory=$true)][string] $Binary,
    [Parameter(Mandatory=$true)][string] $Screenshots
)

$ErrorActionPreference = 'Stop'
$executable = (Resolve-Path -LiteralPath $Binary).Path
$windowTimeoutSeconds = 12
$windowPollMilliseconds = 100
$redrawMilliseconds = 350
$exitTimeoutMilliseconds = 2500
$miniWidth = 420
$miniHeight = 560
$queueWidth = 740
$captionCenterY = 16
$closeInset = 16
$maximizeInset = 48
$minimizeInset = 80
$pointerSettleMilliseconds = 80
Add-Type -AssemblyName System.Windows.Forms
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class MuzoWindows
{
    public const int DpiAwarenessContextPerMonitorAwareV2 = -4;
    public const uint DwmExtendedFrameBounds = 9;
    public const int SwMaximize = 3;
    public const uint SwpNoSize = 0x0001;
    public const uint SwpNoZOrder = 0x0004;
    public const uint SwpNoActivate = 0x0010;
    public const uint LeftButtonDown = 0x0002;
    public const uint LeftButtonUp = 0x0004;

    [StructLayout(LayoutKind.Sequential)]
    public struct Rect { public int Left, Top, Right, Bottom; }

    [StructLayout(LayoutKind.Sequential)]
    public struct Point { public int X, Y; }

    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr window, out Rect rect);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window, out Rect rect);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr window, int command);
    [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr window);
    [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern bool IsZoomed(IntPtr window);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr window, ref Point point);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(
        uint flags, uint dx, uint dy, uint data, UIntPtr extraInfo);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(
        IntPtr window, IntPtr insertAfter, int x, int y, int width, int height, uint flags);
    [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(
        IntPtr window, uint attribute, out Rect rect, uint size);
}
'@

function Test-ClientSize([IntPtr] $Window, [int] $Width, [int] $Height) {
    $client = [MuzoWindows+Rect]::new()
    $scale = [MuzoWindows]::GetDpiForWindow($Window) / 96.0
    if ($scale -eq 0 -or ![MuzoWindows]::GetClientRect($Window, [ref] $client)) { return $false }
    return [Math]::Abs($client.Right / $scale - $Width) -le 1 -and
        [Math]::Abs($client.Bottom / $scale - $Height) -le 1
}

function Wait-MuzoWindow([string] $Stage, [scriptblock] $Match, [switch] $Activate) {
    $timer = [Diagnostics.Stopwatch]::StartNew()
    while ($timer.Elapsed.TotalSeconds -lt $windowTimeoutSeconds) {
        if ($application.HasExited) { throw "Muzo exited while waiting for $Stage with code $($application.ExitCode)." }
        $window = [MuzoWindows]::GetForegroundWindow()
        $owner = [uint32] 0
        [void] [MuzoWindows]::GetWindowThreadProcessId($window, [ref] $owner)
        if ($owner -eq $application.Id -and (& $Match $window)) { return $window }
        if ($Activate) {
            $application.Refresh()
            $handle = $application.MainWindowHandle
            if ($handle -ne [IntPtr]::Zero) { [void] [MuzoWindows]::SetForegroundWindow($handle) }
        }
        Start-Sleep -Milliseconds $windowPollMilliseconds
    }
    $details = "Foreground window $window belongs to process $owner."
    $client = [MuzoWindows+Rect]::new()
    if ($owner -eq $application.Id -and [MuzoWindows]::GetClientRect($window, [ref] $client)) {
        $details += " Its client area is $($client.Right) by $($client.Bottom) pixels."
        $details += " Maximized $([MuzoWindows]::IsZoomed($window)), minimized $([MuzoWindows]::IsIconic($window))."
    }
    throw "Timed out waiting for $Stage. $details"
}

function Send-MuzoKeys([IntPtr] $Window, [string] $Keys) {
    if ([MuzoWindows]::GetForegroundWindow() -ne $Window) { throw 'Muzo lost focus before sending keys.' }
    [Windows.Forms.SendKeys]::SendWait($Keys)
}

function Click-MuzoCaption([IntPtr] $Window, [int] $Inset) {
    if ([MuzoWindows]::GetForegroundWindow() -ne $Window) { throw 'Muzo lost focus before clicking its title bar.' }
    $client = [MuzoWindows+Rect]::new()
    if (![MuzoWindows]::GetClientRect($Window, [ref] $client)) { throw 'Could not read the Muzo client area.' }
    $scale = [MuzoWindows]::GetDpiForWindow($Window) / 96.0
    $point = [MuzoWindows+Point]::new()
    $point.X = $client.Right - [int] [Math]::Round($Inset * $scale)
    $point.Y = [int] [Math]::Round($captionCenterY * $scale)
    if (![MuzoWindows]::ClientToScreen($Window, [ref] $point) -or ![MuzoWindows]::SetCursorPos($point.X, $point.Y)) {
        throw 'Could not position the pointer on the Muzo title bar.'
    }
    Start-Sleep -Milliseconds $pointerSettleMilliseconds
    [MuzoWindows]::mouse_event([MuzoWindows]::LeftButtonDown, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds $pointerSettleMilliseconds
    [MuzoWindows]::mouse_event([MuzoWindows]::LeftButtonUp, 0, 0, 0, [UIntPtr]::Zero)
}

function Get-WindowBounds([IntPtr] $Window) {
    $rect = [MuzoWindows+Rect]::new()
    if (![MuzoWindows]::GetWindowRect($Window, [ref] $rect)) { throw 'Could not read the Muzo window bounds.' }
    return [Drawing.Rectangle]::FromLTRB($rect.Left, $rect.Top, $rect.Right, $rect.Bottom)
}

function Save-MuzoWindow([IntPtr] $Window, [string] $Name) {
    Start-Sleep -Milliseconds $redrawMilliseconds
    if ([MuzoWindows]::GetForegroundWindow() -ne $Window) { throw 'Muzo lost focus before taking a screenshot.' }
    $frame = [MuzoWindows+Rect]::new()
    $frameSize = [Runtime.InteropServices.Marshal]::SizeOf($frame)
    if ([MuzoWindows]::DwmGetWindowAttribute($Window, [MuzoWindows]::DwmExtendedFrameBounds, [ref] $frame, $frameSize) -lt 0) {
        throw 'Could not read the visible bounds of the Muzo window.'
    }
    $visible = [Drawing.Rectangle]::FromLTRB($frame.Left, $frame.Top, $frame.Right, $frame.Bottom)
    $bounds = [Drawing.Rectangle]::Intersect(
        $visible, [Windows.Forms.Screen]::FromHandle($Window).Bounds)
    $bitmap = [Drawing.Bitmap]::new($bounds.Width, $bounds.Height)
    try {
        $graphics = [Drawing.Graphics]::FromImage($bitmap)
        try {
            $graphics.CopyFromScreen($bounds.Location, [Drawing.Point]::Empty, $bounds.Size)
        } finally {
            $graphics.Dispose()
        }
        $bitmap.Save((Join-Path $Screenshots "$Name.png"), [Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $bitmap.Dispose()
    }
}

function Stop-MuzoProcess([Diagnostics.Process] $Process) {
    if (!$Process) { return }
    try {
        if (!$Process.HasExited) {
            [void] $Process.CloseMainWindow()
            if (!$Process.WaitForExit($exitTimeoutMilliseconds)) {
                Stop-Process -InputObject $Process -ErrorAction SilentlyContinue
                if (!$Process.WaitForExit($exitTimeoutMilliseconds)) { throw 'The test Muzo process did not stop.' }
            }
        }
    } catch [InvalidOperationException] {
        if (!$Process.HasExited) { throw }
    } finally {
        $Process.Dispose()
    }
}

$data = [IO.Directory]::CreateTempSubdirectory('muzo-mini-')
$application = $null
$previousDpi = [IntPtr]::Zero
$testFailed = $false
try {
    $previousDpi = [MuzoWindows]::SetThreadDpiAwarenessContext([IntPtr] [MuzoWindows]::DpiAwarenessContextPerMonitorAwareV2)
    if ($previousDpi -eq [IntPtr]::Zero) { throw 'Could not set the capture thread DPI awareness.' }
    New-Item -ItemType Directory -Force -Path $Screenshots | Out-Null
    $recording = Join-Path $data.FullName 'Local track.mp3'
    Copy-Item (Join-Path $PSScriptRoot '../fixtures/tone.mp3') $recording
    $application = Start-Process -FilePath $executable -ArgumentList ('"{0}"' -f $recording) -PassThru `
        -WorkingDirectory $data.FullName -Environment @{ MUZO_DATA_DIR = $data.FullName }
    $library = Wait-MuzoWindow 'the library' { $true } -Activate
    [void] [MuzoWindows]::ShowWindow($library, [MuzoWindows]::SwMaximize)
    $library = Wait-MuzoWindow 'the maximized library' { param($window) $window -eq $library -and [MuzoWindows]::IsZoomed($window) }
    $libraryBounds = Get-WindowBounds $library
    Save-MuzoWindow $library 'windows-library'

    Send-MuzoKeys $library '^m'
    $mini = Wait-MuzoWindow 'the compact player' {
        param($window)
        $window -ne $library -and (Test-ClientSize $window $miniWidth $miniHeight) -and [MuzoWindows]::IsIconic($library)
    }
    $area = [Windows.Forms.Screen]::FromHandle($mini).WorkingArea
    $moveOnly = [MuzoWindows]::SwpNoSize -bor [MuzoWindows]::SwpNoZOrder -bor [MuzoWindows]::SwpNoActivate
    if (![MuzoWindows]::SetWindowPos($mini, [IntPtr]::Zero, $area.Left, $area.Top, 0, 0, $moveOnly)) {
        throw 'Could not position the mini player for capture.'
    }
    Save-MuzoWindow $mini 'windows-mini'

    Click-MuzoCaption $mini $maximizeInset
    $maximized = Wait-MuzoWindow 'the maximized mini player' {
        param($window)
        $window -eq $mini -and [MuzoWindows]::IsZoomed($window)
    }
    Click-MuzoCaption $maximized $maximizeInset
    $null = Wait-MuzoWindow 'the restored mini player' {
        param($window)
        $window -eq $mini -and (Test-ClientSize $window $miniWidth $miniHeight)
    }

    Send-MuzoKeys $mini '^q'
    $expanded = Wait-MuzoWindow 'the expanded mini player queue' {
        param($window)
        $window -eq $mini -and (Test-ClientSize $window $queueWidth $miniHeight)
    }
    Save-MuzoWindow $expanded 'windows-mini-queue'

    Send-MuzoKeys $mini '^q'
    $collapsed = Wait-MuzoWindow 'the collapsed mini player queue' {
        param($window)
        $window -eq $mini -and (Test-ClientSize $window $miniWidth $miniHeight)
    }
    Save-MuzoWindow $collapsed 'windows-mini-collapsed'
    Send-MuzoKeys $mini '{ESC}'
    $restored = Wait-MuzoWindow 'the restored library' {
        param($window)
        $window -eq $library -and [MuzoWindows]::IsZoomed($window) -and
            (Get-WindowBounds $window).Equals($libraryBounds) -and ![MuzoWindows]::IsWindow($mini)
    }
    Save-MuzoWindow $restored 'windows-library-restored'

    Send-MuzoKeys $restored '^m'
    $closeTarget = Wait-MuzoWindow 'the reopened mini player' {
        param($window)
        $window -ne $library -and (Test-ClientSize $window $miniWidth $miniHeight)
    }
    Click-MuzoCaption $closeTarget $closeInset
    $restored = Wait-MuzoWindow 'the library after closing the mini player' {
        param($window)
        $window -eq $library -and [MuzoWindows]::IsZoomed($window) -and ![MuzoWindows]::IsWindow($closeTarget)
    }

    Click-MuzoCaption $restored $minimizeInset
    $timer = [Diagnostics.Stopwatch]::StartNew()
    while (![MuzoWindows]::IsIconic($library) -and $timer.Elapsed.TotalSeconds -lt $windowTimeoutSeconds) {
        Start-Sleep -Milliseconds $windowPollMilliseconds
    }
    if (![MuzoWindows]::IsIconic($library)) { throw 'The library stayed visible after clicking Minimize.' }
    Write-Host 'Verified the Windows title bar controls, mini player queue and maximized library restoration.'
} catch {
    $testFailed = $true
    throw
} finally {
    $cleanupErrors = @(
        try { Stop-MuzoProcess $application } catch { $_ }
        try {
            if ($previousDpi -ne [IntPtr]::Zero -and
                [MuzoWindows]::SetThreadDpiAwarenessContext($previousDpi) -eq [IntPtr]::Zero) {
                throw 'Could not restore the capture thread DPI awareness.'
            }
        } catch { $_ }
        try { $data.Delete($true) } catch { $_ }
    )
    foreach ($cleanupError in $cleanupErrors) {
        Write-Warning $cleanupError.Exception.Message -WarningAction Continue
    }
    if (!$testFailed -and $cleanupErrors.Count) { throw 'Could not clean up after the Muzo smoke test.' }
}
