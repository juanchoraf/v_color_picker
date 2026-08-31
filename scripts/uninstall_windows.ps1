$ErrorActionPreference = "Stop"

$App = "v_color_picker"
$DisplayName = "v_color_picker"
$InstallDir = Join-Path $env:ProgramFiles $App

function Test-Admin {
  $Identity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $Principal = New-Object Security.Principal.WindowsPrincipal($Identity)
  return $Principal.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
}

if (-not (Test-Admin)) {
  Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
  exit
}

$MachinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($MachinePath) {
  $Parts = $MachinePath.Split(";") | Where-Object { $_ -and ($_.TrimEnd("\") -ne $InstallDir.TrimEnd("\")) }
  [Environment]::SetEnvironmentVariable("Path", ($Parts -join ";"), "Machine")
}

Remove-Item -Recurse -Force $InstallDir -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force (Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\v_color_picker") -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force (Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\$DisplayName") -ErrorAction SilentlyContinue
Remove-Item -Force (Join-Path ([Environment]::GetFolderPath("CommonDesktopDirectory")) "v_color_picker.lnk") -ErrorAction SilentlyContinue
Remove-Item -Force (Join-Path ([Environment]::GetFolderPath("CommonDesktopDirectory")) "$DisplayName.lnk") -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\v_color_picker" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "HKLM:\Software\TheVelasquez.com\v_color_picker" -ErrorAction SilentlyContinue

Get-ChildItem "C:\Users" -Directory -ErrorAction SilentlyContinue | ForEach-Object {
  Remove-Item -Recurse -Force (Join-Path $_.FullName "AppData\Roaming\v_color_picker") -ErrorAction SilentlyContinue
  Remove-Item -Recurse -Force (Join-Path $_.FullName "AppData\Local\v_color_picker") -ErrorAction SilentlyContinue
  Remove-Item -Recurse -Force (Join-Path $_.FullName "AppData\Local\TheVelasquez.com\v_color_picker") -ErrorAction SilentlyContinue
}

Write-Host "Uninstalled $App and removed app traces."
