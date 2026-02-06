use colored::Colorize;

pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

pub fn warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

pub fn step(msg: &str) {
    println!("{} {}", "→".cyan().bold(), msg);
}

pub fn header(msg: &str) {
    println!("\n{}", msg.bold().underline());
}
