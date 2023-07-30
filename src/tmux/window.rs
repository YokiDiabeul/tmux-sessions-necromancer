use std::str::FromStr;

use super::command::{TmuxCommand, PANE_SPLIT, WINDOW_SPLIT};
use crate::prelude::*;

#[derive(Debug, Clone)]
struct Layout {
    up: bool,
    left: bool,
    right: bool,
    down: bool,
}

impl Layout {
    fn is_horizontal(&self) -> bool {
        !self.left
    }
}

impl FromStr for Layout {
    type Err = TmuxError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let layout: Vec<char> = s.chars().collect();
        if layout.len() != 4 {
            return Err(TmuxError::LayoutParsing(s.into()));
        }
        Ok(Layout {
            left: layout[0] as u8 == 49,
            up: layout[1] as u8 == 49,
            right: layout[2] as u8 == 49,
            down: layout[3] as u8 == 49,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Pane {
    path: String,
    layout: Layout,
}

impl Pane {
    pub fn split(&self, session: &str, window: &str) -> Result<()> {
        self.split_window(self.layout.is_horizontal(), session, window)?;
        Ok(())
    }

    fn split_window(&self, horizontal: bool, session: &str, window: &str) -> Result<()> {
        let split = if horizontal { "-h" } else { "-v" };
        TmuxCommand::new()
            .with_args(&["splitw", split])
            .with_args(&["-c", &self.path])
            .with_args(&["-t", &f!("{session}:{window}")])
            .execute()?;
        Ok(())
    }
}

impl FromStr for Pane {
    type Err = TmuxError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let splitted: Vec<&str> = s.split(PANE_SPLIT).collect();
        if splitted.len() != 2 {
            return Err(TmuxError::PaneParsing(s.into()));
        }
        Ok(Pane {
            path: splitted[0].to_string(),
            layout: splitted[1].parse()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Window {
    session: String,
    name: String,
    panes: Vec<Pane>,
}

impl Window {
    pub fn restore(&self) -> Result<()> {
        for (i, pane) in self.panes.iter().enumerate() {
            match (i, pane) {
                (0, pane) if !self.exists()? => self.new_session(pane)?,
                (0, pane) => self.add(pane)?,
                (_, pane) => pane.split(&self.session, &self.name)?,
            }
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_pane(&mut self, pane: Pane) {
        self.panes.push(pane);
    }

    pub fn panes(&self) -> &Vec<Pane> {
        &self.panes
    }

    fn add(&self, pane: &Pane) -> Result<()> {
        TmuxCommand::new()
            .with_args(&[
                "neww",
                "-d",
                "-t",
                &self.session,
                "-n",
                &self.name,
                "-c",
                &pane.path,
            ])
            .execute()?;
        Ok(())
    }

    fn new_session(&self, pane: &Pane) -> Result<()> {
        TmuxCommand::new()
            .with_args(&[
                "new",
                "-d",
                "-s",
                &self.session,
                "-n",
                &self.name,
                "-c",
                &pane.path,
            ])
            .execute()?;
        Ok(())
    }

    fn exists(&self) -> Result<bool> {
        Ok(TmuxCommand::new()
            .with_args(&["has", "-t", &self.session])
            .status()?
            .success())
    }
}

impl FromStr for Window {
    type Err = TmuxError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let splitted: Vec<&str> = s.split(WINDOW_SPLIT).collect();
        if splitted.len() != 3 {
            return Err(TmuxError::WindowParsing(s.into()));
        }
        Ok(Window {
            session: splitted[0].to_string(),
            name: splitted[1].to_string(),
            panes: vec![splitted[2].parse()?],
        })
    }
}
