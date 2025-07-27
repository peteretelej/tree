# Tree CLI QA Tool

A comprehensive, automated quality assurance testing tool for the tree CLI application. This Rust-based tool replaces the previous bash-based QA system with better reliability, debugging capabilities, and cross-platform support.

## Overview

The QA tool performs comprehensive testing of the tree CLI across multiple platforms using Docker containers. It validates all major features including directory traversal, file filtering, display options, sorting, and error handling.

## Features

- **Comprehensive Test Coverage**: 33+ tests covering all tree CLI functionality
- **Multi-Platform Support**: Linux, Alpine, and Windows testing
- **Docker Integration**: Isolated, reproducible test environments
- **Structured Reporting**: Detailed test results with categorized summaries
- **Parallel Execution**: Efficient testing across multiple platforms
- **Better Debugging**: Clear error messages and test output analysis

## Installation

### Prerequisites

- Rust (latest stable)
- Docker
- Git

### Build

```bash
cd tools/qa
cargo build --release
```

## Usage

### Basic Commands

```bash
# Run tests on all platforms (Linux and Alpine)
./target/release/qa test

# Run tests on specific platforms
./target/release/qa test --platforms linux
./target/release/qa test --platforms linux,alpine

# Run with verbose output
./target/release/qa test --platforms linux --verbose

# Run all platforms explicitly
./target/release/qa test --all

# Clean up Docker resources
./target/release/qa clean

# Force cleanup without confirmation
./target/release/qa clean --force
```

### Command Reference

#### Test Command

```bash
qa test [OPTIONS]
```

**Options:**
- `--platforms <PLATFORMS>`: Comma-separated list of platforms (linux, alpine, windows)
- `--all`: Run tests on all supported platforms
- `--verbose`: Show detailed test output and container logs

**Examples:**
```bash
# Test single platform
qa test --platforms linux

# Test multiple platforms
qa test --platforms linux,alpine

# Test all platforms with verbose output
qa test --all --verbose
```

#### Clean Command

```bash
qa clean [OPTIONS]
```

**Options:**
- `--force`: Skip confirmation prompt

**Examples:**
```bash
# Interactive cleanup
qa clean

# Force cleanup
qa clean --force
```

## Test Categories

The QA tool executes tests organized into 9 main categories:

### 1. Basic Functionality (4 tests)
- Basic tree display
- Help flag (`--help`)
- Version flag (`--version`)
- Error handling for non-existent paths

### 2. Depth and Structure (5 tests)
- Depth limiting (`-L`, `--level`)
- No indentation (`-i`)
- Full path display (`-f`)
- ASCII character mode (`-A`)

### 3. File Filtering (5 tests)
- Hidden files (`-a`)
- Directories only (`-d`)
- Include patterns (`-P`)
- Exclude patterns (`-I`)
- File limits (`--filelimit`)

### 4. Display Options (7 tests)
- File sizes (`-s`, `-H`)
- Color output (`-C`, `-n`)
- File type indicators (`-F`)
- Permissions (`-p`)
- Modification dates (`-D`)

### 5. Sorting Options (4 tests)
- Time sorting (`-t`)
- Reverse sorting (`-r`)
- Directories first (`--dirsfirst`)
- No report (`--noreport`)

### 6. Output Options (1 test)
- Output to file (`-o`)

### 7. Fromfile Functionality (2 tests)
- Reading from file (`--fromfile`)
- Reading from stdin

### 8. Combined Options (3 tests)
- Multiple flag combinations
- Long flag combinations
- Pattern with exclusion

### 9. Error Conditions (3 tests)
- Invalid flags
- Invalid parameter values
- Error handling validation

## Test Environment

Each test runs in an isolated Docker container with a standardized directory structure:

```
/tmp/qa_test/                    (Linux/Alpine)
├── dir1/
│   ├── subdir1/
│   │   └── deep_file.txt
│   ├── subdir2/
│   └── file_in_dir1.txt
├── dir2/
├── .hidden_dir/
│   └── hidden_file.txt
├── file1.txt
├── file2.log
├── file3.py
├── .hidden_file
├── small.txt
├── large.bin (10KB)
├── old_file.txt (2023-01-01)
├── new_file.txt (current time)
├── script.sh
├── config.ini
├── README.md
└── symlink.txt -> file1.txt
```

## Supported Platforms

### Linux (Ubuntu 22.04)
- Full feature testing
- Symbolic link support
- POSIX compliance validation

### Alpine (Alpine 3.19)
- Musl libc compatibility
- Minimal environment testing
- Static binary validation

### Windows (Server Core LTSC 2022)
- Windows-specific features
- PowerShell-based testing
- Windows 7 compatibility mode

## Output Format

### Test Execution
```
🔬 Tree CLI QA Testing
🐳 Testing platform: linux
  🔨 Building Docker image: tree-qa:linux
  ✅ Image built successfully
  🐳 Creating container: tree-qa-linux-1642857600
  🧪 Running tests in container
  📁 Basic Functionality
    ✅ Basic tree display
    ✅ Help flag
    ✅ Version flag
    ✅ Non-existent directory exit code
  📁 File Filtering
    ✅ All files (including hidden)
    ✅ Directories only
    ✅ Pattern include .log files
    ...
```

### Test Summary
```
==================================================
📊 linux Test Summary
==================================================
Total tests: 33
Passed: 32
Failed: 1
Success rate: 97.0%
Execution time: 45823ms

📁 Category Breakdown:
  Basic Functionality: 4/4 (100.0%)
  Depth and Structure: 5/5 (100.0%)
  File Filtering: 4/5 (80.0%)
  Display Options: 7/7 (100.0%)
  Sorting Options: 4/4 (100.0%)
  Output Options: 1/1 (100.0%)
  Fromfile Functionality: 2/2 (100.0%)
  Combined Options: 3/3 (100.0%)
  Error Conditions: 3/3 (100.0%)
==================================================
```

## Architecture

### Component Overview

```
qa/
├── src/
│   ├── main.rs           # CLI interface and orchestration
│   ├── docker.rs         # Docker API integration
│   ├── tests.rs          # Test definitions and data
│   ├── environment.rs    # Test environment setup
│   ├── executor.rs       # Test execution and reporting
│   └── lib.rs           # Module declarations
├── Cargo.toml           # Dependencies and configuration
└── README.md           # This file
```

### Key Components

1. **Docker Manager**: Handles container lifecycle, image building, and Docker API interactions
2. **Test Executor**: Manages test execution, result parsing, and summary generation
3. **Test Definitions**: Structured test cases with categories and validation logic
4. **Environment Setup**: Standardized test environment creation across platforms

## Benefits Over Bash Implementation

### Reliability
- **Type Safety**: Rust prevents runtime errors that plagued bash scripts
- **Better Error Handling**: Structured error propagation and reporting
- **Consistent Execution**: Eliminates platform-specific bash behavior differences

### Maintainability
- **Clear Structure**: Organized modules with single responsibilities
- **Code Reuse**: Shared logic between platforms and test categories
- **Easy Extension**: Simple addition of new tests and platforms

### Debugging
- **Structured Logging**: Detailed execution traces with timestamps
- **Better Error Messages**: Context-aware error reporting
- **Container Introspection**: Direct Docker API access for debugging

### Performance
- **Parallel Execution**: Concurrent platform testing
- **Efficient Container Management**: Optimized Docker resource usage
- **Fast Feedback**: Quick identification of test failures

## Configuration

### Environment Variables

- `RUST_LOG`: Controls logging level (default: `qa=info`)
- `DOCKER_HOST`: Docker daemon connection (uses Docker defaults)

### Test Customization

Tests are defined in `src/tests.rs` and can be modified to:
- Add new test cases
- Modify expected patterns
- Adjust test categories
- Platform-specific test variations

## Troubleshooting

### Common Issues

**Docker Connection Failed**
```bash
# Check Docker daemon status
sudo systemctl status docker

# Verify Docker access
docker ps
```

**Image Build Failures**
```bash
# Check Dockerfile syntax
docker build -f ../../tests/qa/Dockerfile.linux ../../

# Clean Docker cache
docker system prune -f
```

**Test Failures**
```bash
# Run with verbose output
qa test --platforms linux --verbose

# Check individual test logs
qa test --platforms linux 2>&1 | grep "FAIL"
```

### Debug Tips

1. **Verbose Mode**: Always use `--verbose` for debugging test failures
2. **Single Platform**: Test one platform at a time when debugging
3. **Container Logs**: Failed containers are automatically removed, but logs are captured
4. **Manual Testing**: Use the cleaned bash scripts in `tests/qa/` for manual validation

## Development

### Adding New Tests

1. Define test in `src/tests.rs`:
```rust
TestCase {
    name: "New feature test".to_string(),
    command: "/app/target/release/tree --new-flag .".to_string(),
    expected_pattern: Some("expected_output".to_string()),
    should_fail: false,
    category: TestCategory::DisplayOptions,
}
```

2. Update test environment if needed in `src/environment.rs`
3. Test locally: `cargo run -- test --platforms linux`

### Adding New Platforms

1. Create Dockerfile in `../../tests/qa/Dockerfile.newplatform`
2. Add platform to Docker manager in `src/docker.rs`
3. Create platform-specific environment setup in `src/environment.rs`
4. Test: `cargo run -- test --platforms newplatform`

## Contributing

1. Follow existing code style and structure
2. Add tests for new functionality
3. Update documentation for user-facing changes
4. Test on all supported platforms before submitting

## License

This tool follows the same license as the main tree project (MIT).