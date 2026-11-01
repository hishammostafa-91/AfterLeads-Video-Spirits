---
description: Run all local quality gates (tools/verify.ps1) and summarize the results.
agent: build
---
Run `pwsh -File tools/verify.ps1`.

Report one line per gate: PASS, FAIL, or SKIPPED (with the reason). For each failure show the first error with file:line and the most likely cause, and propose the smallest fix. Do not edit files in this command unless the owner asks you to fix them.
