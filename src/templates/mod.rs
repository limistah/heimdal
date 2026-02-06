pub mod engine;
pub mod variables;

pub use engine::TemplateEngine;
pub use variables::{get_system_variables, merge_variables};
