use crate::core::{
    imdb::Imdb,
    operations::{imdb_cli_args_parser::ImdbCliArgsParser, imdb_operation::ImdbOperation},
    repl::imdb_repl::ImdbRepl,
};

pub fn launch_imdb(args: Vec<String>) -> Result<(), String> {
    let (imdb_config, imdb_operation) = ImdbCliArgsParser::parse(args)?;
    dbg!(&imdb_config);
    dbg!(&imdb_operation);

    match imdb_operation {
        ImdbOperation::Deamon => todo!("[Imdb:: Server|Deamon Mode Not Supported Yet!!]"),
        ImdbOperation::Repl => {
            let db = Imdb::new(imdb_config)?;
            let mut repl: ImdbRepl = ImdbRepl::new(db);
            repl.run();
        }
    }

    Ok(())
}
