<#
.SYNOPSIS
  Build, sign and package GrammarCraft for Windows.

.DESCRIPTION
  Produces, in $env:DIST (default: dist/release):
    grammarcraft-<version>-windows-<arch>.msi
    grammarcraft-<version>-windows-<arch>-portable.zip
    grammarcraft-<version>-windows-<arch>-symbols.zip (when PDBs exist)
#>
param(
  [ValidateSet('x64', 'x86', 'arm64')] [string] $Arch = 'x64',
  [switch] $SkipBuild
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$AppId = 'ai.storyteller.grammarcraft'

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

$Version = $env:GRAMMARCRAFT_VERSION
if (-not $Version) {
  $inPkg = $false
  foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
    if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
    if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
  }
}
if (-not $Version) { throw 'could not read [workspace.package] version from Cargo.toml' }
$MsiVersion = ($Version -split '-')[0]

$Target = switch ($Arch) { 'x64' { 'x86_64-pc-windows-msvc' } 'x86' { 'i686-pc-windows-msvc' } 'arm64' { 'aarch64-pc-windows-msvc' } }
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' }
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

Write-Output "GrammarCraft $Version for Windows $Arch ($Target)"

if (-not $SkipBuild) {
  $flagVar = 'CARGO_TARGET_' + ($Target.ToUpper() -replace '-', '_') + '_RUSTFLAGS'
  [Environment]::SetEnvironmentVariable($flagVar, '-C target-feature=+crt-static')
  Invoke-Native "cargo build ($Target)" { cargo build --release --locked -p grammarcraft -p grammarcraft-cli --target $Target }
}

$Bin = Join-Path $TargetDir "$Target\release"

function Get-PeHeader([string] $Path) {
  $bytes = [System.IO.File]::ReadAllBytes($Path)
  $pe = [BitConverter]::ToInt32($bytes, 0x3C)
  return @{ Machine = [BitConverter]::ToUInt16($bytes, $pe + 4); Subsystem = [BitConverter]::ToUInt16($bytes, $pe + 0x5C) }
}
$Machine = switch ($Arch) { 'x64' { 0x8664 } 'x86' { 0x14C } 'arm64' { 0xAA64 } }
foreach ($check in @(@('grammarcraft.exe', 2), @('grammarcraft-cli.exe', 3))) {
  $h = Get-PeHeader (Join-Path $Bin $check[0])
  if ($h.Machine -ne $Machine) { throw "$($check[0]) is for machine 0x$('{0:X}' -f $h.Machine), expected 0x$('{0:X}' -f $Machine) ($Arch)" }
  if ($h.Subsystem -ne $check[1]) { throw "$($check[0]) has PE subsystem $($h.Subsystem), expected $($check[1])" }
  Write-Output "ok $($check[0]): $Arch, PE subsystem $($h.Subsystem)"
}

$Stage = Join-Path $TargetDir "windows-package\$Arch"
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item (Join-Path $Bin 'grammarcraft.exe'), (Join-Path $Bin 'grammarcraft-cli.exe') $Stage

& (Join-Path $PSScriptRoot 'sign.ps1') (Join-Path $Stage 'grammarcraft.exe') (Join-Path $Stage 'grammarcraft-cli.exe')

$Docs = Join-Path $Stage 'notices'
New-Item -ItemType Directory -Force -Path $Docs | Out-Null
foreach ($f in 'README.md', 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'ATTRIBUTION.md') {
  $src = Join-Path $Root $f
  if (Test-Path -LiteralPath $src) { Copy-Item -LiteralPath $src -Destination $Docs }
}
if ($env:CRAFT_FONTS_DIR -and (Test-Path -LiteralPath $env:CRAFT_FONTS_DIR)) {
  foreach ($ofl in Get-ChildItem -Path (Join-Path $env:CRAFT_FONTS_DIR 'fonts\*\OFL.txt') -File -ErrorAction SilentlyContinue) {
    Copy-Item -LiteralPath $ofl.FullName -Destination (Join-Path $Docs "OFL-$($ofl.Directory.Name).txt")
  }
}

function New-InstallerArt([string] $IconPng, [string] $BannerOut, [string] $DialogOut) {
  Add-Type -AssemblyName System.Drawing
  $navy = [System.Drawing.ColorTranslator]::FromHtml('#1e3a5f')
  $accent = [System.Drawing.ColorTranslator]::FromHtml('#16a34a')
  $icon = [System.Drawing.Image]::FromFile($IconPng)
  try {
    foreach ($spec in @(@($BannerOut, 493, 58), @($DialogOut, 493, 312))) {
      $bmp = New-Object System.Drawing.Bitmap $spec[1], $spec[2], ([System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
      $g = [System.Drawing.Graphics]::FromImage($bmp)
      try {
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
        $g.Clear([System.Drawing.Color]::White)
        if ($spec[2] -eq 58) {
          $g.DrawImage($icon, 493 - 46 - 8, 6, 46, 46)
        } else {
          $panel = New-Object System.Drawing.SolidBrush $navy
          $stripe = New-Object System.Drawing.SolidBrush $accent
          $g.FillRectangle($panel, 0, 0, 164, 312)
          $g.FillRectangle($stripe, 160, 0, 4, 312)
          $g.DrawImage($icon, 18, 64, 124, 124)
          $panel.Dispose(); $stripe.Dispose()
        }
      } finally { $g.Dispose() }
      $bmp.Save($spec[0], [System.Drawing.Imaging.ImageFormat]::Bmp)
      $bmp.Dispose()
    }
  } finally { $icon.Dispose() }
}
$Banner = Join-Path $Stage 'installer-banner.bmp'
$Dialog = Join-Path $Stage 'installer-dialog.bmp'
New-InstallerArt (Join-Path $Root "assets\app-icon\hicolor\256x256\apps\$AppId.png") $Banner $Dialog

$Msi = Join-Path $Dist "grammarcraft-$Version-windows-$Arch.msi"
& (Join-Path $PSScriptRoot 'check-icons.ps1')
Invoke-Native 'wix build' {
  wix build (Join-Path $PSScriptRoot 'grammarcraft.wxs') -arch $Arch `
    -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext `
    -culture en-US -loc (Join-Path $PSScriptRoot 'grammarcraft.en-us.wxl') `
    -d "Version=$MsiVersion" -d "DisplayVersion=$Version" -d "BinDir=$Stage" -d "IconPath=$(Join-Path $Root 'assets\app-icon\grammarcraft.ico')" `
    -d "BannerBmp=$Banner" -d "DialogBmp=$Dialog" -d "DocsDir=$Docs" `
    -o $Msi
}
Invoke-Native 'MSI shortcut icon validation (ICE50)' {
  wix msi validate $Msi -ice ICE50 -intermediateFolder (Join-Path $Stage 'msi-validation')
}
& (Join-Path $PSScriptRoot 'check-install-policy.ps1') -MsiPath $Msi
Remove-Item -Force -ErrorAction SilentlyContinue ([IO.Path]::ChangeExtension($Msi, '.wixpdb'))
& (Join-Path $PSScriptRoot 'sign.ps1') $Msi

$Portable = Join-Path $TargetDir "windows-package\grammarcraft-$Version-windows-$Arch-portable"
Remove-Item -Recurse -Force $Portable -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Portable | Out-Null
Copy-Item (Join-Path $Stage '*.exe') $Portable
Copy-Item (Join-Path $Docs '*') $Portable
Set-Content -LiteralPath (Join-Path $Portable 'grammarcraft-portable') -Encoding ascii -Value 'This copy of GrammarCraft is portable.'
$Zip = Join-Path $Dist "grammarcraft-$Version-windows-$Arch-portable.zip"
Remove-Item -Force $Zip -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $Portable '*') -DestinationPath $Zip
$Check = Join-Path $TargetDir 'windows-package\portable-zip-check'
Remove-Item -Recurse -Force $Check -ErrorAction SilentlyContinue
Expand-Archive -Path $Zip -DestinationPath $Check -Force
foreach ($f in 'grammarcraft.exe', 'grammarcraft-cli.exe', 'NOTICE', 'grammarcraft-portable') {
  if (-not (Test-Path (Join-Path $Check $f))) { throw "the portable zip has no $f at its root: $Zip" }
}
Remove-Item -Recurse -Force $Check -ErrorAction SilentlyContinue

$Outputs = @($Msi, $Zip)
$Pdbs = @('grammarcraft.pdb', 'grammarcraft_cli.pdb' | ForEach-Object { Join-Path $Bin $_ })
if (@($Pdbs | Where-Object { -not (Test-Path $_) }).Count -eq 0) {
  $Symbols = Join-Path $TargetDir "windows-package\grammarcraft-$Version-windows-$Arch-symbols"
  Remove-Item -Recurse -Force $Symbols -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force -Path $Symbols | Out-Null
  Copy-Item $Pdbs $Symbols
  Copy-Item (Join-Path $PSScriptRoot 'symbols.txt') $Symbols
  $SymbolsZip = Join-Path $Dist "grammarcraft-$Version-windows-$Arch-symbols.zip"
  Remove-Item -Force $SymbolsZip -ErrorAction SilentlyContinue
  Compress-Archive -Path (Join-Path $Symbols '*') -DestinationPath $SymbolsZip
  $Outputs += $SymbolsZip
} elseif ($env:GITHUB_ACTIONS) {
  Write-Output "::warning::no PDBs in $Bin, so there is no symbols zip"
} else {
  Write-Warning "no PDBs in $Bin, so there is no symbols zip"
}

$HostArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
if ($Arch -ne 'arm64' -or $HostArch -eq 'arm64') {
  Invoke-Native 'grammarcraft-cli --version' { & (Join-Path $Stage 'grammarcraft-cli.exe') --version }
} else {
  Write-Output "skipping grammarcraft-cli --version: an $Arch build doesn't run on this $HostArch machine"
}
Get-Item $Outputs | Format-Table Name, Length
