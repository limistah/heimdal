pub mod record;
pub mod search;
pub mod shell_init;
pub mod sync;

use crate::cli::HistoryCmd;
use anyhow::Result;

pub fn run(action: HistoryCmd) -> Result<()> {
    match action {
        HistoryCmd::Record {
            cmd,
            exit,
            dir,
            session,
        } => record::run(&cmd, exit, &dir, &session),
        HistoryCmd::Search { query, interactive } => search::run(query.as_deref(), interactive),
        HistoryCmd::ShellInit { shell } => shell_init::run(&shell),
        HistoryCmd::Sync => sync::run(),
        HistoryCmd::SessionId => {
            println!("{}", uuid::Uuid::new_v4());
            Ok(())
        }
    }
}
