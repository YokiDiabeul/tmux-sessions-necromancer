use std::process::{Command, ExitStatus, Output, Stdio};
use std::str::from_utf8;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

use super::error::*;
use super::window::Window;

const TMUX_CMD: &str = "tmux";
const FILE_NAME: &str = "/home/yoki/.tmux-session-rust";

pub const PANE_SPLIT: &str = " - ";
pub const WINDOW_SPLIT: char = '\t';

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

    let wins: Vec<Window> = BufReader::new(f)
        .lines()
        .flatten()
        .filter_map(|l| l.parse::<Window>().ok())
        .collect();

    Ok(merge_windows(wins)
        .iter()
        .map(|w| w.restore())
        .collect::<Result<Vec<_>>>()?
        .len())
}

// Didn't find any more elegent way of doing it, it created windows and then merge them, instead
// of creating the right one dirrectly
fn merge_windows(windows: Vec<Window>) -> Vec<Window> {
    let mut window_map: HashMap<String, Window> = HashMap::new();

    for window in windows {
        if let Some(entry) = window_map.get_mut(window.name()) {
            for pane in window.panes() {
                entry.add_pane(pane.clone());
            }
        } else {
            window_map.insert(window.name().to_string(), window.clone());
        }
    }

    window_map.values().cloned().collect()
}

fn current_state() -> Result<String> {
    let state_format = format!("#S{WINDOW_SPLIT}#W{WINDOW_SPLIT}#{{pane_current_path}}{PANE_SPLIT}#{{pane_height}}{PANE_SPLIT}#{{pane_width}}{PANE_SPLIT}#{{pane_at_left}}#{{pane_at_top}}#{{pane_at_right}}#{{pane_at_bottom}}");
    let out = TmuxCommand::new()
        .with_args(&["lsp", "-a", "-F", &state_format])
        .execute()?;
    Ok(from_utf8(&out.stdout)?.to_string())
}
