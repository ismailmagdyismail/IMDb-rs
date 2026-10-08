use std::{
    io::{BufRead, Write},
    path::Path,
};

use imdb::core::{
    operations::imdb_format::ImdbFormat,
    storage::{
        imdb_inline_metadata_storage_engine::{
            imdb_inline_metadata_disk_manager::IMDB_INLINE_METADATA_RECORDS_FILE_NAME,
            imdb_inline_metadata_pager::ImdbInlineMetaDataPager,
        },
        imdb_storage_operations_status::ImdbOperationStatus,
    },
};
use record_viewer::core::{
    record_viewer::RecordViewer, record_viewer_cli_args::parse_cli_args,
    record_viewer_config::RecordViewerConfig, record_viewer_operation::RecordViewerOperation,
};

fn repl<T>(command_handler: &mut T)
where
    T: FnMut(Vec<&str>),
{
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    loop {
        stdout.write_all("Command > ".as_bytes()).unwrap();
        stdout.flush().unwrap();
        let stdin = stdin.lock();
        for line in stdin.lines() {
            let line = line.unwrap();
            let args: Vec<&str> = line.split(' ').collect();
            command_handler(args);
            break;
        }
    }
}

fn run_inline_metadata_viewer_repl(viewer_config: &RecordViewerConfig) {
    let data_file_path =
        Path::new(&viewer_config.db_dir_path).join(IMDB_INLINE_METADATA_RECORDS_FILE_NAME);
    let pager = ImdbInlineMetaDataPager::new(&data_file_path)
        .map_err(|e| format!("Failed to create pager: {}", e.to_string()))
        .unwrap_or_else(|err| {
            eprintln!("{}", err);
            std::process::exit(1);
        });
    let mut viewer = RecordViewer::new(pager);
    repl(&mut |args| {
        let record_viewer_operation = match RecordViewerOperation::new(&args) {
            Ok(view_operation) => view_operation,
            Err(err) => {
                eprintln!("Error Happened in executing command {}", err);
                return;
            }
        };
        viewer.execute(record_viewer_operation);
    });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let viewer_config = match parse_cli_args(&args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    match viewer_config.imdb_format {
        ImdbFormat::InlineMetadata => {
            run_inline_metadata_viewer_repl(&viewer_config);
        }
        ImdbFormat::SeperateMetadata => todo!("seperate_metadata is not yet supported"),
    };
}
