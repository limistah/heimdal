use crate::cli::SecretCmd;
use crate::secrets::{delete_secret, get_secret, list_secrets, set_secret};
use crate::state::State;
use crate::utils::{info, success};
use anyhow::Result;

pub fn run(action: SecretCmd) -> Result<()> {
    match action {
        SecretCmd::Add { name, value } => add(&name, value.as_deref()),
        SecretCmd::Get { name } => get(&name),
        SecretCmd::Remove { name, force } => remove(&name, force),
        SecretCmd::List => list(),
    }
}

fn add(name: &str, value: Option<&str>) -> Result<()> {
    let state = State::load()?;
    let secret_value = match value {
        Some(v) => v.to_string(),
        None => dialoguer::Password::new()
            .with_prompt(format!("Value for secret '{}'", name))
            .interact()
            .map_err(|e| anyhow::anyhow!("Failed to read secret: {}", e))?,
    };
    set_secret(&state.dotfiles_path, name, &secret_value)?;
    success(&format!("Secret '{}' saved", name));
    Ok(())
}

fn get(name: &str) -> Result<()> {
    println!("{}", get_secret(name)?);
    Ok(())
}

fn remove(name: &str, force: bool) -> Result<()> {
    let state = State::load()?;
    if !force && !crate::utils::confirm(&format!("Remove secret '{}'?", name)) {
        info("Cancelled.");
        return Ok(());
    }
    delete_secret(&state.dotfiles_path, name)?;
    success(&format!("Secret '{}' removed", name));
    Ok(())
}

fn list() -> Result<()> {
    let state = State::load()?;
    let names = list_secrets(&state.dotfiles_path);
    if names.is_empty() {
        info("No secrets stored.");
    } else {
        for n in &names {
            println!("  - {}", n);
        }
    }
    Ok(())
}
