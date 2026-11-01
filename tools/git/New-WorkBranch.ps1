# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
#Requires -Version 7.2
<#
.SYNOPSIS
    Creates a work branch <type>/<NNN>-<slug> from an up-to-date main (docs/10-execution.md §10.4.3).
.EXAMPLE
    pwsh -File tools/git/New-WorkBranch.ps1 -Type feat -Number 12 -Slug editing-engine
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateSet('feat', 'fix', 'docs', 'chore', 'ci', 'refactor', 'test', 'perf', 'i18n', 'build')]
    [string]$Type,

    [Parameter(Mandatory)]
    [ValidateRange(0, 999999)]
    [int]$Number,

    [Parameter(Mandatory)]
    [ValidatePattern('^[a-z0-9]+(-[a-z0-9]+)*$')]
    [string]$Slug
)
$ErrorActionPreference = 'Stop'

# Simple function on purpose: $args keeps git flags such as -c or --ff-only untouched.
function Invoke-Git {
    & git @args
    if ($LASTEXITCODE -ne 0) { throw "git $($args -join ' ') failed with exit code $LASTEXITCODE." }
}

$root = git rev-parse --show-toplevel
if ($LASTEXITCODE -ne 0) { throw 'Run this script inside the repository.' }
Set-Location $root

$dirty = git status --porcelain
if ($dirty) {
    throw "Uncommitted changes are present. Commit them on their own branch first:`n$($dirty -join "`n")"
}

$name = '{0}/{1:D3}-{2}' -f $Type, $Number, $Slug
$existing = git branch --list $name
if ($existing) { throw "Branch '$name' already exists locally." }

Invoke-Git fetch origin
Invoke-Git switch main
Invoke-Git pull --ff-only origin main
Invoke-Git switch -c $name

$head = git rev-parse --short HEAD
Write-Host "Created '$name' from main at $head. Next: run the Spec Kit specify step (docs/06 §6.11)."
