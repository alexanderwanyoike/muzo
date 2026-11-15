param(
    [Parameter(Mandatory=$true)][ValidateSet('Confirm', 'Reopened')][string] $Journey,
    [int] $LegacyProcessId,
    [string] $Executable
)

$ErrorActionPreference = 'Stop'
$waitSeconds = 25
$pollMilliseconds = 250
$timer = [Diagnostics.Stopwatch]::StartNew()

Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class MuzoUpdateWindows
{
    private const uint GetText = 0x000D;
    private const uint ClickButton = 0x00F5;
    private const uint AbortIfHung = 0x0002;
    private const uint QueryTimeout = 250;

    private delegate bool Enumerate(IntPtr window, IntPtr context);

    [DllImport("user32.dll")] private static extern bool EnumWindows(Enumerate callback, IntPtr context);
    [DllImport("user32.dll")] private static extern bool EnumChildWindows(IntPtr parent, Enumerate callback, IntPtr context);
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetClassName(IntPtr window, StringBuilder buffer, int capacity);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern IntPtr SendMessageTimeout(IntPtr window, uint message,
        UIntPtr length, StringBuilder buffer, uint flags, uint timeout, out UIntPtr result);
    [DllImport("user32.dll")] private static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] private static extern bool PostMessage(IntPtr window, uint message, IntPtr wparam, IntPtr lparam);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);

    public static IntPtr[] Windows(int owner)
    {
        var windows = new List<IntPtr>();
        EnumWindows((window, context) => {
            uint process;
            GetWindowThreadProcessId(window, out process);
            if (process == (uint)owner && IsWindowVisible(window)) windows.Add(window);
            return true;
        }, IntPtr.Zero);
        return windows.ToArray();
    }

    public static IntPtr[] Children(IntPtr parent)
    {
        var children = new List<IntPtr>();
        EnumChildWindows(parent, (child, context) => {
            if (IsWindowVisible(child)) children.Add(child);
            return true;
        }, IntPtr.Zero);
        return children.ToArray();
    }

    public static string ClassName(IntPtr window)
    {
        var name = new StringBuilder(256);
        GetClassName(window, name, name.Capacity);
        return name.ToString();
    }

    public static string Text(IntPtr window)
    {
        var text = new StringBuilder(256);
        UIntPtr result;
        SendMessageTimeout(window, GetText, (UIntPtr)text.Capacity, text, AbortIfHung, QueryTimeout, out result);
        return text.ToString();
    }

    public static bool Click(IntPtr dialog, IntPtr button)
    {
        SetForegroundWindow(dialog);
        return PostMessage(button, ClickButton, IntPtr.Zero, IntPtr.Zero);
    }
}
'@

if ($Journey -eq 'Confirm') {
    while ($timer.Elapsed.TotalSeconds -lt $waitSeconds) {
        foreach ($window in [MuzoUpdateWindows]::Windows($LegacyProcessId)) {
            if ([MuzoUpdateWindows]::ClassName($window) -ne '#32770') { continue }
            foreach ($button in [MuzoUpdateWindows]::Children($window)) {
                if ([MuzoUpdateWindows]::ClassName($button) -ne 'Button') { continue }
                if ([MuzoUpdateWindows]::Text($button).Replace('&', '') -ne 'Restart to install') { continue }
                if (![MuzoUpdateWindows]::Click($window, $button)) { throw 'Could not click the restart confirmation.' }
                Write-Output 'Accepted the installed Electron restart dialog.'
                exit 0
            }
        }
        Start-Sleep -Milliseconds $pollMilliseconds
    }
    foreach ($window in [MuzoUpdateWindows]::Windows($LegacyProcessId)) {
        $class = [MuzoUpdateWindows]::ClassName($window)
        Write-Output "Visible window $class with title '$([MuzoUpdateWindows]::Text($window))'"
        if ($class -eq '#32770') {
            foreach ($child in [MuzoUpdateWindows]::Children($window)) {
                Write-Output "  $([MuzoUpdateWindows]::ClassName($child)) '$([MuzoUpdateWindows]::Text($child))'"
            }
        }
    }
    throw 'The installed Electron restart dialog could not be confirmed.'
}

$expected = (Resolve-Path -LiteralPath $Executable).Path
while ($timer.Elapsed.TotalSeconds -lt $waitSeconds) {
    $application = Get-Process -Name 'muzo' -ErrorAction SilentlyContinue | Where-Object {
        $_.Id -ne $LegacyProcessId -and $_.Path -eq $expected -and $_.MainWindowTitle -eq 'Muzo' -and
            [MuzoUpdateWindows]::IsWindowVisible($_.MainWindowHandle)
    } | Select-Object -First 1
    if ($application) {
        Write-Output 'The replacement executable opened the Muzo library window.'
        exit 0
    }
    Start-Sleep -Milliseconds $pollMilliseconds
}
throw 'The replacement executable did not reopen the Muzo window.'
