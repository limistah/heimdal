use console::style;

/// Print a section header
pub fn print_header(text: &str) {
    println!("\n{}", style(text).bold().underlined());
}

/// Print an info message
pub fn print_info(text: &str) {
    println!("{} {}", style("ℹ").blue(), text);
}

/// Print a success message
pub fn print_success(text: &str) {
    println!("{} {}", style("✓").green(), text);
}

/// Print a warning message
pub fn print_warning(text: &str) {
    println!("{} {}", style("⚠").yellow(), text);
}

/// Print an error message
pub fn print_error(text: &str) {
    eprintln!("{} {}", style("✗").red(), text);
}

/// Print a step message
pub fn print_step(text: &str) {
    println!("{} {}", style("→").cyan(), text);
}
