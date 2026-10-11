use std::{env, io, process::Command};

const ON: &str =
    r#"{"text":"DND: on","class":"on","tooltip":"Notifications paused — click to enable"}"#;
const OFF: &str =
    r#"{"text":"DND: off","class":"off","tooltip":"Notifications on — click to pause (DND)"}"#;
const HELP: &str = "waybar-dnd — dnd switch for waybar

usage: waybar-dnd <command>

status    print json for waybar
toggle    toggle dunst's notification pause

requires dunstctl on your path
";

fn main() {
    if let Err(error) = run() {
        eprintln!("waybar-dnd: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let command = args.next().ok_or(HELP)?;
    if args.next().is_some() {
        return Err(format!("unexpected arguments\n{HELP}"));
    }
    match command.to_str() {
        Some("status") => {
            let paused = match is_paused() {
                Ok(paused) => paused,
                Err(error) => {
                    eprintln!("waybar-dnd: {error}");
                    false
                }
            };
            println!("{}", if paused { ON } else { OFF });
        }
        Some("toggle") => {
            let status = Command::new("dunstctl")
                .args(["set-paused", "toggle"])
                .status()
                .map_err(|error| command_error("set-paused toggle", error))?;
            if !status.success() {
                return Err(format!("dunstctl set-paused toggle exited with {status}"));
            }
        }
        Some("--help" | "-h") => print!("{HELP}"),
        Some("--version" | "-V") => {
            println!("waybar-dnd {}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            return Err(format!(
                "unknown command: {}\n{HELP}",
                command.to_string_lossy()
            ));
        }
    }
    Ok(())
}

fn is_paused() -> Result<bool, String> {
    let output = Command::new("dunstctl")
        .arg("is-paused")
        .output()
        .map_err(|error| command_error("is-paused", error))?;
    if !output.status.success() {
        return Err(format!(
            "dunstctl is-paused exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    // A successful command may still have emitted a useful diagnostic.
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    match output.stdout.as_slice() {
        b"true" | b"true\n" => Ok(true),
        b"false" | b"false\n" => Ok(false),
        _ => Err("dunstctl is-paused returned an unknown response".into()),
    }
}

fn command_error(action: &str, error: io::Error) -> String {
    format!("could not run dunstctl {action}: {error}")
}
