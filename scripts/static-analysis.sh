#!/bin/bash
# Static Analysis Script for Heimdal
# Runs all static analysis tools locally

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BOLD}=== Heimdal Static Analysis ===${NC}\n"

# Check if tools are installed
check_tool() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${YELLOW}⚠️  $1 not installed. Run: cargo install $1${NC}"
        return 1
    fi
    return 0
}

# 1. Format Check
echo -e "${BOLD}1. Checking code formatting...${NC}"
if cargo fmt --check; then
    echo -e "${GREEN}✅ Format check passed${NC}\n"
else
    echo -e "${RED}❌ Format check failed. Run: cargo fmt${NC}\n"
    exit 1
fi

# 2. Clippy
echo -e "${BOLD}2. Running Clippy...${NC}"
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✅ Clippy passed${NC}\n"
else
    echo -e "${RED}❌ Clippy found issues${NC}\n"
    exit 1
fi

# 3. Tests
echo -e "${BOLD}3. Running tests...${NC}"
if cargo test --all-targets; then
    echo -e "${GREEN}✅ Tests passed${NC}\n"
else
    echo -e "${RED}❌ Tests failed${NC}\n"
    exit 1
fi

# 4. Security Audit
echo -e "${BOLD}4. Running security audit...${NC}"
if check_tool cargo-audit; then
    if cargo audit; then
        echo -e "${GREEN}✅ Security audit passed${NC}\n"
    else
        echo -e "${YELLOW}⚠️  Security audit found issues${NC}\n"
    fi
else
    echo -e "${YELLOW}⚠️  Skipping security audit${NC}\n"
fi

# 5. Dependency Check
echo -e "${BOLD}5. Running dependency checks...${NC}"
if check_tool cargo-deny; then
    if cargo deny check; then
        echo -e "${GREEN}✅ Dependency check passed${NC}\n"
    else
        echo -e "${YELLOW}⚠️  Dependency check found issues${NC}\n"
    fi
else
    echo -e "${YELLOW}⚠️  Skipping dependency check${NC}\n"
fi

# 6. Outdated Dependencies
echo -e "${BOLD}6. Checking for outdated dependencies...${NC}"
if check_tool cargo-outdated; then
    cargo outdated
    echo ""
else
    echo -e "${YELLOW}⚠️  Skipping outdated check${NC}\n"
fi

# 7. Unused Dependencies
echo -e "${BOLD}7. Checking for unused dependencies...${NC}"
if check_tool cargo-udeps; then
    if cargo +nightly udeps --all-targets 2>/dev/null; then
        echo -e "${GREEN}✅ No unused dependencies${NC}\n"
    else
        echo -e "${YELLOW}⚠️  Requires nightly Rust${NC}\n"
    fi
else
    echo -e "${YELLOW}⚠️  Skipping unused deps check${NC}\n"
fi

# 8. Code Coverage (optional)
if [ "$1" == "--coverage" ]; then
    echo -e "${BOLD}8. Generating code coverage...${NC}"
    if check_tool cargo-tarpaulin; then
        cargo tarpaulin --out Html --output-dir coverage
        echo -e "${GREEN}✅ Coverage report generated in coverage/index.html${NC}\n"
    else
        echo -e "${YELLOW}⚠️  Skipping coverage${NC}\n"
    fi
fi

# 9. Binary Size Analysis (optional)
if [ "$1" == "--bloat" ]; then
    echo -e "${BOLD}9. Analyzing binary size...${NC}"
    if check_tool cargo-bloat; then
        cargo bloat --release --crates
        echo ""
    else
        echo -e "${YELLOW}⚠️  Skipping bloat check${NC}\n"
    fi
fi

echo -e "${BOLD}${GREEN}=== All checks completed ===${NC}"
echo ""
echo "Optional checks:"
echo "  --coverage  Generate code coverage report"
echo "  --bloat     Analyze binary size"
echo ""
echo "To install missing tools:"
echo "  cargo install cargo-audit cargo-deny cargo-outdated cargo-udeps cargo-tarpaulin cargo-bloat"
