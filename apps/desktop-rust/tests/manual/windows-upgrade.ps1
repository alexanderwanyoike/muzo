param(
    [Parameter(Mandatory=$true)][string] $LegacyInstaller,
    [Parameter(Mandatory=$true)][string] $RustInstaller,
    [Parameter(Mandatory=$true)][string] $InstallDirectory,
    [Parameter(Mandatory=$true)][string] $RustBinary,
    [Parameter(Mandatory=$true)][string] $FixtureDirectory
)

$ErrorActionPreference = 'Stop'
$uninstallRoot = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall'
$legacyKey = Join-Path $uninstallRoot '13cf9733-a28d-56dd-812e-45bf355a27be'
if (Test-Path $legacyKey) { throw 'The test requires a fresh runner profile.' }

function Get-MuzoRegistrations {
    Get-ChildItem $uninstallRoot | Where-Object {
        (Get-ItemProperty $_.PSPath).DisplayName -like 'Muzo*'
    } | Select-Object -ExpandProperty PSChildName
}

function Invoke-Installer([string] $Installer, [string[]] $Arguments) {
    $process = Start-Process -FilePath $Installer -ArgumentList $Arguments -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "Installer exited with code $($process.ExitCode)" }
}

$profileDirectory = Join-Path $env:APPDATA 'muzo'
$chromiumProfile = Join-Path $env:APPDATA 'Muzo'
$leveldb = Join-Path $chromiumProfile 'Local Storage\leveldb'
if ($env:MUZO_DATA_DIR -or (Test-Path (Join-Path $profileDirectory 'muzo.sqlite')) -or
    (Test-Path (Join-Path $profileDirectory 'muzo-rust.sqlite')) -or (Test-Path $leveldb)) {
    throw 'The profile migration test requires the default paths on a fresh runner.'
}

function Invoke-ProfileTool([string] $Script, [string[]] $Arguments) {
    $tool = Join-Path $PSScriptRoot $Script
    & node $tool @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Script exited with code $LASTEXITCODE." }
}

function Close-UpgradeApp([Diagnostics.Process] $Application) {
    $exitTimeoutMs = 10000
    if ($Application.HasExited -or !$Application.CloseMainWindow() -or !$Application.WaitForExit($exitTimeoutMs)) {
        throw 'Muzo did not close cleanly after the profile check.'
    }
    if ($Application.ExitCode -ne 0) { throw "Muzo exited with code $($Application.ExitCode)." }
}

Invoke-Installer $LegacyInstaller @('/S', "/D=$InstallDirectory")
if (!(Test-Path "$InstallDirectory\muzo.exe")) { throw 'Legacy installation did not finish.' }
if (!(Test-Path $legacyKey)) { throw 'The Electron installer did not register.' }
$legacyRegistrations = @(Get-MuzoRegistrations)

$applicationPath = Join-Path $InstallDirectory 'muzo.exe'
New-Item $FixtureDirectory -ItemType Directory | Out-Null
$music = Join-Path $FixtureDirectory 'Music'
$preferences = Join-Path $FixtureDirectory 'preferences.json'
$baseline = Join-Path $FixtureDirectory 'baseline.json'
$legacyApp = Start-Process -FilePath $applicationPath -ArgumentList '--remote-debugging-port=0' -PassThru
try {
    Invoke-ProfileTool 'seed-upgrade-profile.mjs' @($chromiumProfile, $music, $preferences)
    Close-UpgradeApp $legacyApp
} finally {
    if (!$legacyApp.HasExited) { Stop-Process -InputObject $legacyApp }
    $legacyApp.Dispose()
}
Invoke-ProfileTool 'profile-upgrade.mjs' @('capture', $profileDirectory, $leveldb, $baseline, $preferences)

New-Item $profileDirectory -ItemType Directory -Force | Out-Null
$marker = Join-Path $profileDirectory 'rust-upgrade-keep.txt'
Set-Content $marker 'legacy profile'
Invoke-Installer $RustInstaller @('/S')

$installed = Get-FileHash "$InstallDirectory\muzo.exe"
if ($installed.Hash -ne (Get-FileHash $RustBinary).Hash) {
    throw 'The Rust executable was not installed in the existing directory.'
}
foreach ($runtime in @('resources\app', 'resources\app.asar', 'chrome_elf.dll', 'icudtl.dat')) {
    if (Test-Path "$InstallDirectory\$runtime") { throw "Electron runtime payload remains: $runtime" }
}
if (!(Test-Path $legacyKey)) { throw 'The established Windows uninstall registration is missing.' }
if (Compare-Object $legacyRegistrations @(Get-MuzoRegistrations)) { throw 'The Muzo uninstall registrations changed.' }
if ((Get-Content $marker) -ne 'legacy profile') { throw 'The installer did not preserve the existing profile.' }

$rustApp = Start-Process -FilePath $applicationPath -PassThru
try {
    Invoke-ProfileTool 'profile-upgrade.mjs' @('verify', $profileDirectory, $leveldb, $baseline)
    Close-UpgradeApp $rustApp
} finally {
    if (!$rustApp.HasExited) { Stop-Process -InputObject $rustApp }
    $rustApp.Dispose()
}
