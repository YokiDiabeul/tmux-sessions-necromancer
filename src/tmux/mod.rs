mod command;
mod window;

pub use command::{restore, save};

pub const FILE_NAME: &str = "/home/yoki/.tmux-session-rust"; //TODO: use relative path
pub const EXCLUDED: [&str; 3] = ["SPOTIFY", "WIKI", "VPN"];
pub const KNOWN_CMDS: [&str; 1] = ["nvim"];
