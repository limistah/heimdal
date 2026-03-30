use crate::cli::ImportArgs;
use crate::import::{detect_tool, generate_heimdal_yaml, import_from, SourceTool};
use crate::utils::{expand_path, info, success, warning};
use anyhow::Result;

pub fn run(args: ImportArgs) -> Result<()> {
    let path = expand_path(&args.path);

    if !path.exists() {
        anyhow::bail!(
            "Path '{}' does not exist. Check the path and try again.",
            path.display()
        );
    }

    // Resolve tool
    let tool = match &args.from {
        Some(f) if f == "auto" || f.is_empty() => {
            let detected = detect_tool(&path);
            if let Some(ref t) = detected {
                info(&format!("Auto-detected: {} format", t.as_str()));
            } else {
                info("No format detected, using stow-style walk");
            }
            detected
        }
        Some(f) => match SourceTool::from_str(f) {
            Some(t) => Some(t),
            None => anyhow::bail!(
                "Unknown dotfile manager '{}'. Valid options: stow, dotbot, chezmoi, yadm, homesick, auto",
                f
            ),
        },
        None => {
            let detected = detect_tool(&path);
            if let Some(ref t) = detected {
                info(&format!("Auto-detected: {} format", t.as_str()));
            }
            detected
        }
    };

    let tool_name = tool.as_ref().map(|t| t.as_str()).unwrap_or("stow");
    info(&format!(
        "Importing from {} at {}",
        tool_name,
        path.display()
    ));

    let result = import_from(&path, tool)?;

    // Print warnings
    for w in &result.warnings {
        warning(w);
    }

    info(&format!(
        "Found {} dotfile mapping(s)",
        result.dotfiles.len()
    ));

    // Generate YAML
    let yaml = generate_heimdal_yaml(&result, "default")?;

    if args.preview {
        println!("\n--- Generated heimdal.yaml (preview) ---");
        println!("{}", yaml);
        println!("--- end preview ---");
        info("Use --output <path> to write this file.");
        return Ok(());
    }

    // Determine output path
    let output_path = match &args.output {
        Some(p) => expand_path(p),
        None => {
            // Write to the imported dotfiles dir
            path.join("heimdal.yaml")
        }
    };

    if output_path.exists() {
        warning(&format!(
            "'{}' already exists. Use --output to specify a different path.",
            output_path.display()
        ));
        anyhow::bail!("Output file already exists: {}", output_path.display());
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output_path, &yaml)?;
    success(&format!(
        "Generated heimdal.yaml at {}",
        output_path.display()
    ));
    info("Next: review the file, then run 'heimdal init --repo <url> --profile default'");

    Ok(())
}
