use std::env;

// fn load(db_file_path: &str) -> Result<(), String> {
//     //! load db-file if it exists and set up in memory data structurs
//     Ok(())
// }

fn main() {
    let args: Vec<String> = env::args().collect();
    imdb::core::imdb_launcher::launch_imdb(args).unwrap_or_else(|error| {
        eprintln!("{}", error);
        std::process::exit(1);
    });
}
