use std::io::{BufRead, Write};

use crate::core::imdb::Imdb;

pub struct ImdbRepl {
    imdb: Imdb,
}

impl ImdbRepl {
    pub fn new(imdb: Imdb) -> ImdbRepl {
        ImdbRepl { imdb }
    }

    pub fn run(&mut self) {
        loop {
            let mut stdout = std::io::stdout();
            stdout.write_all("Command > ".as_bytes()).unwrap();
            stdout.flush().unwrap();

            let stdin = std::io::stdin().lock();
            for line in stdin.lines() {
                let line = line.unwrap();
                let operation_result = self.imdb.execute_command(line);
                match operation_result {
                    Err(error_message) => {
                        eprintln!("{}", error_message);
                    }
                    Ok(res) => {
                        if let Some(value) = res {
                            println!("{}", value);
                        }
                    }
                }
                break;
            }
        }
    }
}
