[CmdletBinding()]
param(
  [switch]$Locked,
  [switch]$AllowUpdates,
  [switch]$NoUpdate,
  [switch]$SkipCodeSigning,
  [string]$CodeSigningCertPath = $env:V_COLOR_PICKER_CODESIGN_PFX,
  [string]$CodeSigningCertPassword = $env:V_COLOR_PICKER_CODESIGN_PASSWORD,
  [string]$CodeSigningCertThumbprint = $env:V_COLOR_PICKER_CODESIGN_THUMBPRINT,
  [string]$CodeSigningTimestampUrl = $(if ($env:V_COLOR_PICKER_CODESIGN_TIMESTAMP_URL) { $env:V_COLOR_PICKER_CODESIGN_TIMESTAMP_URL } else { "http://timestamp.digicert.com" })
)

$ErrorActionPreference = "Stop"

$App = "v_color_picker"
$DisplayName = "V_Color_Picker"
$Publisher = "TheVelasquez.com"
$RootDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Manifest = Join-Path $RootDir "Cargo.toml"
$VersionMatch = Select-String -Path $Manifest -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
$Version = $VersionMatch.Matches[0].Groups[1].Value

if (-not [Environment]::Is64BitOperatingSystem) {
  throw "Only 64-bit Windows builds are supported."
}

$Arch = "x86_64"
$OutDir = Join-Path $RootDir "versions\$($App)_v$Version"
$Stage = Join-Path $OutDir "windows_stage"
$ArtifactArch = "windows_$Arch"
$ExeAsset = Join-Path $OutDir "$($App)_v$($Version)_$ArtifactArch.exe"
$MsiAsset = Join-Path $OutDir "$($App)_v$($Version)_$ArtifactArch.msi"
$CertDir = Join-Path $RootDir "certs"
$CertSubject = "CN=TheVelasquez.com V_Color_Picker Code Signing"
$CertPassword = "TheVelasquez.com-v_color_picker-local"
$PfxPath = Join-Path $CertDir "v_color_picker_codesign.pfx"
$CerPath = Join-Path $CertDir "v_color_picker_codesign.cer"
$RunningInGitHubActions = $env:GITHUB_ACTIONS -eq "true"

if ($Locked) {
  $NoUpdate = $true
}
if ($AllowUpdates -and $NoUpdate) {
  throw "Use either -AllowUpdates or -NoUpdate, not both."
}
if (-not $AllowUpdates) {
  $NoUpdate = $true
}

function Sign-IfPossible {
  param(
    [Parameter(Mandatory = $true)][string]$Path,
    [AllowEmptyString()][string]$PfxPath,
    [AllowEmptyString()][string]$Password,
    [AllowEmptyString()][string]$Thumbprint
  )

  if ($SkipCodeSigning) {
    Write-Warning "Skipped signing $Path"
    return
  }

  $SignTool = Get-Command signtool.exe -ErrorAction SilentlyContinue
  if (-not $SignTool) {
    Write-Warning "signtool.exe not found; skipped signing $Path"
    return
  }

  $SignArgs = @("sign", "/fd", "SHA256")
  if (-not [string]::IsNullOrWhiteSpace($CodeSigningTimestampUrl)) {
    $SignArgs += @("/tr", $CodeSigningTimestampUrl, "/td", "SHA256")
  }
  if (-not [string]::IsNullOrWhiteSpace($Thumbprint)) {
    $SignArgs += @("/sha1", $Thumbprint)
  } elseif (-not [string]::IsNullOrWhiteSpace($PfxPath)) {
    $SignArgs += @("/f", $PfxPath)
    if (-not [string]::IsNullOrEmpty($Password)) {
      $SignArgs += @("/p", $Password)
    }
  } else {
    Write-Warning "No code signing certificate configured; skipped signing $Path"
    return
  }
  $SignArgs += $Path

  & $SignTool.Source @SignArgs
  if ($LASTEXITCODE -ne 0) {
    throw "signtool.exe failed for $Path with exit code $LASTEXITCODE"
  }
}

function Get-InnoSetupCompiler {
  $Command = Get-Command ISCC.exe -ErrorAction SilentlyContinue
  if ($Command) {
    return $Command.Source
  }

  $Candidates = @(
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
  )

  foreach ($Candidate in $Candidates) {
    if (Test-Path $Candidate) {
      return $Candidate
    }
  }

  return $null
}

New-Item -ItemType Directory -Force -Path $OutDir, $Stage, $CertDir | Out-Null
if (-not $NoUpdate) {
  Push-Location $RootDir
  try {
    cargo update
    if ($LASTEXITCODE -ne 0) {
      throw "cargo update failed with exit code $LASTEXITCODE"
    }
  } finally {
    Pop-Location
  }
}

$CargoArgs = @("build", "--release", "--manifest-path", $Manifest)
if ($Locked) {
  $CargoArgs += "--locked"
}
& cargo @CargoArgs
if ($LASTEXITCODE -ne 0) {
  throw "cargo build failed with exit code $LASTEXITCODE"
}

Copy-Item (Join-Path $RootDir "target\release\$App.exe") (Join-Path $Stage "$App.exe") -Force
Copy-Item (Join-Path $RootDir "assets\logo.svg") (Join-Path $Stage "logo.svg") -Force
Copy-Item (Join-Path $RootDir "assets\logo.svg") (Join-Path $OutDir "logo.svg") -Force
Set-Content -Path (Join-Path $Stage "$App.ps1") -Encoding UTF8 -Value @"
& "`$PSScriptRoot\$App.exe" @args
"@

$SigningPfxPath = $CodeSigningCertPath
$SigningPassword = $CodeSigningCertPassword
$SigningThumbprint = $CodeSigningCertThumbprint
$HasConfiguredCodeSigningCertificate = (
  -not [string]::IsNullOrWhiteSpace($SigningThumbprint) -or
  -not [string]::IsNullOrWhiteSpace($SigningPfxPath)
)

if ($RunningInGitHubActions -and -not $HasConfiguredCodeSigningCertificate -and -not $SkipCodeSigning) {
  Write-Warning "No V_COLOR_PICKER_CODESIGN_* secret configured. Building unsigned Windows artifacts in GitHub Actions."
  $SkipCodeSigning = $true
}

if (-not $SkipCodeSigning -and [string]::IsNullOrWhiteSpace($SigningThumbprint)) {
  if (-not [string]::IsNullOrWhiteSpace($SigningPfxPath)) {
    $SigningPfxPath = (Resolve-Path $SigningPfxPath).Path
    if ($RunningInGitHubActions -and [string]::IsNullOrEmpty($SigningPassword)) {
      throw "V_COLOR_PICKER_CODESIGN_PASSWORD is required when V_COLOR_PICKER_CODESIGN_PFX is configured in GitHub Actions."
    }
  } else {
    $Cert = Get-ChildItem Cert:\CurrentUser\My |
      Where-Object { $_.Subject -eq $CertSubject } |
      Select-Object -First 1

    if (-not $Cert) {
      $Cert = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject $CertSubject `
        -KeyUsage DigitalSignature `
        -FriendlyName "$DisplayName local code signing" `
        -CertStoreLocation Cert:\CurrentUser\My
    }

    $SecurePassword = ConvertTo-SecureString $CertPassword -AsPlainText -Force
    Export-PfxCertificate -Cert $Cert -FilePath $PfxPath -Password $SecurePassword -Force | Out-Null
    Export-Certificate -Cert $Cert -FilePath $CerPath -Force | Out-Null
    Import-Certificate -FilePath $CerPath -CertStoreLocation Cert:\CurrentUser\TrustedPublisher | Out-Null
    Import-Certificate -FilePath $CerPath -CertStoreLocation Cert:\CurrentUser\Root | Out-Null
    $SigningPfxPath = $PfxPath
    $SigningPassword = $CertPassword
  }
}

Copy-Item (Join-Path $Stage "$App.exe") $ExeAsset -Force
Sign-IfPossible -Path $ExeAsset -PfxPath $SigningPfxPath -Password $SigningPassword -Thumbprint $SigningThumbprint

$Wix = Get-Command wix.exe -ErrorAction SilentlyContinue
if ($Wix) {
  $Wxs = Join-Path $OutDir "$App.wxs"
  Set-Content -Path $Wxs -Encoding UTF8 -Value @"
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="$DisplayName" Manufacturer="$Publisher" Version="$Version" UpgradeCode="8f812a7d-4e61-4c59-9fc5-44df6ad67f8a">
    <MajorUpgrade DowngradeErrorMessage="A newer version of $DisplayName is already installed." />
    <MediaTemplate EmbedCab="yes" />
    <StandardDirectory Id="ProgramFiles64Folder">
      <Directory Id="INSTALLFOLDER" Name="v_color_picker">
        <Component Id="MainExecutable" Guid="a3d6d137-7d36-4d77-a164-4f72b5c67266">
          <File Source="$Stage\$App.exe" KeyPath="yes" />
          <File Source="$Stage\$App.ps1" />
          <File Source="$Stage\logo.svg" />
          <Environment Id="PathEntry" Name="PATH" Value="[INSTALLFOLDER]" Permanent="no" Part="last" Action="set" System="yes" />
        </Component>
      </Directory>
    </StandardDirectory>
    <StandardDirectory Id="ProgramMenuFolder">
      <Directory Id="AppProgramMenuFolder" Name="$DisplayName">
        <Component Id="StartMenuShortcut" Guid="55df50a0-1331-4f0c-a29e-a4b377a03123">
          <Shortcut Id="StartMenuShortcutLink" Name="$DisplayName" Target="[System64Folder]WindowsPowerShell\v1.0\powershell.exe" Arguments="-NoExit -ExecutionPolicy Bypass -File &quot;[INSTALLFOLDER]$App.ps1&quot;" WorkingDirectory="INSTALLFOLDER" />
          <RemoveFolder Id="RemoveAppProgramMenuFolder" On="uninstall" />
          <RegistryValue Root="HKLM" Key="Software\TheVelasquez.com\v_color_picker" Name="installed" Type="integer" Value="1" KeyPath="yes" />
        </Component>
      </Directory>
    </StandardDirectory>
    <Feature Id="MainFeature" Title="$DisplayName" Level="1">
      <ComponentRef Id="MainExecutable" />
      <ComponentRef Id="StartMenuShortcut" />
    </Feature>
  </Package>
</Wix>
"@
  & $Wix.Source build -arch x64 -o $MsiAsset $Wxs
  if ($LASTEXITCODE -ne 0) {
    throw "wix.exe failed with exit code $LASTEXITCODE"
  }
  Sign-IfPossible -Path $MsiAsset -PfxPath $SigningPfxPath -Password $SigningPassword -Thumbprint $SigningThumbprint
} else {
  Write-Warning "WiX wix.exe not found; skipped .msi packaging."
}

$Iscc = Get-InnoSetupCompiler
if ($Iscc) {
  $Iss = Join-Path $OutDir "$App.iss"
  Set-Content -Path $Iss -Encoding UTF8 -Value @"
[Setup]
AppId={{9D2B4F32-0FF1-4A91-A08A-0F3B9E2E29A5}
AppName=$DisplayName
AppVersion=$Version
AppPublisher=$Publisher
DefaultDirName={autopf}\v_color_picker
DefaultGroupName=$DisplayName
OutputDir=$OutDir
OutputBaseFilename=$($App)_v$($Version)_$ArtifactArch
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin
UninstallDisplayName=$DisplayName
Compression=lzma2
SolidCompression=yes

[Files]
Source: "$Stage\$App.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "$Stage\$App.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "$Stage\logo.svg"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\$DisplayName"; Filename: "powershell.exe"; Parameters: "-NoExit -ExecutionPolicy Bypass -File ""{app}\$App.ps1"""; WorkingDir: "{app}"
Name: "{autodesktop}\$DisplayName"; Filename: "powershell.exe"; Parameters: "-NoExit -ExecutionPolicy Bypass -File ""{app}\$App.ps1"""; WorkingDir: "{app}"

[Registry]
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath(ExpandConstant('{app}'))

[Run]
Filename: "powershell.exe"; Parameters: "-NoExit -ExecutionPolicy Bypass -File ""{app}\$App.ps1"""; Description: "Open $DisplayName"; Flags: postinstall skipifsilent nowait

[Code]
function NeedsAddPath(Path: string): Boolean;
var
  OldPath: string;
begin
  if not RegQueryStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', OldPath) then
    Result := True
  else
    Result := Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(OldPath) + ';') = 0;
end;
"@
  & $Iscc $Iss
  if ($LASTEXITCODE -ne 0) {
    throw "Inno Setup failed with exit code $LASTEXITCODE"
  }
  Sign-IfPossible -Path $ExeAsset -PfxPath $SigningPfxPath -Password $SigningPassword -Thumbprint $SigningThumbprint
} else {
  Write-Warning "Inno Setup ISCC.exe not found; kept the signed portable .exe asset."
}

Copy-Item (Join-Path $RootDir "scripts\install_windows.ps1") (Join-Path $OutDir "$($App)_v$($Version)_$($ArtifactArch)_install.ps1") -Force
Copy-Item (Join-Path $RootDir "scripts\uninstall_windows.ps1") (Join-Path $OutDir "$($App)_v$($Version)_$($ArtifactArch)_uninstall.ps1") -Force

Write-Host "Release files written to $OutDir"
