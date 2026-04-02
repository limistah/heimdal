use crate::cli::KeyCmd;
use crate::state::State;
use crate::utils::{info, success};
use anyhow::Result;

pub fn run(action: KeyCmd) -> Result<()> {
    match action {
        KeyCmd::Gen => gen(),
        KeyCmd::Set => set(),
        KeyCmd::Show => show(),
        KeyCmd::Export => export(),
        KeyCmd::Import { blob } => import(blob.as_deref()),
    }
}

fn gen() -> Result<()> {
    let state = State::load()?;
    let key = crate::key::generate(&state.dotfiles_path)?;
    success("Bifrost key generated and stored in OS keychain.");
    println!("  Key: {}", hex::encode(key));
    println!();
    info("Back it up now:  heimdal key export");
    info("Without a backup, losing this machine means losing access to all encrypted data.");
    Ok(())
}

fn set() -> Result<()> {
    let state = State::load()?;
    let hex = dialoguer::Password::new()
        .with_prompt("Paste bifrost key (64 hex characters)")
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to read key: {e}"))?;
    crate::key::set(&state.dotfiles_path, &hex)?;
    success("Bifrost key stored in OS keychain.");
    Ok(())
}

fn show() -> Result<()> {
    let key = crate::key::load()?;
    println!("{}", hex::encode(key));
    Ok(())
}

fn export() -> Result<()> {
    let key = crate::key::load()?;
    let passphrase = dialoguer::Password::new()
        .with_prompt("Passphrase to protect the export")
        .with_confirmation("Confirm passphrase", "Passphrases do not match")
        .interact()
        .map_err(|e| anyhow::anyhow!("failed to read passphrase: {e}"))?;
    let blob = crate::key::backup::export_with_passphrase(&key, &passphrase)?;
    println!();
    println!("{}", blob);
    println!();
    info("Save this blob in your password manager or a secure note.");
    info("It is safe to store anywhere — it is protected by your passphrase.");
    Ok(())
}

fn import(blob_arg: Option<&str>) -> Result<()> {
    let state = State::load()?;
    let blob = match blob_arg {
        Some(b) => b.to_string(),
        None => dialoguer::Input::<String>::new()
            .with_prompt("Paste the export blob")
            .interact_text()
            .map_err(|e| anyhow::anyhow!("failed to read blob: {e}"))?,
    };
    let passphrase = dialoguer::Password::new()
        .with_prompt("Passphrase")
        .interact()
        .map_err(|e| anyhow::anyhow!("failed to read passphrase: {e}"))?;
    let key = crate::key::backup::import_with_passphrase(&blob, &passphrase)?;
    crate::key::set(&state.dotfiles_path, &hex::encode(key))?;
    success("Bifrost key restored and stored in OS keychain.");
    Ok(())
}
