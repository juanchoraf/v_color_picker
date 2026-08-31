$ErrorActionPreference = "Stop"

$App = "v_color_picker"
$DisplayName = "v_color_picker"
$Publisher = "TheVelasquez.com"
$InstallDir = Join-Path $env:ProgramFiles $App
$Version = "0.1.0"

function Test-Admin {
  $Identity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $Principal = New-Object Security.Principal.WindowsPrincipal($Identity)
  return $Principal.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
}

if (-not (Test-Admin)) {
  Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
  exit
}

$ScriptDir = Split-Path -Parent $PSCommandPath
$SourceExe = Join-Path $ScriptDir "$App.exe"
if (-not (Test-Path $SourceExe)) {
  $SourceExe = Get-ChildItem -Path $ScriptDir -Filter "$($App)_v*_x86_64.exe" | Select-Object -First 1 -ExpandProperty FullName
}
if (-not $SourceExe -or -not (Test-Path $SourceExe)) {
  throw "Missing $App executable next to this installer script."
}

if ($SourceExe -match "$($App)_v([0-9]+\.[0-9]+\.[0-9]+[^\\]*)_x86_64\.exe$") {
  $Version = $Matches[1]
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item $SourceExe (Join-Path $InstallDir "$App.exe") -Force
if (Test-Path (Join-Path $ScriptDir "logo.svg")) {
  Copy-Item (Join-Path $ScriptDir "logo.svg") (Join-Path $InstallDir "logo.svg") -Force
}

$Programs = Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\$DisplayName"
New-Item -ItemType Directory -Force -Path $Programs | Out-Null
$Shell = New-Object -ComObject WScript.Shell
$Shortcut = $Shell.CreateShortcut((Join-Path $Programs "$DisplayName.lnk"))
$Shortcut.TargetPath = Join-Path $InstallDir "$App.exe"
$Shortcut.Arguments = ""
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Save()

$UninstallKey = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\v_color_picker"
New-Item -Path $UninstallKey -Force | Out-Null
New-ItemProperty -Path $UninstallKey -Name DisplayName -Value $DisplayName -PropertyType String -Force | Out-Null
New-ItemProperty -Path $UninstallKey -Name DisplayVersion -Value $Version -PropertyType String -Force | Out-Null
New-ItemProperty -Path $UninstallKey -Name Publisher -Value $Publisher -PropertyType String -Force | Out-Null
New-ItemProperty -Path $UninstallKey -Name InstallLocation -Value $InstallDir -PropertyType String -Force | Out-Null
New-ItemProperty -Path $UninstallKey -Name UninstallString -Value "powershell.exe -ExecutionPolicy Bypass -File `"$InstallDir\uninstall_windows.ps1`"" -PropertyType String -Force | Out-Null
Copy-Item (Join-Path $ScriptDir "uninstall_windows.ps1") (Join-Path $InstallDir "uninstall_windows.ps1") -Force -ErrorAction SilentlyContinue

Write-Host "Installed $App for all users."
