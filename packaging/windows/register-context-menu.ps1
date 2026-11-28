<#
.SYNOPSIS
  Register or remove the per-user Windows Explorer context menu for GrammarCraft Quick Review.

.DESCRIPTION
  Adds HKCU:\Software\Classes\*\shell\GrammarCraftQuickReview (no administrator
  privileges required) so right-clicking any .txt, .md, .docx, or .grammarcraft.json
  file launches `grammarcraft.exe --quick-review --file "%1"`.
  Also prints instructions for binding a global clipboard shortcut using
  `grammarcraft.exe --quick-review --from-clipboard`.
#>
param(
  [string] $ExePath = '',
  [switch] $Uninstall
)

$ErrorActionPreference = 'Stop'
$ShellKey = 'HKCU:\Software\Classes\*\shell\GrammarCraftQuickReview'
$CommandKey = Join-Path $ShellKey 'command'

if ($Uninstall) {
  if (Test-Path -LiteralPath $ShellKey) {
    Remove-Item -LiteralPath $ShellKey -Recurse -Force
    Write-Output "Removed per-user context menu key: $ShellKey"
  } else {
    Write-Output "Context menu key already absent: $ShellKey"
  }
  exit 0
}

if (-not $ExePath) {
  $candidates = @(
    (Join-Path $PSScriptRoot 'grammarcraft.exe'),
    (Join-Path $env:LOCALAPPDATA 'Programs\GrammarCraft\grammarcraft.exe')
  )
  foreach ($cand in $candidates) {
    if ($cand -and (Test-Path -LiteralPath $cand)) {
      $ExePath = (Resolve-Path -LiteralPath $cand).Path
      break
    }
  }
  if (-not $ExePath) {
    $cmd = Get-Command 'grammarcraft.exe' -ErrorAction SilentlyContinue
    if ($cmd) { $ExePath = $cmd.Source }
  }
}

if (-not $ExePath) {
  throw 'Could not locate grammarcraft.exe. Pass -ExePath <full-path-to-grammarcraft.exe>.'
}

New-Item -Path $ShellKey -Force | Out-Null
Set-ItemProperty -LiteralPath $ShellKey -Name '(default)' -Value 'GrammarCraft Quick Review'
Set-ItemProperty -LiteralPath $ShellKey -Name 'MUIVerb' -Value 'GrammarCraft Quick Review'
Set-ItemProperty -LiteralPath $ShellKey -Name 'Icon' -Value "$ExePath,0"

New-Item -Path $CommandKey -Force | Out-Null
Set-ItemProperty -LiteralPath $CommandKey -Name '(default)' -Value "`"$ExePath`" --quick-review --file `"%1`""

Write-Output "Registered per-user Explorer context menu: $ShellKey"
Write-Output ""
Write-Output "Global clipboard shortcut tip (no admin needed):"
Write-Output "  Create a Start Menu or Desktop shortcut targeting:"
Write-Output "    `"$ExePath`" --quick-review --from-clipboard"
Write-Output "  and assign Shortcut key (e.g. Ctrl+Alt+G) in Properties, or map it in PowerToys Keyboard Manager."
