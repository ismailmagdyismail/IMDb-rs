use std::{
    env,
    io::{BufRead, Write},
};

const USAGE_MESSAGE: &'static str = "
run      >  imdb      [Imdb_FILE_PATH] repl|deamon
run      >  imdb      [Imdb_FILE_PATH] repl|deamon
run      >  imdb      [Imdb_FILE_PATH] repl|deamon
commands >  get       [KEY]
commands >  insert    [KEY] [VALUE]
commands >  delete    [KEY]
";

fn error_message_formatter(error: String) -> String {
    let mut formatted_error_message = String::from("\n\n");
    formatted_error_message.push_str(&error);
    formatted_error_message.push_str("\n");
    formatted_error_message.push_str(USAGE_MESSAGE);
    formatted_error_message
}

#[derive(Debug)]
struct ImdbConfig {
    db_path: String,
}

impl ImdbConfig {
    pub fn new(db_path: String) -> Result<ImdbConfig, String> {
        //! should we validate the path here ? or maybe leave out side parser
        Ok(ImdbConfig { db_path })
    }
}

#[derive(Debug)]
enum ImdbCommand {
    Get,
    Insert,
    Delete,
}

#[derive(Debug)]
enum ImdbOperation {
    Repl,
    Deamon,
}

impl ImdbOperation {
    fn from(operation_arg: &String) -> Result<ImdbOperation, String> {
        let normalized_imdb_operation_arg = operation_arg.to_lowercase();
        let normalized_imdb_operation_arg = normalized_imdb_operation_arg.trim();
        let imdb_operation = match normalized_imdb_operation_arg {
            "deamon" => ImdbOperation::Deamon,
            "repl" => ImdbOperation::Repl,
            _ => {
                return Result::Err(error_message_formatter(
                    "[Imdb Invalid operation]".to_string(),
                ));
            }
        };

        Ok(imdb_operation)
    }
}

struct ImdbCliArgsParser {}

impl ImdbCliArgsParser {
    pub fn parse(mut args: Vec<String>) -> Result<(ImdbConfig, ImdbOperation), String> {
        if args.len() < 2 {
            return Result::Err(error_message_formatter("[Imdb Invalid Args]".to_string()));
        }

        let db_path = args[1].clone(); // TODO: remove clone , take ownership | borrow
        let imdb_config = ImdbConfig::new(db_path)?;
        let imdb_operation = ImdbOperation::from(&args[2])?;
        Ok((imdb_config, imdb_operation))
    }
}

fn load(db_file_path: &str) -> Result<(), String> {
    //! load db-file if it exists and set up in memory data structurs
    Ok(())
}

fn command_parser(command: &String) -> Result<(), String> {
    let command_entries: Vec<&str> = command.split(' ').collect();
    let command_name = command_entries[0];
    let normalized_command_name = command_name.to_lowercase();
    let normalized_command_name = normalized_command_name.trim();
    match normalized_command_name {
        "get" => Ok(()),
        "insert" => Ok(()),
        "delete" => Ok(()),
        _ => {
            return Result::Err(error_message_formatter(
                "[Imdb Invalid Command name]".to_string(),
            ));
        }
    }
}

fn run_repl() {
    loop {
        let mut stdout = std::io::stdout();
        stdout.write_all("Command > ".as_bytes()).unwrap();
        stdout.flush().unwrap();

        let stdin = std::io::stdin().lock();
        for line in stdin.lines() {
            let line = line.unwrap();
            if let Result::Err(error) = command_parser(&line) {
                println!("{}", error);
            }
            break;
        }
    }
}

fn run_db(args: Vec<String>) -> Result<(), String> {
    let (imdb_config, imdb_operation) = ImdbCliArgsParser::parse(args)?;
    dbg!(&imdb_config);
    dbg!(&imdb_operation);

    match imdb_operation {
        ImdbOperation::Deamon => todo!("[Imdb:: Server|Deamon Mode Not Supported Yet!!]"),
        ImdbOperation::Repl => run_repl(),
    }

    // load(file_path)?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    run_db(args).unwrap_or_else(|error| {
        eprintln!("{}", error);
        std::process::exit(1);
    });
}
