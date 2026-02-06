use anyhow::{Context, Result};
use colored::Colorize;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Compile regex once at startup for better performance
static VARIABLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{\{\s*(\w+)\s*\}\}").expect("Invalid variable regex pattern"));

/// Simple template engine for variable substitution
/// Supports {{ variable }} syntax only - NO complex logic
pub struct TemplateEngine {
    variables: HashMap<String, String>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Create template engine with variables
    pub fn with_variables(variables: HashMap<String, String>) -> Self {
        Self { variables }
    }

    /// Add a variable for substitution
    pub fn add_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    /// Add multiple variables
    pub fn add_variables(&mut self, vars: HashMap<String, String>) {
        self.variables.extend(vars);
    }

    /// Get all variables
    pub fn get_variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Render a template string
    /// Replaces {{ variable }} with values from the variables map
    pub fn render(&self, template: &str) -> Result<String> {
        // Track missing variables
        let mut missing_vars = Vec::new();

        // Single-pass replacement using replace_all with closure
        let result = VARIABLE_REGEX.replace_all(template, |caps: &regex::Captures| {
            let var_name = &caps[1];

            if let Some(value) = self.variables.get(var_name) {
                value.clone()
            } else {
                missing_vars.push(var_name.to_string());
                // Keep the original placeholder for missing variables
                caps[0].to_string()
            }
        });

        // Warn about missing variables
        if !missing_vars.is_empty() {
            eprintln!(
                "{} Template contains undefined variables: {}",
                "⚠".yellow(),
                missing_vars.join(", ")
            );
        }

        Ok(result.to_string())
    }

    /// Render a template file and write to output
    pub fn render_file(&self, input: &Path, output: &Path) -> Result<()> {
        let content = fs::read_to_string(input)
            .with_context(|| format!("Failed to read template file: {}", input.display()))?;

        let rendered = self.render(&content)?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        fs::write(output, rendered)
            .with_context(|| format!("Failed to write rendered file: {}", output.display()))?;

        Ok(())
    }

    /// Find all variables used in a template
    pub fn find_variables(template: &str) -> Vec<String> {
        VARIABLE_REGEX
            .captures_iter(template)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    /// Check if a template has undefined variables
    pub fn validate(&self, template: &str) -> Result<Vec<String>> {
        let used_vars = Self::find_variables(template);
        let mut missing = Vec::new();

        for var in used_vars {
            if !self.variables.contains_key(&var) {
                missing.push(var);
            }
        }

        Ok(missing)
    }

    /// Check if a file is a template (has .tmpl extension)
    pub fn is_template_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "tmpl")
            .unwrap_or(false)
    }

    /// Get output path for a template file
    /// Example: .gitconfig.tmpl → .gitconfig
    pub fn get_output_path(template_path: &Path) -> Option<String> {
        let path_str = template_path.to_str()?;

        if path_str.ends_with(".tmpl") {
            Some(path_str.trim_end_matches(".tmpl").to_string())
        } else {
            None
        }
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_template() {
        let mut engine = TemplateEngine::new();
        engine.add_variable("name", "John");
        let result = engine.render("Hello {{ name }}!").unwrap();
        assert_eq!(result, "Hello John!");
    }

    #[test]
    fn test_multiple_variables() {
        let mut engine = TemplateEngine::new();
        engine.add_variable("name", "John");
        engine.add_variable("email", "john@example.com");

        let template = "Name: {{ name }}, Email: {{ email }}";
        let result = engine.render(template).unwrap();
        assert_eq!(result, "Name: John, Email: john@example.com");
    }

    #[test]
    fn test_whitespace_handling() {
        let mut engine = TemplateEngine::new();
        engine.add_variable("name", "John");

        // Test various whitespace patterns
        assert_eq!(engine.render("{{name}}").unwrap(), "John");
        assert_eq!(engine.render("{{ name }}").unwrap(), "John");
        assert_eq!(engine.render("{{  name  }}").unwrap(), "John");
    }

    #[test]
    fn test_missing_variable() {
        let engine = TemplateEngine::new();
        let result = engine.render("Hello {{ name }}!").unwrap();
        // Should still render but keep the variable marker
        assert_eq!(result, "Hello {{ name }}!");
    }

    #[test]
    fn test_find_variables() {
        let template = "{{ name }} at {{ email }} - {{ company }}";
        let vars = TemplateEngine::find_variables(template);
        assert_eq!(vars.len(), 3);
        assert!(vars.contains(&"name".to_string()));
        assert!(vars.contains(&"email".to_string()));
        assert!(vars.contains(&"company".to_string()));
    }

    #[test]
    fn test_validate() {
        let mut engine = TemplateEngine::new();
        engine.add_variable("name", "John");

        let template = "{{ name }} at {{ email }}";
        let missing = engine.validate(template).unwrap();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "email");
    }

    #[test]
    fn test_is_template_file() {
        assert!(TemplateEngine::is_template_file(Path::new(
            ".gitconfig.tmpl"
        )));
        assert!(TemplateEngine::is_template_file(Path::new(
            "config.yaml.tmpl"
        )));
        assert!(!TemplateEngine::is_template_file(Path::new(".gitconfig")));
        assert!(!TemplateEngine::is_template_file(Path::new("file.txt")));
    }

    #[test]
    fn test_get_output_path() {
        assert_eq!(
            TemplateEngine::get_output_path(Path::new(".gitconfig.tmpl")).unwrap(),
            ".gitconfig"
        );
        assert_eq!(
            TemplateEngine::get_output_path(Path::new("dir/config.yaml.tmpl")).unwrap(),
            "dir/config.yaml"
        );
        assert!(TemplateEngine::get_output_path(Path::new(".gitconfig")).is_none());
    }

    #[test]
    fn test_duplicate_variables() {
        let mut engine = TemplateEngine::new();
        engine.add_variable("name", "John");

        let template = "{{ name }} and {{ name }} again";
        let result = engine.render(template).unwrap();
        assert_eq!(result, "John and John again");
    }
}
