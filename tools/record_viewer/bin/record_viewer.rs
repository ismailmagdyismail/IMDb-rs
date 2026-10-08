use std::path::Path;

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
use record_viewer::core::{record_viewer::RecordViewer, record_viewer_cli_args::parse_cli_args};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (viewer_config, viewer_operation) = match parse_cli_args(&args) {
        Ok((config, operation)) => (config, operation),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    match viewer_config.imdb_format {
        ImdbFormat::InlineMetadata => {
            let data_file_path =
                Path::new(&viewer_config.db_dir_path).join(IMDB_INLINE_METADATA_RECORDS_FILE_NAME);
            let pager = ImdbInlineMetaDataPager::new(&data_file_path)
                .map_err(|e| format!("Failed to create pager: {}", e.to_string()))
                .unwrap_or_else(|err| {
                    eprintln!("{}", err);
                    std::process::exit(1);
                });
            let mut viewer = RecordViewer::new(pager);
            viewer.execute(viewer_operation);
        }
        ImdbFormat::SeperateMetadata => todo!("seperate_metadata is not yet supported"),
    };
}
