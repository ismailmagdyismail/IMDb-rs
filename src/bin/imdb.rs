use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    imdb::core::operations::imdb_launcher::launch_imdb(args).unwrap_or_else(|error| {
        eprintln!("{}", error);
        std::process::exit(1);
    });
}
