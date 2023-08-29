use std::marker::PhantomData;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::str::from_utf8;

use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

use super::window::Window;
use super::{EXCLUDED, FILE_NAME, KNOWN_CMDS};
use crate::prelude::*;

const TMUX_CMD: &str = "tmux";

pub const PANE_SPLIT: &str = " - ";
pub const WINDOW_SPLIT: char = '\t';

#[derive(Default)]
pub struct NoArgs {}

#[derive(Default)]
pub struct WithArgs(Vec<String>);

#[derive(Default)]
pub struct NoCmd {}

#[derive(Default)]
pub struct WithCmd;

#[derive(Default)]
pub struct TmuxCommand<A, B> {
    args: A,
    cmd: PhantomData<B>,
}

impl TmuxCommand<NoArgs, NoCmd> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_arg(&self, arg: &str) -> TmuxCommand<WithArgs, NoCmd> {
        TmuxCommand {
            args: WithArgs(vec![arg.to_string()]),
            cmd: PhantomData,
        }
    }

    pub fn with_args(&self, args: &[&str]) -> TmuxCommand<WithArgs, NoCmd> {
        TmuxCommand {
            args: WithArgs(args.iter().map(|a| a.to_string()).collect()),
            cmd: PhantomData,
        }
    }
}

impl TmuxCommand<WithArgs, NoCmd> {
    pub fn with_arg(mut self, arg: &str) -> Self {
        self.args.0.push(arg.to_string());
        self
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.args.0.extend(args.iter().map(|arg| arg.to_string()));
        self
    }

    pub fn with_cmd(mut self, cmd: &str) -> TmuxCommand<WithArgs, WithCmd> {
        if KNOWN_CMDS.contains(&cmd) {
            self.args.0.push(cmd.to_string());
        }
        TmuxCommand {
            args: self.args,
            cmd: PhantomData,
        }
    }
}

impl<B> TmuxCommand<WithArgs, B> {
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
    let filename: String = f!("{}/{FILE_NAME}", std::env::var("HOME").unwrap());
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&filename)
        .map_err(|_| TmuxError::OpenFile(filename.clone()))?
        .write_all(current_state()?.as_bytes())
        .map_err(|_| TmuxError::WriteFile(filename.clone()))?;
    Ok(())
}

pub fn restore() -> Result<usize> {
    TmuxCommand::new().with_arg("start").execute()?;

    let filename: String = f!("{}/{FILE_NAME}", std::env::var("HOME").unwrap());
    let f = OpenOptions::new()
        .read(true)
        .open(&filename)
        .map_err(|_| TmuxError::OpenFile(filename))?;

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
        if existing_names.contains(&window.id()) {
            let entry = new_list.get_mut(existing_names.len() - 1).unwrap();
            for pane in window.panes() {
                entry.add_pane(pane.clone());
            }
        } else {
            existing_names.insert(window.id());
            new_list.push(window.clone());
        }
    }
    new_list
}

fn current_state() -> Result<String> {
    let state_format = f!("#S{WINDOW_SPLIT}#W{WINDOW_SPLIT}#{{pane_current_path}}{PANE_SPLIT}#{{pane_current_command}}{PANE_SPLIT}#{{pane_height}}{PANE_SPLIT}#{{pane_width}}{PANE_SPLIT}#{{pane_at_left}}");
    let out = TmuxCommand::new()
        .with_args(&["lsp", "-a", "-F", &state_format])
        .execute()?;
    Ok(from_utf8(&out.stdout)?.to_string())
}
