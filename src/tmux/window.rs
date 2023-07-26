use std::str::FromStr;

use super::command::TmuxCommand;
use super::error::*;

#[derive(Debug)]
pub struct Window {
    session: String,
    name: String,
    path: String,
    layout: String,
}

impl Window {
    pub fn restore(&self) -> Result<()> {
        if self.exists()? {
            self.add()
        } else {
            self.new_session()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn add(&self) -> Result<()> {
        TmuxCommand::new()
            .with_args(&[
                "neww",
                "-d",
                "-t",
                &self.session,
                "-n",
                &self.name,
                "-c",
                &self.path,
            ])
            .execute()?;
        Ok(())
    }

    fn new_session(&self) -> Result<()> {
        TmuxCommand::new()
            .with_args(&[
                "new",
                "-d",
                "-s",
                &self.session,
                "-n",
                &self.name,
                "-c",
                &self.path,
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
        let splitted: Vec<&str> = s.split('\t').collect();
        if splitted.len() != 4 {
            return Err(TmuxError::WindowParsing(s.into()));
        }
        Ok(Window {
            session: splitted[0].to_string(),
            name: splitted[1].to_string(),
            path: splitted[2].to_string(),
            layout: splitted[3].to_string(),
        })
    }
}
