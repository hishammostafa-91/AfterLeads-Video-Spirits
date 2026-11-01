# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
#Requires -Version 7.2
<#
.SYNOPSIS
    Verifies the Windows 11 workstation for AfterLeads-Video-Spirits (docs/10-execution.md §10.2).
    Exit code 0 when every required check passes.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Continue'
$results = [System.Collections.Generic.List[object]]::new()

function Add-Result([string]$Name, [bool]$Ok, [string]$Detail, [bool]$Required = $true) {
    $state = if ($Ok) { 'PASS' } elseif ($Required) { 'FAIL' } else { 'WARN' }
    $results.Add([pscustomobject]@{ Check = $Name; Result = $state; Detail = $Detail })
}

function Get-Version([string]$Command, [string[]]$Arguments) {
    if (-not (Get-Command $Command -ErrorAction SilentlyContinue)) { return $null }
    $out = & $Command @Arguments 2>&1 | Select-Object -First 1
    return "$out".Trim()
}

function Get-Major([string]$Text) {
    $m = [regex]::Match("$Text", '(\d+)\.(\d+)')
    if ($m.Success) { return [int]$m.Groups[1].Value } else { return -1 }
}

# Operating system
$os = Get-CimInstance Win32_OperatingSystem -ErrorAction SilentlyContinue
$build = if ($os) { [int]$os.BuildNumber } else { 0 }
Add-Result 'Windows 11 (build >= 22621)' ($build -ge 22621) "build $build"

$fs = Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem' -ErrorAction SilentlyContinue
Add-Result 'Long paths enabled' ($fs -and $fs.LongPathsEnabled -eq 1) 'LongPathsEnabled registry value'

$wv = Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue
if (-not $wv) { $wv = Get-ItemProperty 'HKCU:\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue }
Add-Result 'WebView2 Runtime' ([bool]$wv) $(if ($wv) { "version $($wv.pv)" } else { 'not found' })

# Tools
$psv = $PSVersionTable.PSVersion
Add-Result 'PowerShell 7.2+' (($psv.Major -gt 7) -or ($psv.Major -eq 7 -and $psv.Minor -ge 2)) "$psv"
$git = Get-Version git @('--version'); Add-Result 'Git' ([bool]$git) "$git"
$rustup = Get-Version rustup @('--version'); Add-Result 'rustup' ([bool]$rustup) "$rustup"
$cargo = Get-Version cargo @('--version'); Add-Result 'cargo' ([bool]$cargo) "$cargo"

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$msvc = $null
if (Test-Path $vswhere) { $msvc = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath }
Add-Result 'MSVC C++ build tools' ([bool]$msvc) "$msvc"

$libclang = $env:LIBCLANG_PATH
if (-not $libclang) { $libclang = Join-Path $env:ProgramFiles 'LLVM\bin' }
Add-Result 'libclang (LLVM)' (Test-Path (Join-Path $libclang 'libclang.dll')) $libclang

$node = Get-Version node @('--version'); Add-Result 'Node.js >= 22.12 (baseline 26 LTS)' ((Get-Major $node) -ge 22) "$node"
Add-Result 'Node.js is the 26 LTS line' ((Get-Major $node) -eq 26) "$node" $false
$pnpm = Get-Version pnpm @('--version'); Add-Result 'pnpm' ([bool]$pnpm) "$pnpm"
$python = Get-Version python @('--version'); Add-Result 'Python 3.11+' ([bool]($python -match 'Python 3\.(1[1-9]|[2-9]\d)')) "$python"
$uv = Get-Version uv @('--version'); Add-Result 'uv' ([bool]$uv) "$uv"
$gh = Get-Version gh @('--version'); Add-Result 'GitHub CLI' ([bool]$gh) "$gh"
$cmake = Get-Version cmake @('--version'); Add-Result 'CMake' ([bool]$cmake) "$cmake"
$ninja = Get-Version ninja @('--version'); Add-Result 'Ninja' ([bool]$ninja) "$ninja"

$vcpkgExe = if ($env:VCPKG_ROOT) { Join-Path $env:VCPKG_ROOT 'vcpkg.exe' } else { '' }
Add-Result 'vcpkg (VCPKG_ROOT)' ($vcpkgExe -and (Test-Path $vcpkgExe)) $(if ($env:VCPKG_ROOT) { $env:VCPKG_ROOT } else { 'VCPKG_ROOT not set' })

$opencode = Get-Version opencode @('--version'); Add-Result 'OpenCode 2' ([bool]($opencode -and (Get-Major $opencode) -ge 2)) "$opencode"
$specify = Get-Version specify @('version'); Add-Result 'Spec Kit (specify)' ([bool]$specify) "$specify"

foreach ($tool in 'cargo-nextest', 'cargo-deny', 'cargo-llvm-cov') {
    Add-Result $tool ([bool](Get-Command $tool -ErrorAction SilentlyContinue)) 'install with tools/setup/Install-DevTools.ps1' $false
}

# Git configuration
$name = git config --get user.name; $email = git config --get user.email
Add-Result 'git user.name and user.email' ([bool]$name -and [bool]$email) "$name <$email>"
Add-Result 'git merge.ff = false' ((git config --get merge.ff) -eq 'false') 'git config --global merge.ff false' $false
Add-Result 'git pull.ff = only' ((git config --get pull.ff) -eq 'only') 'git config --global pull.ff only' $false
$inRepo = git rev-parse --is-inside-work-tree 2>$null
if ($inRepo -eq 'true') {
    Add-Result 'Repository hooks active' ((git config --get core.hooksPath) -eq '.githooks') 'pwsh -File tools/git/Install-Hooks.ps1'
}

$ghAuth = $false
if ($gh) { gh auth status *> $null; $ghAuth = ($LASTEXITCODE -eq 0) }
Add-Result 'GitHub CLI logged in' $ghAuth 'gh auth login' $false

$results | Format-Table -AutoSize | Out-String | Write-Host
$failed = @($results | Where-Object Result -eq 'FAIL').Count
if ($failed -gt 0) {
    Write-Host "$failed required check(s) failed. Fix them (docs/10-execution.md §10.2) and re-run." -ForegroundColor Red
    exit 1
}
Write-Host 'Workstation ready.' -ForegroundColor Green
exit 0
