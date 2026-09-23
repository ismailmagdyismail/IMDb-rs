use std::io::{BufRead, Write};

use crate::core::imdb_commands::ImdbCommand;

pub struct ImdbRepl {}

impl ImdbRepl {
    pub fn run(&self) {
        loop {
            let mut stdout = std::io::stdout();
            stdout.write_all("Command > ".as_bytes()).unwrap();
            stdout.flush().unwrap();

            let stdin = std::io::stdin().lock();
            for line in stdin.lines() {
                let line = line.unwrap();
                if let Result::Err(error) = ImdbCommand::parse(&line) {
                    println!("{}", error);
                }
                break;
            }
        }
    }
}
