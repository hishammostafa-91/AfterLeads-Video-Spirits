# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
#Requires -Version 7.2
<#
.SYNOPSIS
    Activates the repository git hooks (.githooks) for this clone (docs/10-execution.md §10.4.13).
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'

$root = git rev-parse --show-toplevel
if ($LASTEXITCODE -ne 0) { throw 'Run this script inside the repository.' }
Set-Location $root

git config core.hooksPath .githooks
if ($LASTEXITCODE -ne 0) { throw 'git config failed.' }

$value = git config --get core.hooksPath
Write-Host "Git hooks active: core.hooksPath = $value"
