#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Session(PathBuf);

impl Session {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "waybar-dnd-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn backend(&self, body: &str) {
        let path = self.0.join("dunstctl");
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_waybar-dnd"));
        command.env("PATH", &self.0).env("DND_TEST_HOME", &self.0);
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn status_preserves_both_existing_waybar_outputs() {
    let session = Session::new();
    for (response, expected) in [
        (
            "true",
            "{\"text\":\"DND: on\",\"class\":\"on\",\"tooltip\":\"Notifications paused — click to enable\"}\n",
        ),
        (
            "false",
            "{\"text\":\"DND: off\",\"class\":\"off\",\"tooltip\":\"Notifications on — click to pause (DND)\"}\n",
        ),
    ] {
        session.backend(&format!(
            "[ \"$#\" = 1 ] && [ \"$1\" = is-paused ] || exit 2\nprintf '%s\\n' '{response}'"
        ));
        let result = session.run(&["status"]);
        assert!(result.status.success());
        assert_eq!(String::from_utf8(result.stdout).unwrap(), expected);
        assert!(result.stderr.is_empty());
    }
}

#[test]
fn toggle_changes_backend_state_and_other_panels_read_it() {
    let session = Session::new();
    session.backend(r#"
case "$*" in
  is-paused)
    if [ -f "$DND_TEST_HOME/paused" ]; then
      read -r paused < "$DND_TEST_HOME/paused"; printf '%s\n' "$paused";
    else printf 'false\n'; fi ;;
  'set-paused toggle')
    paused=false
    if [ -f "$DND_TEST_HOME/paused" ]; then read -r paused < "$DND_TEST_HOME/paused"; fi
    if [ "$paused" = true ]; then printf 'false\n'; else printf 'true\n'; fi > "$DND_TEST_HOME/paused" ;;
  *) exit 2 ;;
esac
"#);
    assert!(String::from_utf8_lossy(&session.run(&["status"]).stdout).contains("DND: off"));
    for state in ["on", "off"] {
        let toggle = session.run(&["toggle"]);
        assert!(toggle.status.success());
        assert!(toggle.stdout.is_empty());
        for _panel in 0..2 {
            assert!(
                String::from_utf8_lossy(&session.run(&["status"]).stdout)
                    .contains(&format!("DND: {state}"))
            );
        }
    }
}

#[test]
fn read_failures_keep_off_output_and_report_the_cause() {
    let session = Session::new();
    for body in [
        None,
        Some("printf 'backend unavailable\\n' >&2; exit 7"),
        Some("printf 'unknown\\n'"),
        Some("printf ' true\\n'"),
    ] {
        if let Some(body) = body {
            session.backend(body);
        }
        let result = session.run(&["status"]);
        assert!(result.status.success());
        assert!(String::from_utf8_lossy(&result.stdout).contains("\"class\":\"off\""));
        assert!(!result.stderr.is_empty());
        if body.is_some_and(|body| body.contains("backend unavailable")) {
            assert!(String::from_utf8_lossy(&result.stderr).contains("backend unavailable"));
        }
    }
}

#[test]
fn toggle_failures_have_a_nonzero_exit_code() {
    let session = Session::new();
    for body in [None, Some("printf 'cannot toggle\\n' >&2; exit 7")] {
        if let Some(body) = body {
            session.backend(body);
        }
        let result = session.run(&["toggle"]);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
}

#[test]
fn invalid_arguments_do_not_call_the_backend() {
    let session = Session::new();
    session.backend("printf 'called' > \"$DND_TEST_HOME/called\"");
    for args in [
        &[][..],
        &["unknown"],
        &["toggle", "extra"],
        &["status", "extra"],
        &["--help", "extra"],
    ] {
        let result = session.run(args);
        assert!(!result.status.success());
        assert!(!session.0.join("called").exists());
    }
}

#[test]
fn help_and_version_work_without_dunst() {
    let session = Session::new();
    for arg in ["--help", "-h", "--version", "-V"] {
        let result = session.run(&[arg]);
        assert!(result.status.success());
        assert!(result.stderr.is_empty());
        let text = String::from_utf8(result.stdout).unwrap();
        if arg == "--version" || arg == "-V" {
            assert_eq!(text, "waybar-dnd 0.1.0\n");
        } else {
            assert!(text.contains("status"));
            assert!(text.contains("toggle"));
        }
    }
}
