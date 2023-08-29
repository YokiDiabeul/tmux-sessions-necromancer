mod command;
mod window;

pub use command::{restore, save};

pub const FILE_NAME: &str = ".tmux-session-rust";
pub const EXCLUDED: [&str; 3] = ["SPOTIFY", "WIKI", "VPN"];
pub const KNOWN_CMDS: [&str; 1] = ["nvim"];
