# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
#Requires -Version 7.2
<#
.SYNOPSIS
    Runs the local quality gates (docs/08-operations.md §8.4). Gates whose targets do not exist yet
    are reported as SKIPPED. Exit code 1 when any gate fails.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'

$root = git rev-parse --show-toplevel
if ($LASTEXITCODE -ne 0) { throw 'Run this script inside the repository.' }
Set-Location $root

$results = [System.Collections.Generic.List[object]]::new()

function Test-Tool([string]$Name) { [bool](Get-Command $Name -ErrorAction SilentlyContinue) }

function Test-AppScript([string]$Name) {
    if (-not (Test-Path 'app/package.json')) { return $false }
    $pkg = Get-Content 'app/package.json' -Raw | ConvertFrom-Json
    return [bool]($pkg.scripts -and $pkg.scripts.PSObject.Properties.Name -contains $Name)
}

function Invoke-Gate([string]$Name, [bool]$Enabled, [string]$SkipReason, [scriptblock]$Run) {
    if (-not $Enabled) {
        $results.Add([pscustomobject]@{ Gate = $Name; Result = 'SKIPPED'; Detail = $SkipReason })
        return
    }
    Write-Host "==> $Name" -ForegroundColor Cyan
    $global:LASTEXITCODE = 0
    $code = 0
    try {
        & $Run
        $code = $LASTEXITCODE
    } catch {
        Write-Host $_ -ForegroundColor Red
        $code = 1
    }
    $state = if ($code -eq 0) { 'PASS' } else { 'FAIL' }
    $detail = if ($code -eq 0) { '' } else { "exit code $code" }
    $results.Add([pscustomobject]@{ Gate = $Name; Result = $state; Detail = $detail })
}

$hasCargo = (Test-Path 'Cargo.toml') -and (Test-Tool 'cargo')
$hasMain = [bool](git rev-parse --verify --quiet origin/main)

Invoke-Gate 'docs' (Test-Path 'tools/docs/check_docs.py') 'checker missing' { python tools/docs/check_docs.py }
Invoke-Gate 'commits' $hasMain 'origin/main not available' { python tools/git/check_commits.py origin/main..HEAD }
Invoke-Gate 'rust-format' $hasCargo 'no Cargo workspace yet' { cargo fmt --all -- --check }
Invoke-Gate 'rust-lint' $hasCargo 'no Cargo workspace yet' { cargo clippy --workspace --all-targets --locked -- -D warnings }
Invoke-Gate 'dependency-policy' ($hasCargo -and (Test-Path 'deny.toml') -and (Test-Tool 'cargo-deny')) 'deny.toml or cargo-deny missing' { cargo deny check }
Invoke-Gate 'crate-rules' (Test-Path 'tools/check_deps.py') 'created in WP-01' { python tools/check_deps.py }
Invoke-Gate 'placeholders' (Test-Path 'tools/check_placeholders.py') 'created in WP-01' { python tools/check_placeholders.py }
Invoke-Gate 'rust-test' $hasCargo 'no Cargo workspace yet' { cargo test --workspace --locked }
Invoke-Gate 'bindings-drift' ($hasCargo -and (Test-Path 'app/src/ipc/bindings')) 'no bindings yet' {
    cargo test -p alvs-model export_bindings --locked
    if ($LASTEXITCODE -eq 0) { git diff --exit-code -- app/src/ipc/bindings }
}
Invoke-Gate 'ui-typecheck' (Test-AppScript 'typecheck') 'script not defined yet' { pnpm -C app typecheck }
Invoke-Gate 'ui-lint' (Test-AppScript 'lint') 'script not defined yet' { pnpm -C app lint }
Invoke-Gate 'ui-test' (Test-AppScript 'test') 'script not defined yet' { pnpm -C app test }
Invoke-Gate 'i18n' (Test-AppScript 'i18n:check') 'script not defined yet' { pnpm -C app i18n:check }

$results | Format-Table -AutoSize | Out-String | Write-Host
$failed = @($results | Where-Object Result -eq 'FAIL').Count
if ($failed -gt 0) {
    Write-Host "$failed gate(s) failed." -ForegroundColor Red
    exit 1
}
Write-Host 'All available gates passed.' -ForegroundColor Green
exit 0
