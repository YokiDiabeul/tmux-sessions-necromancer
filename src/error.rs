use thiserror::Error;

#[derive(Error, Debug)]
pub enum TmuxError {
    #[error("Command failed")]
    Failed,
    #[error("No argument passed to the command")]
    NoArgs,
    #[error("Parsing window failed with {0}")]
    WindowParsing(String),
    #[error("Parsing layout failed with {0}")]
    LayoutParsing(String),
    #[error("Parsing pane failed with {0}")]
    PaneParsing(String),
    #[error("Unable to open the file {0}")]
    OpenFile(String),
    #[error("Unable to write to the file {0}")]
    WriteFile(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Parsing(#[from] std::str::Utf8Error),
    #[error(transparent)]
    IntParsing(#[from] std::num::ParseIntError),
}
