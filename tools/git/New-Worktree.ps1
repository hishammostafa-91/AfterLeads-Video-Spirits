# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
#Requires -Version 7.2
<#
.SYNOPSIS
    Creates a sibling worktree on a new branch from origin/main for parallel work
    (docs/10-execution.md §10.4.10, docs/06-ai-automation.md §6.15).
.EXAMPLE
    pwsh -File tools/git/New-Worktree.ps1 -Type feat -Number 17 -Slug titles-generators
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

$root = git rev-parse --show-toplevel
if ($LASTEXITCODE -ne 0) { throw 'Run this script inside the repository.' }

$name = '{0}/{1:D3}-{2}' -f $Type, $Number, $Slug
$parent = Split-Path -Parent $root
$container = Join-Path $parent 'AfterLeads-Video-Spirits.worktrees'
$path = Join-Path $container ($name -replace '/', '-')

if (Test-Path $path) { throw "Worktree folder already exists: $path" }
New-Item -ItemType Directory -Force -Path $container | Out-Null

git -C $root fetch origin
if ($LASTEXITCODE -ne 0) { throw 'git fetch failed.' }

git -C $root worktree add --no-track -b $name $path origin/main
if ($LASTEXITCODE -ne 0) { throw 'git worktree add failed.' }

Write-Host "Worktree ready: $path (branch '$name' from origin/main)."
Write-Host "Open a new terminal there and start OpenCode: Set-Location '$path'; opencode"
