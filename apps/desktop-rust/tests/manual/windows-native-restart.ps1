param(
    [Parameter(Mandatory=$true)][string] $Installer,
    [Parameter(Mandatory=$true)][string] $InstallDirectory
)

$ErrorActionPreference = 'Stop'
$fixtureInstall = Join-Path $env:RUNNER_TEMP 'muzo-in-app-upgrade'
if (!$env:CI -or $InstallDirectory -ne $fixtureInstall) {
    throw 'The restart check requires the isolated Windows upgrade installation.'
}

$installed = Join-Path $InstallDirectory 'muzo.exe'
$profile = Join-Path $env:APPDATA 'muzo'
$appearance = Join-Path $profile 'appearance-rust.json'
$catalogue = Join-Path $profile 'muzo-rust.sqlite'
if (!(Test-Path $catalogue)) { throw 'The migrated catalogue is missing.' }
$originalAppearanceHash = (Get-FileHash $appearance).Hash
$installedHash = (Get-FileHash $installed).Hash

function Get-InstalledMuzo {
    Get-Process -Name 'muzo' -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $installed }
}

foreach ($application in @(Get-InstalledMuzo)) {
    if (!$application.CloseMainWindow() -or !$application.WaitForExit(10000)) {
        throw 'The previous Rust Muzo instance did not exit cleanly.'
    }
}

$previousBinary = Join-Path $env:RUNNER_TEMP 'muzo-before-native-restart.exe'
Move-Item -LiteralPath $installed -Destination $previousBinary
$env:MUZO_RESTART_INSTALLER = (Resolve-Path -LiteralPath $Installer).Path
$env:MUZO_RESTART_INSTALLER_SHA256 = (Get-FileHash $Installer).Hash
cargo +1.95 test --release --locked --manifest-path apps/desktop-rust/Cargo.toml --lib it_restarts_with_the_installed_windows_update -- --ignored
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$deadline = [DateTime]::UtcNow.AddSeconds(75)
do {
    $reopened = Get-InstalledMuzo | Where-Object { $_.MainWindowTitle -eq 'Muzo' } | Select-Object -First 1
    if ($reopened) { break }
    Start-Sleep -Milliseconds 250
} while ([DateTime]::UtcNow -lt $deadline)
if (!$reopened -or !(Test-Path $installed) -or (Get-FileHash $installed).Hash -ne $installedHash) {
    throw 'The Rust restart did not replace the installed executable and reopen the Muzo window.'
}
if (!(Test-Path $catalogue) -or (Get-FileHash $appearance).Hash -ne $originalAppearanceHash) {
    throw 'The Rust restart did not preserve the migrated profile.'
}
Write-Host 'Rust Restart reinstalled Muzo silently and reopened the migrated profile.'
