use crate::cli::TemplateCmd;
use crate::config::{load_config, resolve_profile};
use crate::state::State;
use crate::templates::{build_vars, render_string};
use crate::utils::info;
use anyhow::Result;

pub fn run(action: TemplateCmd) -> Result<()> {
    match action {
        TemplateCmd::List => list(),
        TemplateCmd::Preview { src, profile } => preview(&src, profile.as_deref()),
        TemplateCmd::Variables { profile } => variables(profile.as_deref()),
    }
}

fn list() -> Result<()> {
    let state = State::load()?;
    let config = load_config(&state.dotfiles_path.join("heimdal.yaml"))?;
    let profile = resolve_profile(&config, &state.active_profile)?;
    if profile.templates.is_empty() {
        info("No templates configured for this profile.");
    } else {
        for t in &profile.templates {
            println!("  {} → {}", t.src, t.dest);
        }
    }
    Ok(())
}

fn preview(src_name: &str, profile_name: Option<&str>) -> Result<()> {
    let state = State::load()?;
    let config = load_config(&state.dotfiles_path.join("heimdal.yaml"))?;
    let prof_name = profile_name.unwrap_or(&state.active_profile);
    let profile = resolve_profile(&config, prof_name)?;

    let entry = profile.templates.iter()
        .find(|t| t.src == src_name || t.src.ends_with(src_name))
        .ok_or_else(|| anyhow::anyhow!(
            "Template '{}' not found in profile '{}'. Use 'heimdal template list'.",
            src_name, prof_name
        ))?;

    let src_path = state.dotfiles_path.join(&entry.src);
    let vars = build_vars(&entry.vars, "env");
    let content = std::fs::read_to_string(&src_path)?;
    print!("{}", render_string(&content, &vars));
    Ok(())
}

fn variables(profile_name: Option<&str>) -> Result<()> {
    let state = State::load()?;
    let config = load_config(&state.dotfiles_path.join("heimdal.yaml"))?;
    let prof_name = profile_name.unwrap_or(&state.active_profile);
    let profile = resolve_profile(&config, prof_name)?;

    let sys = crate::templates::system_vars();
    println!("System variables:");
    let mut keys: Vec<_> = sys.keys().collect();
    keys.sort();
    for k in keys { println!("  {}: {}", k, sys[k]); }

    for tmpl in &profile.templates {
        if !tmpl.vars.is_empty() {
            println!("\nVars for {}:", tmpl.src);
            let mut pairs: Vec<_> = tmpl.vars.iter().collect();
            pairs.sort_by_key(|(k, _)| (*k).clone());
            for (k, v) in pairs { println!("  {}: {}", k, v); }
        }
    }
    Ok(())
}
