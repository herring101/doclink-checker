# doclink-checker

[![CI](https://github.com/herring101/doclink-checker/workflows/CI/badge.svg)](https://github.com/herring101/doclink-checker/actions)
[![Release](https://github.com/herring101/doclink-checker/workflows/Release/badge.svg)](https://github.com/herring101/doclink-checker/releases)

A fast and reliable Rust tool to analyze markdown documents for broken links, orphaned documents, and generate comprehensive link statistics.

## Features

- 🔍 **Link Detection**: Finds both inline `[text](url)` and reference-style `[text][ref]` links
- 🚨 **Broken Link Detection**: Identifies internal links that point to non-existent files
- 🏝️ **Orphaned Document Detection**: Finds markdown files that aren't referenced by any other documents
- 📊 **Comprehensive Statistics**: Detailed analysis of link patterns across your documentation
- 🎨 **Beautiful CLI Output**: Colorful and well-formatted terminal output
- 🌐 **Multiple Output Formats**: Text and JSON output for easy integration
- ⚡ **Fast Performance**: Built with Rust for speed and reliability

## Installation

### From GitHub Releases (Recommended)

Download the latest binary for your platform from the [releases page](https://github.com/herring101/doclink-checker/releases):

```bash
# Linux x86_64
curl -L https://github.com/herring101/doclink-checker/releases/latest/download/doclink-checker-linux-x86_64.tar.gz | tar xz

# macOS x86_64
curl -L https://github.com/herring101/doclink-checker/releases/latest/download/doclink-checker-macos-x86_64.tar.gz | tar xz

# macOS ARM64 (Apple Silicon)
curl -L https://github.com/herring101/doclink-checker/releases/latest/download/doclink-checker-macos-aarch64.tar.gz | tar xz

# Windows x86_64
# Download doclink-checker-windows-x86_64.exe from the releases page
```

### From Source

Ensure you have [Rust](https://rustup.rs/) installed, then:

```bash
git clone https://github.com/herring101/doclink-checker.git
cd doclink-checker
cargo build --release
```

The binary will be available at `target/release/doclink-checker`.

### Using Cargo

```bash
cargo install --git https://github.com/herring101/doclink-checker.git
```

## Usage

### Check for Broken Links

```bash
# Check current directory
doclink-checker check

# Check specific directory
doclink-checker check --path ./docs

# Verbose output with markdown syntax
doclink-checker check --verbose
```

**Example output:**
```
✗ Found 2 broken links:

  File: docs/api.md:15
  Link: API Reference
  Target: ./missing-file.md
  Reason: File not found: /path/to/docs/missing-file.md

  File: README.md:8
  Link: Contributing Guide
  Target: ./CONTRIBUTING.md
  Reason: File not found: /path/to/CONTRIBUTING.md
```

### Generate Statistics

```bash
# Text output (default)
doclink-checker stats

# JSON output for automation
doclink-checker stats --format json

# Analyze specific directory
doclink-checker stats --path ./documentation
```

**Example output:**
```
Document Link Statistics

Total Documents: 15
Total Links: 42
Internal Links: 38 (90%)
External Links: 4 (9%)
Broken Links: 0
Orphaned Documents: 1

Per-Document Statistics:
  README.md 8 links (7 internal, 1 external)
  docs/api.md 12 links (12 internal, 0 external)
  docs/guide.md 5 links (4 internal, 1 external)
  ...
```

### Find Orphaned Documents

```bash
# Find documents not linked from anywhere
doclink-checker orphans

# Check specific directory
doclink-checker orphans --path ./docs
```

**Example output:**
```
⚠ Found 3 orphaned documents:
  old-deprecated-guide.md
  drafts/unused-feature.md
  temp/scratch-notes.md
```

## Supported Link Formats

doclink-checker recognizes standard markdown link formats:

### Inline Links
```markdown
[Link text](./relative/path.md)
[Link text](/absolute/path.md)
[External link](https://example.com)
```

### Reference Links
```markdown
[Link text][ref-id]
[Link text][]  # Uses link text as reference

[ref-id]: ./target.md
[Link text]: ./target.md
```

## Exit Codes

- `0`: Success, no broken links found
- `1`: Broken links detected or error occurred

This makes it easy to use in CI/CD pipelines:

```bash
# In your CI script
doclink-checker check || exit 1
```

## CI/CD Integration

### GitHub Actions

Add this to your `.github/workflows/docs.yml`:

```yaml
name: Documentation

on: [push, pull_request]

jobs:
  check-links:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Download doclink-checker
        run: |
          curl -L https://github.com/herring101/doclink-checker/releases/latest/download/doclink-checker-linux-x86_64.tar.gz | tar xz
          chmod +x doclink-checker
          
      - name: Check documentation links
        run: ./doclink-checker check --path ./docs
```

### GitLab CI

Add this to your `.gitlab-ci.yml`:

```yaml
check-docs:
  stage: test
  image: ubuntu:latest
  before_script:
    - apt-get update && apt-get install -y curl
    - curl -L https://github.com/herring101/doclink-checker/releases/latest/download/doclink-checker-linux-x86_64.tar.gz | tar xz
    - chmod +x doclink-checker
  script:
    - ./doclink-checker check
```

## Configuration

doclink-checker works out of the box with sensible defaults. Currently, configuration is done via command-line arguments.

### Planned Features

- Configuration file support (`.doclink.toml`)
- External link checking with timeout/retry logic
- Custom ignore patterns
- Integration with popular documentation generators

## Development

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later
- Git

### Building

```bash
git clone https://github.com/herring101/doclink-checker.git
cd doclink-checker
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_find_broken_links
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for security vulnerabilities
cargo audit
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for your changes
4. Implement your changes
5. Ensure all tests pass (`cargo test`)
6. Format your code (`cargo fmt`)
7. Run clippy (`cargo clippy`)
8. Commit your changes (`git commit -m 'Add amazing feature'`)
9. Push to the branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

## Architecture

doclink-checker is built with a clean, modular architecture:

- **LinkAnalyzer**: Core analysis engine that parses markdown and extracts links
- **CLI Module**: Command-line interface built with `clap`
- **Output Formatters**: Text and JSON output formatters
- **Link Detection**: Regex-based parsing for inline and reference links
- **Path Resolution**: Robust relative/absolute path resolution

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- CLI powered by [clap](https://github.com/clap-rs/clap)
- Colorful output via [colored](https://github.com/colored-rs/colored)
- Regular expressions with [regex](https://github.com/rust-lang/regex)
- File traversal using [walkdir](https://github.com/BurntSushi/walkdir)

## Changelog

### v0.1.0 (Initial Release)

- ✨ Link detection for inline and reference-style markdown links
- 🔍 Broken link detection for internal file references
- 🏝️ Orphaned document detection
- 📊 Comprehensive link statistics with per-document breakdown
- 🎨 Beautiful CLI with colored output
- 🌐 JSON output format for automation
- ⚡ Fast performance with Rust
- 🔄 CI/CD ready with proper exit codes

---

**Made with ❤️ and Rust**