.PHONY: help build build-release test test-verbose clean install uninstall fmt fmt-check clippy clippy-strict check dev run doc bench coverage all

# Default target
.DEFAULT_GOAL := help

# Binary name
BINARY_NAME := heimdal
INSTALL_PATH := /usr/local/bin

# Rust toolchain
CARGO := cargo
RUSTC := rustc

# Colors for output
COLOR_RESET := \033[0m
COLOR_BOLD := \033[1m
COLOR_GREEN := \033[32m
COLOR_YELLOW := \033[33m
COLOR_BLUE := \033[34m

help: ## Show this help message
	@echo "$(COLOR_BOLD)$(COLOR_BLUE)Heimdal - Makefile Commands$(COLOR_RESET)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(COLOR_GREEN)%-20s$(COLOR_RESET) %s\n", $$1, $$2}'
	@echo ""

build: ## Build the project in debug mode
	@echo "$(COLOR_BOLD)Building $(BINARY_NAME) (debug mode)...$(COLOR_RESET)"
	$(CARGO) build

build-release: ## Build the project in release mode (optimized)
	@echo "$(COLOR_BOLD)Building $(BINARY_NAME) (release mode)...$(COLOR_RESET)"
	$(CARGO) build --release

test: ## Run tests
	@echo "$(COLOR_BOLD)Running tests...$(COLOR_RESET)"
	$(CARGO) test

test-verbose: ## Run tests with verbose output
	@echo "$(COLOR_BOLD)Running tests (verbose)...$(COLOR_RESET)"
	$(CARGO) test --verbose

test-all: ## Run all tests including ignored ones
	@echo "$(COLOR_BOLD)Running all tests...$(COLOR_RESET)"
	$(CARGO) test -- --include-ignored

clean: ## Clean build artifacts
	@echo "$(COLOR_BOLD)Cleaning build artifacts...$(COLOR_RESET)"
	$(CARGO) clean
	@rm -rf target/
	@echo "$(COLOR_GREEN)Clean complete!$(COLOR_RESET)"

install: build-release ## Install the binary to /usr/local/bin
	@echo "$(COLOR_BOLD)Installing $(BINARY_NAME) to $(INSTALL_PATH)...$(COLOR_RESET)"
	@install -m 755 target/release/$(BINARY_NAME) $(INSTALL_PATH)/$(BINARY_NAME)
	@echo "$(COLOR_GREEN)Installation complete!$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)Run '$(BINARY_NAME) --version' to verify installation$(COLOR_RESET)"

uninstall: ## Uninstall the binary from /usr/local/bin
	@echo "$(COLOR_BOLD)Uninstalling $(BINARY_NAME)...$(COLOR_RESET)"
	@rm -f $(INSTALL_PATH)/$(BINARY_NAME)
	@echo "$(COLOR_GREEN)Uninstall complete!$(COLOR_RESET)"

fmt: ## Format code using rustfmt
	@echo "$(COLOR_BOLD)Formatting code...$(COLOR_RESET)"
	$(CARGO) fmt

fmt-check: ## Check code formatting without making changes
	@echo "$(COLOR_BOLD)Checking code formatting...$(COLOR_RESET)"
	$(CARGO) fmt -- --check

clippy: ## Run clippy linter
	@echo "$(COLOR_BOLD)Running clippy...$(COLOR_RESET)"
	$(CARGO) clippy --all-targets

clippy-strict: ## Run clippy with strict warnings (deny warnings)
	@echo "$(COLOR_BOLD)Running clippy (strict mode)...$(COLOR_RESET)"
	$(CARGO) clippy --all-targets -- -D warnings

check: ## Quick compile check without building
	@echo "$(COLOR_BOLD)Running cargo check...$(COLOR_RESET)"
	$(CARGO) check

dev: fmt clippy test ## Run full development workflow (format, lint, test)
	@echo "$(COLOR_GREEN)Development checks passed!$(COLOR_RESET)"

run: ## Run the project (debug mode)
	@echo "$(COLOR_BOLD)Running $(BINARY_NAME)...$(COLOR_RESET)"
	$(CARGO) run

doc: ## Generate and open project documentation
	@echo "$(COLOR_BOLD)Generating documentation...$(COLOR_RESET)"
	$(CARGO) doc --open --no-deps

doc-all: ## Generate documentation including dependencies
	@echo "$(COLOR_BOLD)Generating documentation (including dependencies)...$(COLOR_RESET)"
	$(CARGO) doc --open

bench: ## Run benchmarks
	@echo "$(COLOR_BOLD)Running benchmarks...$(COLOR_RESET)"
	$(CARGO) bench

coverage: ## Generate test coverage report (requires tarpaulin)
	@echo "$(COLOR_BOLD)Generating coverage report...$(COLOR_RESET)"
	@command -v cargo-tarpaulin >/dev/null 2>&1 || { echo "$(COLOR_YELLOW)Installing cargo-tarpaulin...$(COLOR_RESET)"; $(CARGO) install cargo-tarpaulin; }
	$(CARGO) tarpaulin --out Html --output-dir coverage

update: ## Update dependencies
	@echo "$(COLOR_BOLD)Updating dependencies...$(COLOR_RESET)"
	$(CARGO) update

outdated: ## Check for outdated dependencies (requires cargo-outdated)
	@echo "$(COLOR_BOLD)Checking for outdated dependencies...$(COLOR_RESET)"
	@command -v cargo-outdated >/dev/null 2>&1 || { echo "$(COLOR_YELLOW)Installing cargo-outdated...$(COLOR_RESET)"; $(CARGO) install cargo-outdated; }
	$(CARGO) outdated

audit: ## Check for security vulnerabilities (requires cargo-audit)
	@echo "$(COLOR_BOLD)Auditing dependencies for vulnerabilities...$(COLOR_RESET)"
	@command -v cargo-audit >/dev/null 2>&1 || { echo "$(COLOR_YELLOW)Installing cargo-audit...$(COLOR_RESET)"; $(CARGO) install cargo-audit; }
	$(CARGO) audit

watch: ## Watch for changes and rebuild (requires cargo-watch)
	@echo "$(COLOR_BOLD)Watching for changes...$(COLOR_RESET)"
	@command -v cargo-watch >/dev/null 2>&1 || { echo "$(COLOR_YELLOW)Installing cargo-watch...$(COLOR_RESET)"; $(CARGO) install cargo-watch; }
	$(CARGO) watch -x build

watch-test: ## Watch for changes and run tests (requires cargo-watch)
	@echo "$(COLOR_BOLD)Watching for changes and running tests...$(COLOR_RESET)"
	@command -v cargo-watch >/dev/null 2>&1 || { echo "$(COLOR_YELLOW)Installing cargo-watch...$(COLOR_RESET)"; $(CARGO) install cargo-watch; }
	$(CARGO) watch -x test

bloat: ## Analyze binary size (requires cargo-bloat)
	@echo "$(COLOR_BOLD)Analyzing binary size...$(COLOR_RESET)"
	@command -v cargo-bloat >/dev/null 2>&1 || { echo "$(COLOR_YELLOW)Installing cargo-bloat...$(COLOR_RESET)"; $(CARGO) install cargo-bloat; }
	$(CARGO) bloat --release

all: fmt clippy test build-release ## Run all checks and build release binary
	@echo "$(COLOR_GREEN)All tasks completed successfully!$(COLOR_RESET)"

ci: fmt-check clippy-strict test ## Run CI checks (format check, strict clippy, tests)
	@echo "$(COLOR_GREEN)CI checks passed!$(COLOR_RESET)"

version: ## Show version information
	@echo "$(COLOR_BOLD)Version Information:$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)Cargo:$(COLOR_RESET)     $$($(CARGO) --version)"
	@echo "$(COLOR_YELLOW)Rustc:$(COLOR_RESET)     $$($(RUSTC) --version)"
	@if [ -f target/release/$(BINARY_NAME) ]; then \
		echo "$(COLOR_YELLOW)$(BINARY_NAME):$(COLOR_RESET)  $$(target/release/$(BINARY_NAME) --version)"; \
	elif [ -f target/debug/$(BINARY_NAME) ]; then \
		echo "$(COLOR_YELLOW)$(BINARY_NAME):$(COLOR_RESET)  $$(target/debug/$(BINARY_NAME) --version)"; \
	fi
