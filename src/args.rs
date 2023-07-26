use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Action {
    Save,
    Restore,
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct TmuxSessionArgs {
    /// Action to do with the tmux sessions
    #[clap(value_enum)]
    pub action: Action,
}
