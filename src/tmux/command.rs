use std::process::{Command, ExitStatus, Output, Stdio};
use std::str::from_utf8;

use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

use super::window::Window;
use crate::prelude::*;

const TMUX_CMD: &str = "tmux";
const FILE_NAME: &str = "/home/yoki/.tmux-session-rust"; //TODO: use relative path
const EXCLUDED: [&str; 3] = ["SPOTIFY", "WIKI", "VPN"];

pub const PANE_SPLIT: &str = " - ";
pub const WINDOW_SPLIT: char = '\t';

#[derive(Default)]
pub struct NoArgs {}

#[derive(Default)]
pub struct WithArgs(Vec<String>);

#[derive(Default)]
pub struct TmuxCommand<A> {
    args: A,
}

impl TmuxCommand<NoArgs> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_arg(&self, arg: &str) -> TmuxCommand<WithArgs> {
        TmuxCommand {
            args: WithArgs(vec![arg.to_string()]),
        }
    }

    pub fn with_args(&self, args: &[&str]) -> TmuxCommand<WithArgs> {
        TmuxCommand {
            args: WithArgs(args.iter().map(|a| a.to_string()).collect()),
        }
    }
}

impl TmuxCommand<WithArgs> {
    pub fn with_arg(mut self, arg: &str) -> Self {
        self.args.0.push(arg.to_string());
        self
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.args.0.extend(args.iter().map(|arg| arg.to_string()));
        self
    }

    pub fn execute(self) -> Result<Output> {
        let output = Command::new(TMUX_CMD).args(&self.args.0).output()?;
        if !output.status.success() {
            return Err(TmuxError::Failed);
        }

        Ok(output)
    }

    pub fn status(self) -> Result<ExitStatus> {
        Ok(Command::new(TMUX_CMD)
            .args(&self.args.0)
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
        .map_err(|_| TmuxError::OpenFile(FILE_NAME.to_string()))?
        .write_all(current_state()?.as_bytes())
        .map_err(|_| TmuxError::WriteFile(FILE_NAME.to_string()))?;
    Ok(())
}

pub fn restore() -> Result<usize> {
    TmuxCommand::new().with_arg("start").execute()?;

    let f = OpenOptions::new()
        .read(true)
        .open(FILE_NAME)
        .map_err(|_| TmuxError::OpenFile(FILE_NAME.to_string()))?;

    let wins: Vec<Window> = BufReader::new(f)
        .lines()
        .flatten()
        .filter_map(|l| l.parse::<Window>().ok())
        .filter(|w| !EXCLUDED.contains(&w.name()))
        .collect();

    Ok(merge_windows_keeping_order(wins)
        .iter()
        .map(|w| w.restore())
        .collect::<Result<Vec<_>>>()?
        .len())
}

fn merge_windows_keeping_order(windows: Vec<Window>) -> Vec<Window> {
    let mut existing_names: HashSet<String> = HashSet::new();
    let mut new_list: Vec<Window> = Vec::new();

    for window in windows {
        if existing_names.contains(window.name()) {
            let entry = new_list.get_mut(existing_names.len() - 1).unwrap();
            for pane in window.panes() {
                entry.add_pane(pane.clone());
            }
        } else {
            existing_names.insert(window.name().to_string());
            new_list.push(window.clone());
        }
    }
    new_list
}

fn current_state() -> Result<String> {
    let state_format = f!("#S{WINDOW_SPLIT}#W{WINDOW_SPLIT}#{{pane_current_path}}{PANE_SPLIT}#{{pane_at_left}}#{{pane_at_top}}#{{pane_at_right}}#{{pane_at_bottom}}");
    let out = TmuxCommand::new()
        .with_args(&["lsp", "-a", "-F", &state_format])
        .execute()?;
    Ok(from_utf8(&out.stdout)?.to_string())
}
