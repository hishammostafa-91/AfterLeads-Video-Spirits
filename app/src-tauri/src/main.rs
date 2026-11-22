// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>
// Prevents an additional console window on Windows in release builds; do not remove.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! The single executable's entry (02 s2.3): the mode is parsed
//! BEFORE any Tauri initialization. WP-05 implements the GUI branch;
//! the worker, crash-server, and CLI branches stay with the work
//! packages that own them (WP-07, WP-22, and the CLI WP) - a launch
//! in those modes reports exactly that and exits 2 instead of
//! inventing behavior.

use afterleads_video_spirits_lib::gui;
use afterleads_video_spirits_lib::state::Boot;

fn main() {
    let mode = parse_mode(std::env::args().collect());
    match mode {
        Mode::CrashServer => not_yet_shipped("--crash-server", "WP-22"),
        Mode::Worker => not_yet_shipped("--worker", "WP-07"),
        Mode::Cli => {
            attach_parent_console();
            not_yet_shipped("cli", "the CLI work package");
        }
        Mode::Gui { safe_mode, path } => {
            // 02 s2.5 step 1: --safe-mode, the Shift key state, and a
            // file path argument all feed the GUI boot.
            let mut boot = Boot::from_flags(safe_mode || shift_held(), path);
            if let Err(error) = gui::run(&mut boot) {
                eprintln!("gui startup failed: {error}");
                std::process::exit(1);
            }
        }
    }
}

/// The parsed launch mode (02 s2.3's dispatch tree, testable pure).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    CrashServer,
    Worker,
    Cli,
    Gui {
        safe_mode: bool,
        path: Option<String>,
    },
}

/// Map raw argv (including argv[0]) onto the mode.
pub fn parse_mode(args: Vec<String>) -> Mode {
    let mut rest = args.into_iter().skip(1).peekable();
    let head = rest.peek().map(String::as_str);
    match head {
        Some("--crash-server") => Mode::CrashServer,
        Some("--worker") => Mode::Worker,
        Some("cli") => Mode::Cli,
        _ => {
            let mut safe_mode = false;
            let mut path = None;
            for arg in rest {
                match arg.as_str() {
                    "--safe-mode" => safe_mode = true,
                    other if other.ends_with(".alvs") => path = Some(other.to_owned()),
                    _ => {}
                }
            }
            Mode::Gui { safe_mode, path }
        }
    }
}

/// Report an unshipped mode and exit 2 (distinct from GUI failure).
fn not_yet_shipped(flag: &str, owner: &str) -> ! {
    eprintln!("`{flag}` mode ships with {owner}; this build only runs the GUI.");
    std::process::exit(2);
}

/// CLI mode attaches to the launching console (02 s2.3); the call is
/// harmless in GUI mode where there is nothing to attach to.
fn attach_parent_console() {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn AttachConsole(dw_process_id: usize) -> i32;
        }
        const ATTACH_PARENT_PROCESS: usize = usize::MAX;
        // SAFETY: AttachConsole with the documented parent-process id
        // only binds stdio handles; it has no memory-safety contract
        // to violate and failing (no parent console) is ignorable.
        unsafe {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}

/// Whether Shift is held at launch (02 s2.5 step 1's safe-mode path).
fn shift_held() -> bool {
    #[cfg(windows)]
    {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn GetAsyncKeyState(v_key: i32) -> i16;
        }
        const VK_SHIFT: i32 = 0x10;
        // SAFETY: GetAsyncKeyState only reads the key state; the high
        // bit means "held since the last call" per its documentation.
        let state = unsafe { GetAsyncKeyState(VK_SHIFT) };
        (state as u32 & 0x8000) != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, parse_mode};

    fn gui() -> Mode {
        Mode::Gui {
            safe_mode: false,
            path: None,
        }
    }

    #[test]
    fn dispatches_before_tauri_modes() {
        let argv = |args: &[&str]| {
            std::iter::once("exe".to_owned())
                .chain(args.iter().map(|s| (*s).to_owned()))
                .collect()
        };
        assert_eq!(
            parse_mode(argv(&["--crash-server", "id1"])),
            Mode::CrashServer
        );
        assert_eq!(parse_mode(argv(&["--worker", "--job", "j"])), Mode::Worker);
        assert_eq!(parse_mode(argv(&["cli", "probe"])), Mode::Cli);
        assert_eq!(parse_mode(argv(&[])), gui());
        assert_eq!(
            parse_mode(argv(&["--safe-mode"])),
            Mode::Gui {
                safe_mode: true,
                path: None
            }
        );
        assert_eq!(
            parse_mode(argv(&["C:\\cut.alvs", "--safe-mode"])),
            Mode::Gui {
                safe_mode: true,
                path: Some("C:\\cut.alvs".to_owned())
            }
        );
        assert_eq!(
            parse_mode(argv(&["--flag", "note.txt"])),
            Mode::Gui {
                safe_mode: false,
                path: None
            }
        );
    }
}
