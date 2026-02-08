/// Integration tests for heimdal CLI
///
/// These tests verify actual CLI behavior following global testing standards:
/// - Test happy paths (expected usage)
/// - Test sad paths (error cases)
/// - Test edge cases
/// - Use actual commands that exist in the binary
mod helpers;
mod test_apply;
mod test_basic;
mod test_git_commands;
mod test_import;
mod test_init;
mod test_packages;
mod test_profile;
mod test_profiles;
mod test_secret;
mod test_state;
mod test_status;
mod test_template;
mod test_utility_commands;
mod test_validate;
