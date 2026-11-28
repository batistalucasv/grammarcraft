<#
.SYNOPSIS
  Check the compiled installation policy without installing any product.
#>
param([Parameter(Mandatory = $true)] [string] $MsiPath)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$policy = Get-Content -Raw -LiteralPath (Join-Path $PSScriptRoot 'install-policy.json') | ConvertFrom-Json
function Read-Rows($Database, [string] $Sql, [int] $Columns) {
  $view = $Database.OpenView($Sql)
  try {
    [void]$view.Execute()
    while ($null -ne ($record = $view.Fetch())) {
      try {
        $values = @(for ($i = 1; $i -le $Columns; $i++) {
          $record.GetType().InvokeMember('StringData', 'GetProperty', $null, $record, @($i))
        })
        ,$values
      } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($record) }
    }
  } finally {
    [void]$view.Close()
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($view)
  }
}
function Set-SessionProperty($Session, [string] $Name, [string] $Value) {
  [void]$Session.GetType().InvokeMember('Property', 'SetProperty', $null, $Session, @($Name, $Value))
}
function Get-SessionProperty($Session, [string] $Name) {
  $Session.GetType().InvokeMember('Property', 'GetProperty', $null, $Session, @($Name))
}
function Get-PackageTemplate($Database) {
  $summary = $Database.GetType().InvokeMember('SummaryInformation', 'GetProperty', $null, $Database, @(0))
  try { [string]$summary.GetType().InvokeMember('Property', 'GetProperty', $null, $summary, @(7)) }
  finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($summary) }
}
$installer = $null; $database = $null; $session = $null
try {
  $installer = New-Object -ComObject WindowsInstaller.Installer
  $file = (Resolve-Path -LiteralPath $MsiPath).Path
  $database = $installer.OpenDatabase($file, 0)
  $session = $installer.OpenPackage($file, 1)
  $properties = @{}
  foreach ($row in (Read-Rows $database 'SELECT `Property`, `Value` FROM `Property`' 2)) { $properties[$row[0]] = $row[1] }
  $secureProperties = @($properties['SecureCustomProperties'] -split ';')
  if ('INSTALLFOLDER' -notin $secureProperties) {
    throw 'The selected installation folder must survive elevation (secure INSTALLFOLDER).'
  }
  if ($properties['ALLUSERS'] -ne '2' -or $properties['MSIINSTALLPERUSER'] -ne '1') {
    throw 'Policy regression: new packages must default to the per-user dual-scope context.'
  }
  if ($properties['UpgradeCode'].Trim('{}') -ne $policy.upgradeCode) { throw 'The existing upgrade identity must not change.' }
  $directories = @{}
  foreach ($row in (Read-Rows $database 'SELECT `Directory`, `Directory_Parent`, `DefaultDir` FROM `Directory`' 3)) {
    $directories[$row[0]] = @($row[1], $row[2])
  }
  if ($directories['INSTALLFOLDER'][0] -ne 'CraftSuiteFolder' -or
      $directories['CraftSuiteFolder'][1] -ne 'ArtCraft') { throw 'Expected the app under the ArtCraft suite folder.' }
  $programs = $directories['CraftSuiteFolder'][0]
  if ($programs -eq 'ProgramFiles6432Folder') { $programs = $directories[$programs][0] }
  if ($programs -notin 'ProgramFilesFolder', 'ProgramFiles64Folder') {
    throw 'Use Windows Installer native folder redirection, not a hard-coded profile path.'
  }
  $guards = @(Read-Rows $database 'SELECT `Condition`, `Description` FROM `LaunchCondition`' 2 |
    Where-Object { $_[0] -match 'CRAFT_(USER|MACHINE)_DIR' })
  if ($guards.Count -ne 2) { throw 'Expected both duplicate-installation guards.' }
  $cases = 0
  foreach ($scope in @('', '2', '1')) {
    foreach ($user in @('', 'D:\Existing user app')) {
      foreach ($machine in @('', 'E:\Existing machine app')) {
        foreach ($installed in @('', '1')) {
          Set-SessionProperty $session 'ALLUSERS' $scope
          Set-SessionProperty $session 'Installed' $installed
          Set-SessionProperty $session 'CRAFT_USER_DIR' $user
          Set-SessionProperty $session 'CRAFT_MACHINE_DIR' $machine
          $allowed = $true
          foreach ($guard in $guards) {
            $value = $session.EvaluateCondition($guard[0])
            if ($value -notin 0, 1) { throw 'Invalid duplicate guard.' }
            if ($value -eq 0) { $allowed = $false }
          }
          $opposite = if ($scope -eq '1') { $user } else { $machine }
          $expected = [bool]$installed -or -not [bool]$opposite
          if ($allowed -ne $expected) { throw "Scope regression: ALLUSERS='$scope', user=$([bool]$user), machine=$([bool]$machine), maintenance=$([bool]$installed)." }
          $cases++
        }
      }
    }
  }
  foreach ($scope in @('', '2', '1')) {
    foreach ($explicit in @('', 'F:\Chosen exact app')) {
      Set-SessionProperty $session 'ALLUSERS' $scope
      Set-SessionProperty $session 'INSTALLFOLDER' $explicit
      Set-SessionProperty $session 'CRAFT_USER_DIR' 'D:\Existing user app'
      Set-SessionProperty $session 'CRAFT_MACHINE_DIR' 'E:\Existing machine app'
      $action = if ($scope -eq '1') { 'CraftRestoreMachineFolder' } else { 'CraftRestoreUserFolder' }
      $restore = @(Read-Rows $database ('SELECT `Condition` FROM `InstallExecuteSequence` WHERE `Action` = ''{0}''' -f $action) 1)[0]
      $selected = $session.EvaluateCondition($restore[0]) -eq 1
      if ($selected -eq [bool]$explicit) { throw "$action must run only when no folder was supplied (condition: $($restore[0]))." }
      if ($selected -and $session.DoAction($action) -ne 1) { throw 'Remembered-folder property action failed.' }
      $expected = if ($explicit) { $explicit } elseif ($scope -eq '1') { 'E:\Existing machine app' } else { 'D:\Existing user app' }
      if ((Get-SessionProperty $session 'INSTALLFOLDER') -ne $expected) { throw 'Custom or remembered folder was not preserved.' }
    }
  }
  $ui = @{}
  foreach ($row in (Read-Rows $database 'SELECT `Action`, `Sequence` FROM `InstallUISequence`' 2)) { $ui[$row[0]] = [int]$row[1] }
  if (-not ($ui['AppSearch'] -lt $ui['CraftPreferMachineScope'] -and
      $ui['CraftPreferMachineScope'] -lt $ui['CraftScopeDlg'] -and
      $ui['CraftScopeDlg'] -lt $ui['FindRelatedProducts'] -and
      $ui['FindRelatedProducts'] -lt $ui['LaunchConditions'] -and
      $ui['CraftScopeDlg'] -lt $ui['CraftRestoreUserFolder'] -and $ui['CraftRestoreUserFolder'] -lt $ui['CostInitialize'] -and
      $ui['CraftScopeDlg'] -lt $ui['CraftRestoreMachineFolder'] -and $ui['CraftRestoreMachineFolder'] -lt $ui['CostInitialize'])) {
    throw 'Scope must be chosen after AppSearch and before FindRelatedProducts, launch checks and costing.'
  }
  $defaults = @(Read-Rows $database 'SELECT `Action`, `Sequence`, `Condition` FROM `InstallUISequence`' 3 | Where-Object { $_[0] -like 'CraftDefault*' })
  if ($defaults.Count -lt 2) { throw 'The wizard must set the new-install folder after the scope page.' }
  foreach ($row in $defaults) {
    if (-not ($ui['CraftScopeDlg'] -lt [int]$row[1] -and [int]$row[1] -lt $ui['CostInitialize'])) { throw "$($row[0]) must run after the scope page and before costing." }
  }
  if (@(Read-Rows $database 'SELECT `Action` FROM `InstallExecuteSequence`' 1 | Where-Object { $_[0] -like 'CraftDefault*' }).Count) {
    throw 'Silent installs must not use the wizard-only default folder actions.'
  }
  $machineRoot = if ((Get-PackageTemplate $database) -match '^Intel') {
    if (Get-SessionProperty $session 'VersionNT64') { ${env:ProgramFiles(x86)} } else { $env:ProgramFiles }
  } elseif ($env:ProgramW6432) { $env:ProgramW6432 } else { $env:ProgramFiles }
  $machineFolder = (Join-Path $machineRoot "ArtCraft\$($policy.name)") + '\'
  foreach ($scope in @('1', '')) {
    Set-SessionProperty $session 'ALLUSERS' $scope
    Set-SessionProperty $session 'INSTALLFOLDER' ''
    Set-SessionProperty $session 'CRAFT_MACHINE_DIR' ''
    Set-SessionProperty $session 'CRAFT_USER_DIR' ''
    foreach ($row in @($defaults | Sort-Object { [int]$_[1] })) {
      if ($session.EvaluateCondition($row[2]) -eq 1 -and $session.DoAction($row[0]) -ne 1) { throw "$($row[0]) failed." }
    }
    $folder = Get-SessionProperty $session 'INSTALLFOLDER'
    $wanted = if ($scope -eq '1') { $machineFolder } else { (Join-Path $env:LOCALAPPDATA "Programs\ArtCraft\$($policy.name)") + '\' }
    if ($folder -ine $wanted) { throw "Wizard default folder for ALLUSERS='$scope' is '$folder', expected '$wanted'." }
  }
  $execute = @{}
  foreach ($row in (Read-Rows $database 'SELECT `Action`, `Sequence` FROM `InstallExecuteSequence`' 2)) { $execute[$row[0]] = [int]$row[1] }
  foreach ($action in @('CraftRestoreUserFolder','CraftRestoreMachineFolder')) {
    if (-not ($execute['AppSearch'] -lt $execute[$action] -and $execute[$action] -lt $execute['CostInitialize'])) {
      throw 'Silent updates must restore the folder before directory costing.'
    }
  }
  foreach ($resolver in @('CraftUFromLegacyExe','CraftMFromLegacyExe')) {
    if (-not ($execute['AppSearch'] -lt $execute[$resolver] -and $execute[$resolver] -lt $execute['LaunchConditions'])) {
      throw 'Both legacy folder resolvers must run after AppSearch and before the launch checks.'
    }
  }
  $appSearch = @{}
  foreach ($row in (Read-Rows $database 'SELECT `Property`, `Signature_` FROM `AppSearch`' 2)) { $appSearch[$row[0]] = $row[1] }
  $locators = @{}
  foreach ($row in (Read-Rows $database 'SELECT `Signature_`, `Root`, `Key`, `Name`, `Type` FROM `RegLocator`' 5)) { $locators[$row[0]] = $row }
  $appPaths = '^Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\[^\\]+\.exe$'
  $searchContract = @(
    @('DIR',       'exact', "Software\$($policy.name)", 'InstallDir',    '2'),
    @('FOLDER',    'exact', "Software\$($policy.name)", 'InstallFolder', '2'),
    @('PATH',      'regex', $appPaths,                  'Path',          '2'),
    @('EXEFOLDER', 'regex', $appPaths,                  '',              '1'),
    @('EXE',       'regex', $appPaths,                  '',              '1'))
  foreach ($scopeSpec in @(@('U', '1', 'CRAFT_USER_DIR'), @('M', '2', 'CRAFT_MACHINE_DIR'))) {
    $letter = $scopeSpec[0]; $target = $scopeSpec[2]
    foreach ($item in $searchContract) {
      $property = "CRAFT_${letter}_$($item[0])"
      $locator = if ($appSearch.ContainsKey($property)) { $locators[$appSearch[$property]] } else { $null }
      $keyOk = $null -ne $locator -and $(if ($item[1] -eq 'exact') { $locator[2] -eq $item[2] } else { $locator[2] -match $item[2] })
      if (-not $keyOk -or $locator[1] -ne $scopeSpec[1] -or $locator[3] -ne $item[3] -or ([int]$locator[4] -band 3) -ne [int]$item[4]) {
        throw "Compiled search for $property is missing or points at the wrong registry location."
      }
    }
  }
  foreach ($route in @(@('WelcomeDlg','Next','InstallDirDlg'),@('InstallDirDlg','Back','WelcomeDlg'))) {
    $dialog = $route[0]; $control = $route[1]
    $routes = @(Read-Rows $database "SELECT ``Argument``, ``Condition`` FROM ``ControlEvent`` WHERE ``Dialog_`` = '$dialog' AND ``Control_`` = '$control' AND ``Event`` = 'NewDialog' ORDER BY ``Ordering``" 2)
    Set-SessionProperty $session 'Installed' ''
    $destination = ''
    foreach ($event in $routes) { if ($session.EvaluateCondition($event[1]) -eq 1) { $destination = $event[0] } }
    if ($destination -ne $route[2]) { throw 'Skipping the licence page must work in both navigation directions.' }
  }
  $registry = @(Read-Rows $database 'SELECT `Root`, `Key`, `Name`, `Value` FROM `Registry`' 4)
  if (@($registry | Where-Object { $_[0] -in '1', '2' }).Count) { throw 'Registration must follow the selected context, not force HKCU or HKLM.' }
  foreach ($entry in $policy.registry) {
    $found = @($registry | Where-Object { $_[0] -eq $entry[0] -and $_[1] -eq $entry[1] -and $_[2] -eq $entry[2] -and $_[3] -eq $entry[3] })
    if ($found.Count -ne 1) { throw "Lost or changed registration: $($entry[1]) / $($entry[2])" }
  }
  $installDir = @($registry | Where-Object { $_[0] -eq '-1' -and $_[1] -eq "Software\$($policy.name)" -and $_[2] -eq 'InstallDir' -and $_[3] -eq '[INSTALLFOLDER]' })
  if ($installDir.Count -ne 1) { throw 'A context-sensitive installation-folder record is required.' }
  Write-Output "PASS $($policy.name): $cases scope cases, folder preservation, action sequencing and registry contract. No installation performed."
} finally {
  foreach ($com in @($session, $database, $installer)) {
    if ($null -ne $com) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($com) }
  }
}
