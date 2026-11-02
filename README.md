# AfterLeads-Video-Spirits

A free, open-source, professional video editor for **Windows 11** — fast, private, and built for every language from day one (English, Arabic with full right-to-left support, French, Spanish, German, and Brazilian Portuguese).

> **Status:** in development. The complete implementation documentation is in [`docs/`](docs/00-index.md); progress is tracked in [`STATUS.md`](STATUS.md). There is no release yet.

## What it will do (version 1.0)

- Projects with multiple sequences, a multitrack timeline, and professional trimming (ripple, roll, slip, slide).
- GPU-accelerated effects with keyframes, masks, chroma/luma keying, LUTs, 24 blend modes, and transitions.
- Titles with animation presets and templates, including correct Arabic text shaping.
- Color correction with wheels, curves, and scopes (waveform, parade, vectorscope, histogram).
- Audio mixer, audio effects, loudness measurement and normalization, voice-over recording, auto-ducking.
- Captions: import/export SRT, WebVTT, ASS; styling; burn-in; **local speech-to-text** for automatic captions.
- Background proxies, a render queue, hardware encoders (NVIDIA, AMD, Intel), and verified exports.
- No account, no telemetry, no cloud: everything runs on your computer.

## System requirements

Windows 11 x64 (22H2 or newer), a CPU with AVX2, 8 GB RAM (16 GB recommended), any GPU with a Direct3D 12 driver (a software fallback exists).

## Documentation

| Start here | Purpose |
|---|---|
| [docs/00-index.md](docs/00-index.md) | Master index, global rules, identity, technology baseline |
| [docs/01-product.md](docs/01-product.md) | Vision, scope, and every requirement |
| [docs/10-execution.md](docs/10-execution.md) | Workstation setup, git protocol, work packages |
| [AGENTS.md](AGENTS.md) | Operating manual for the AI coding agents that build the product |

## Building from source

Follow [docs/10-execution.md §10.2](docs/10-execution.md#102-windows-11-workstation-setup). In short: install Git and PowerShell 7 with winget, clone the repository, install the hooks with `tools/git/Install-Hooks.ps1`, install the remaining tools with `tools/setup/Install-DevTools.ps1`, verify with `tools/setup/Test-DevEnvironment.ps1`, then follow the work packages.

## Contributing

Improvements are welcome **in this repository**: code, translations, title templates, LUT looks, documentation, testing, and bug reports. Read [CONTRIBUTING.md](CONTRIBUTING.md). All contributions are signed off under the Developer Certificate of Origin.

## License

AfterLeads-Video-Spirits is licensed under the **GNU Affero General Public License v3.0 or later** — see [LICENSE](LICENSE). Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>.

## Trademarks

"AfterLeads" and "AfterLeads-Video-Spirits" and their logos are trademarks of AfterLeads. The license grants rights to the code, not to the names or logos; see [TRADEMARKS.md](TRADEMARKS.md).

## Security

Please report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).
