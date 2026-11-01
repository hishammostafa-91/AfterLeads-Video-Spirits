# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
#Requires -Version 7.2
<#
.SYNOPSIS
    Installs the Windows 11 development tools for AfterLeads-Video-Spirits (docs/10-execution.md §10.2).
.DESCRIPTION
    Installs missing tools with winget, the Visual Studio 2022 Build Tools C++ workload, and pinned
    cargo tools. Safe to re-run: present tools are skipped. Run from an elevated PowerShell 7 window.
.PARAMETER SkipCargoTools
    Do not install the cargo-based tools (they compile from source and take a while).
.PARAMETER DryRun
    Only print what would be installed.
#>
[CmdletBinding()]
param(
    [switch]$SkipCargoTools,
    [switch]$DryRun
)
$ErrorActionPreference = 'Stop'

if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    throw 'winget is missing. Install "App Installer" from the Microsoft Store, then re-run.'
}

$packages = @(
    @{ Id = 'Git.Git';                Probe = 'git' },
    @{ Id = 'Microsoft.PowerShell';   Probe = 'pwsh' },
    @{ Id = 'Rustlang.Rustup';        Probe = 'rustup' },
    @{ Id = 'LLVM.LLVM';              Probe = 'clang' },
    @{ Id = 'OpenJS.NodeJS.LTS';      Probe = 'node' },
    @{ Id = 'pnpm.pnpm';              Probe = 'pnpm' },
    @{ Id = 'Python.Python.3.12';     Probe = 'python' },
    @{ Id = 'astral-sh.uv';           Probe = 'uv' },
    @{ Id = 'GitHub.cli';             Probe = 'gh' },
    @{ Id = 'Kitware.CMake';          Probe = 'cmake' },
    @{ Id = 'Ninja-build.Ninja';      Probe = 'ninja' }
)

foreach ($p in $packages) {
    if (Get-Command $p.Probe -ErrorAction SilentlyContinue) {
        Write-Host ("OK      {0,-28} ('{1}' found)" -f $p.Id, $p.Probe)
        continue
    }
    Write-Host ("INSTALL {0}" -f $p.Id) -ForegroundColor Yellow
    if (-not $DryRun) {
        winget install --id $p.Id --exact --silent --accept-package-agreements --accept-source-agreements
        if ($LASTEXITCODE -ne 0) { Write-Warning "winget could not install $($p.Id) (exit $LASTEXITCODE). Install it manually." }
    }
}

# Visual Studio 2022 Build Tools with the C++ workload (MSVC v143 + Windows SDK).
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$hasMsvc = $false
if (Test-Path $vswhere) {
    $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    $hasMsvc = [bool]$vsPath
}
if ($hasMsvc) {
    Write-Host 'OK      MSVC build tools (C++ workload found)'
} else {
    Write-Host 'INSTALL Microsoft.VisualStudio.2022.BuildTools (C++ workload)' -ForegroundColor Yellow
    if (-not $DryRun) {
        winget install --id Microsoft.VisualStudio.2022.BuildTools --exact --accept-package-agreements --accept-source-agreements `
            --override '--quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'
        if ($LASTEXITCODE -ne 0) { Write-Warning 'Build Tools installation reported an error; open the Visual Studio Installer and add "Desktop development with C++".' }
    }
}

# Pinned cargo tools (versions verified for the 2026-11-01 baseline, docs/00-index.md §0.8).
$cargoTools = @(
    @{ Crate = 'cargo-nextest';   Version = '0.9.150'; Probe = 'cargo-nextest' },
    @{ Crate = 'cargo-deny';      Version = '0.20.2';  Probe = 'cargo-deny' },
    @{ Crate = 'cargo-llvm-cov';  Version = '0.9.1';   Probe = 'cargo-llvm-cov' },
    @{ Crate = 'cargo-about';     Version = '0.9.3';   Probe = 'cargo-about' },
    @{ Crate = 'cargo-cyclonedx'; Version = '0.5.9';   Probe = 'cargo-cyclonedx' },
    @{ Crate = 'git-cliff';       Version = '2.14.2';  Probe = 'git-cliff' },
    @{ Crate = 'tauri-driver';    Version = '2.1.0';   Probe = 'tauri-driver' }
)
if (-not $SkipCargoTools) {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Warning 'cargo is not on PATH yet. Open a new PowerShell 7 window and re-run this script to install cargo tools.'
    } else {
        foreach ($t in $cargoTools) {
            if (Get-Command $t.Probe -ErrorAction SilentlyContinue) {
                Write-Host ("OK      {0,-28} (found)" -f $t.Crate)
                continue
            }
            Write-Host ("INSTALL {0}@{1}" -f $t.Crate, $t.Version) -ForegroundColor Yellow
            if (-not $DryRun) {
                cargo install --locked "$($t.Crate)@$($t.Version)"
                if ($LASTEXITCODE -ne 0) { Write-Warning "cargo install $($t.Crate) failed (exit $LASTEXITCODE)." }
            }
        }
    }
}

Write-Host ''
Write-Host 'Next: open a NEW PowerShell 7 window (PATH changes), then run: pwsh -File tools/setup/Test-DevEnvironment.ps1'
