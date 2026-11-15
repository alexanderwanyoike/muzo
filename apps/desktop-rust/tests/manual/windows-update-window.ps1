param(
    [Parameter(Mandatory=$true)][ValidateSet('Confirm', 'Reopened')][string] $Journey,
    [int] $LegacyProcessId,
    [string] $Executable
)

$ErrorActionPreference = 'Stop'
$waitSeconds = 25
$pollMilliseconds = 250
$timer = [Diagnostics.Stopwatch]::StartNew()

if ($Journey -eq 'Confirm') {
    Add-Type -AssemblyName UIAutomationClient
    Add-Type -AssemblyName UIAutomationTypes
    $root = [System.Windows.Automation.AutomationElement]::RootElement
    $processMatch = [System.Windows.Automation.PropertyCondition]::new(
        [System.Windows.Automation.AutomationElement]::ProcessIdProperty, $LegacyProcessId)
    $labelMatch = [System.Windows.Automation.PropertyCondition]::new(
        [System.Windows.Automation.AutomationElement]::NameProperty, 'Restart to install')
    $buttonMatch = [System.Windows.Automation.PropertyCondition]::new(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)
    $confirmationMatch = [System.Windows.Automation.AndCondition]::new($labelMatch, $buttonMatch)
    while ($timer.Elapsed.TotalSeconds -lt $waitSeconds) {
        foreach ($window in $root.FindAll([System.Windows.Automation.TreeScope]::Children, $processMatch)) {
            $confirm = $window.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $confirmationMatch)
            if ($confirm -and $confirm.Current.IsEnabled) {
                $confirm.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
                Write-Output 'Accepted the installed Electron restart dialog.'
                exit 0
            }
        }
        Start-Sleep -Milliseconds $pollMilliseconds
    }
    throw 'The installed Electron restart dialog did not appear.'
}

$expected = (Resolve-Path -LiteralPath $Executable).Path
while ($timer.Elapsed.TotalSeconds -lt $waitSeconds) {
    $application = Get-Process -Name 'muzo' -ErrorAction SilentlyContinue | Where-Object {
        $_.Id -ne $LegacyProcessId -and $_.Path -eq $expected -and
            $_.MainWindowHandle -ne [IntPtr]::Zero -and $_.MainWindowTitle -eq 'Muzo'
    } | Select-Object -First 1
    if ($application) {
        Write-Output 'The replacement executable opened the Muzo library window.'
        exit 0
    }
    Start-Sleep -Milliseconds $pollMilliseconds
}
throw 'The replacement executable did not reopen the Muzo window.'
