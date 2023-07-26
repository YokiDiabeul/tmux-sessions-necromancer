use std::process::{Command, ExitStatus, Output, Stdio};
use std::str::from_utf8;

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

use super::error::*;
use super::window::Window;

const TMUX_CMD: &str = "tmux";
const FILE_NAME: &str = "/home/yoki/.tmux-session-rust";
const EXLUDED: [&str; 4] = ["log", "man", "WIKI", "VPN"];

pub struct TmuxCommand {
    args: Vec<String>,
}

impl TmuxCommand {
    pub fn new() -> Self {
        TmuxCommand { args: Vec::new() }
    }

    pub fn with_arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.args.extend(args.iter().map(|arg| arg.to_string()));
        self
    }

    pub fn execute(&self) -> Result<Output> {
        let output = Command::new(TMUX_CMD).args(&self.args).output()?;

        if !output.status.success() {
            Err(TmuxError::Failed)
        } else {
            Ok(output)
        }
    }

    pub fn status(&self) -> Result<ExitStatus> {
        Ok(Command::new(TMUX_CMD)
            .args(&self.args)
            .stderr(Stdio::null())
            .status()?)
    }
}

pub fn save() -> Result<()> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(FILE_NAME)
        .map_err(TmuxError::OpenFile)?
        .write_all(current_state()?.as_bytes())
        .map_err(TmuxError::WriteFile)?;

    Ok(())
}

pub fn restore() -> Result<usize> {
    TmuxCommand::new().with_arg("start").execute()?;

    let f = OpenOptions::new()
        .read(true)
        .open(FILE_NAME)
        .map_err(TmuxError::OpenFile)?;

    Ok(BufReader::new(f)
        .lines()
        .flatten()
        .filter_map(|l| l.parse::<Window>().ok())
        .filter(|w| !EXLUDED.contains(&w.name()))
        .map(|w| w.restore())
        .collect::<Result<Vec<_>>>()?
        .len())
}

fn current_state() -> Result<String> {
    let out = TmuxCommand::new()
        .with_args(&[
            "lsw",
            "-a",
            "-F",
            "#S\t#W\t#{pane_current_path}\t#{window_layout}",
        ])
        .execute()?;
    Ok(from_utf8(&out.stdout)?.to_string())
}

// fn add_window(session: &str, window_name: &str, path: &str) -> Result<()> {
//     TmuxCommand::new()
//         .with_args(&["neww", "-d", "-t", session, "-n", window_name, "-c", path])
//         .execute()?;
//     Ok(())
// }
//
// fn new_session(session: &str, window_name: &str, path: &str) -> Result<()> {
//     TmuxCommand::new()
//         .with_args(&["new", "-d", "-s", session, "-n", window_name, "-c", path])
//         .execute()?;
//     Ok(())
// }
//
// fn session_exists(name: &str) -> Result<bool> {
//     let out = TmuxCommand::new()
//         .with_args(&["has", "-t", name, "2>/dev/null"])
//         .execute()?;
//     Ok(out.status.success())
// }
