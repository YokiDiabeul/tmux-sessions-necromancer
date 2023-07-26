mod args;
mod tmux;

use args::{Action, TmuxSessionArgs};
use clap::Parser;

use tmux::{restore, save};

fn main() {
    match TmuxSessionArgs::parse().action {
        Action::Save => match save() {
            Ok(_) => println!("Session saved"),
            Err(e) => eprintln!("{e}"),
        },
        Action::Restore => match restore() {
            Ok(n) => println!("{n} sessions restored"),
            Err(e) => eprintln!("{e}"),
        },
    }
}
