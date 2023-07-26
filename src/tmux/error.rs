use thiserror::Error;

pub type Result<T> = std::result::Result<T, TmuxError>;

#[derive(Error, Debug)]
pub enum TmuxError {
    #[error("Command failed")]
    Failed,
    #[error("Parsing window failed with {0}")]
    WindowParsing(String),

    #[error("Command init error")]
    Init(#[from] std::io::Error),
    #[error("Cannot parse output to string")]
    Parsing(#[from] std::str::Utf8Error),

    #[error("Unable to open the file")]
    OpenFile(#[source] std::io::Error),
    #[error("Unable to write to the file")]
    WriteFile(#[source] std::io::Error),
}
