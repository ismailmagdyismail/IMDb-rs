use crate::core::{
    imdb::Imdb, imdb_cli_args_parser::ImdbCliArgsParser, imdb_opeartion::ImdbOperation,
    imdb_repl::ImdbRepl,
};

pub fn launch_imdb(args: Vec<String>) -> Result<(), String> {
    let (imdb_config, imdb_operation) = ImdbCliArgsParser::parse(args)?;
    dbg!(&imdb_config);
    dbg!(&imdb_operation);

    match imdb_operation {
        ImdbOperation::Deamon => todo!("[Imdb:: Server|Deamon Mode Not Supported Yet!!]"),
        ImdbOperation::Repl => {
            let mut repl: ImdbRepl = ImdbRepl::new(Imdb::new());
            repl.run();
        }
    }

    Ok(())
}
