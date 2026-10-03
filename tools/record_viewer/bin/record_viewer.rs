use std::path::Path;

use imdb::core::storage::imdb_inline_metadata_storage_engine::imdb_inline_metadata_pager::ImdbInlineMetaDataPager;

const USAGE: &'static str = "record-viewer inline_metadata|seperate_metadata [Imdb_DB_DIR_PATH]";
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Invalid number of args");
        eprintln!("{}", USAGE);
        std::process::exit(1);
    }
    let db_format_type = &args[1];
    let normalized_db_format_type = db_format_type.to_lowercase();
    let normalized_db_format_type = normalized_db_format_type.trim();
    match normalized_db_format_type {
        "inline_metadata" => {
            println!("inline_metadata");
            if args.len() != 3 {
                eprintln!("invalid inline metadata format args");
                std::process::exit(1);
            }
            let db_path = Path::new(&args[2]);
            println!("db dir {}", db_path.to_str().unwrap());
            let mut data_file_path = db_path.to_path_buf();
            data_file_path.push("imdb_inline_metadata.bin");
            println!("file path {}", data_file_path.to_str().unwrap());
            let pager = ImdbInlineMetaDataPager::new(Path::new(&data_file_path)).unwrap();
            for entry in pager {
                let storage_entry = entry.unwrap();
                println!(
                    "read entry with key {}, value {}",
                    String::from_utf8(storage_entry.record.key).unwrap(),
                    String::from_utf8(storage_entry.record.value).unwrap()
                );
            }
        }
        "seperate_metadata" => todo!("seperate_metadata is not yet supported"),
        _ => {
            eprintln!("unkown db file format!");
            std::process::exit(1);
        }
    }
}
