use console::style;

// Note: These are utility functions for future use in wizard UI
#[allow(dead_code)]
/// Print a section header
pub fn print_header(text: &str) {
    println!("\n{}", style(text).bold().underlined());
}

#[allow(dead_code)]
/// Print an info message
pub fn print_info(text: &str) {
    println!("{} {}", style("ℹ").blue(), text);
}

#[allow(dead_code)]
/// Print a success message
pub fn print_success(text: &str) {
    println!("{} {}", style("✓").green(), text);
}

#[allow(dead_code)]
/// Print a warning message
pub fn print_warning(text: &str) {
    println!("{} {}", style("⚠").yellow(), text);
}

#[allow(dead_code)]
/// Print an error message
pub fn print_error(text: &str) {
    eprintln!("{} {}", style("✗").red(), text);
}

#[allow(dead_code)]
/// Print a step message
pub fn print_step(text: &str) {
    println!("{} {}", style("→").cyan(), text);
}
