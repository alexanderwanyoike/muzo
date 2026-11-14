param(
    [Parameter(Mandatory=$true)][string] $LegacyInstaller,
    [Parameter(Mandatory=$true)][string] $RustInstaller,
    [Parameter(Mandatory=$true)][string] $InstallDirectory,
    [Parameter(Mandatory=$true)][string] $RustBinary
)

$ErrorActionPreference = 'Stop'
$uninstallRoot = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall'
$legacyKey = Join-Path $uninstallRoot '13cf9733-a28d-56dd-812e-45bf355a27be'
if (Test-Path $legacyKey) { throw 'The test requires a fresh runner profile.' }

function Invoke-Installer([string] $Installer, [string[]] $Arguments) {
    $process = Start-Process -FilePath $Installer -ArgumentList $Arguments -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "Installer exited with code $($process.ExitCode)" }
}

Invoke-Installer $LegacyInstaller @('/S', "/D=$InstallDirectory")
if (!(Test-Path "$InstallDirectory\muzo.exe")) { throw 'Legacy installation did not finish.' }
if (!(Test-Path $legacyKey)) { throw 'The Electron installer did not register.' }

$profileDirectory = Join-Path $env:APPDATA 'muzo'
New-Item $profileDirectory -ItemType Directory -Force | Out-Null
$marker = Join-Path $profileDirectory 'rust-upgrade-keep.txt'
Set-Content $marker 'legacy profile'
Invoke-Installer $RustInstaller @('/S')

$installed = Get-FileHash "$InstallDirectory\muzo.exe"
if ($installed.Hash -ne (Get-FileHash $RustBinary).Hash) {
    throw 'The Rust executable was not installed in the existing directory.'
}
if (Test-Path "$InstallDirectory\resources\app\dist-electron\main.cjs") { throw 'Electron runtime payload remains.' }
if (!(Test-Path $legacyKey)) { throw 'The established Windows uninstall registration is missing.' }
if (Test-Path (Join-Path $uninstallRoot 'Muzo')) { throw 'The installer added a second uninstall registration.' }
if ((Get-Content $marker) -ne 'legacy profile') { throw 'The installer did not preserve the existing profile.' }
