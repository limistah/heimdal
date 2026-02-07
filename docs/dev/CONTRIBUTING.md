# Contributing to Heimdal

Thank you for your interest in contributing to Heimdal! We welcome contributions from the community.

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue on GitHub with:
- A clear title and description
- Steps to reproduce the issue
- Expected vs actual behavior
- Your environment (OS, Rust version, etc.)
- Relevant configuration files (redacted of sensitive info)

### Suggesting Features

We love new ideas! Please open an issue with:
- A clear description of the feature
- Why it would be useful
- Possible implementation approaches

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Run the test suite (`cargo test`)
6. Run the linter (`cargo clippy`)
7. Format your code (`cargo fmt`)
8. Commit your changes (`git commit -m 'Add amazing feature'`)
9. Push to your branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Git

### Building

```bash
git clone https://github.com/limistah/heimdal.git
cd heimdal
cargo build
```

### Running Tests

```bash
cargo test
```

### Running Locally

```bash
cargo run -- --help
```

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Pass all clippy lints (`cargo clippy`)
- Write clear, descriptive commit messages
- Add comments for complex logic
- Update documentation when adding features

## Project Structure

```
heimdal/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── cli.rs            # Command line interface
│   ├── config/           # Configuration parsing
│   ├── package/          # Package managers
│   ├── symlink/          # Symlink management
│   ├── state/            # State management
│   ├── sync/             # Git sync & cron
│   └── utils/            # Utilities
├── examples/             # Example configurations
├── README.md
└── Cargo.toml
```

## Adding a New Package Manager

To add support for a new package manager:

1. Create a new file in `src/package/` (e.g., `zypper.rs`)
2. Implement the `PackageManager` trait
3. Add the package manager to `src/package/mod.rs`
4. Update `detect_package_manager()` to detect the new manager
5. Update `install_packages()` to handle the new source type
6. Add package mappings in `src/package/mapper.rs`
7. Update documentation

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

Create a test dotfiles repository and test the full workflow:

```bash
mkdir /tmp/test-dotfiles
cd /tmp/test-dotfiles
git init
# Add test configuration
cargo run -- init --profile test --repo /tmp/test-dotfiles --path /tmp/test-install
cargo run -- apply --dry-run
```

## Documentation

- Update README.md for user-facing features
- Add examples to the `examples/` directory
- Document complex functions with doc comments
- Update CHANGELOG.md

## Questions?

Feel free to open an issue or discussion on GitHub if you have any questions!

## Code of Conduct

Be respectful and inclusive. We're all here to build something great together.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
