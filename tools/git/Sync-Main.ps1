# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
#Requires -Version 7.2
<#
.SYNOPSIS
    Merges the latest origin/main into the current work branch with a signed-off merge commit
    (docs/10-execution.md §10.4.6). Never rebases.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'

$root = git rev-parse --show-toplevel
if ($LASTEXITCODE -ne 0) { throw 'Run this script inside the repository.' }
Set-Location $root

$branch = git rev-parse --abbrev-ref HEAD
if ($branch -in @('main', 'master', 'HEAD')) {
    throw "Run this on a work branch, not '$branch'."
}

$dirty = git status --porcelain
if ($dirty) { throw "Commit your changes before syncing:`n$($dirty -join "`n")" }

git fetch origin
if ($LASTEXITCODE -ne 0) { throw 'git fetch failed.' }

git merge --no-ff --signoff --no-edit origin/main
if ($LASTEXITCODE -ne 0) {
    Write-Warning "Merge conflicts. Resolve the files shown by 'git status', run 'pwsh -File tools/verify.ps1', then 'git add -A' and 'git commit -s --no-edit'."
    exit 1
}
Write-Host "'$branch' now contains origin/main."
