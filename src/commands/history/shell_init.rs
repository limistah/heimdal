use crate::history::shell::{init_script, Shell};
use anyhow::Result;
use std::str::FromStr;

pub fn run(shell: &str) -> Result<()> {
    let shell = Shell::from_str(shell)?;
    print!("{}", init_script(shell));
    Ok(())
}
