# QA Testing for Tree CLI

This directory contains comprehensive Docker-based QA test scripts for validating the tree CLI functionality across different platforms in isolated containers.

## Purpose

Manual QA testing scripts that validate all features advertised in the README using built binaries in clean Docker environments. These tests are designed to catch real-world usage issues and cross-platform compatibility problems without polluting the host filesystem.

## Docker-Based Testing

All testing runs inside Docker containers to ensure:
- Clean, reproducible environments
- No host filesystem pollution  
- Accurate cross-platform testing
- Isolated build and test environments

## Test Scripts

### `qa-test.sh` (Host orchestrator)
Bash script that orchestrates Docker containers for testing on different platforms.

### `qa-docker-test.sh` (Container test script)
Internal script that runs inside Docker containers to perform the actual testing.

## Features Tested

### Core Functionality
- Basic directory tree display
- Depth control (`-L`/`--level`)
- Full path display (`-f`/`--full-path`)
- No indentation (`-i`/`--no-indent`)
- Hidden files (`-a`/`--all`)

### File Filtering & Pattern Matching
- Include patterns (`-P`/`--pattern`)
- Exclude patterns (`-I`/`--exclude`)
- Directories only (`-d`/`--directories`)
- File limit per directory (`--filelimit`)

### Display Options
- File sizes (`-s`/`--size`)
- Human readable sizes (`-H`/`--human-readable`)
- Colorization (`-C`/`--color`, `-n`/`--no-color`)
- ASCII characters (`-A`/`--ascii`)
- File type indicators (`-F`)
- Permissions (`-p`)
- Modification dates (`-D`)

### Sorting & Output
- Sort by modification time (`-t`)
- Reverse sorting (`-r`)
- Directories first (`--dirsfirst`)
- Output to file (`-o`)
- No summary report (`--noreport`)

### Advanced Features
- Read from file/stdin (`--fromfile`)
- Archive format support (TAR, ZIP, 7z, RAR)

## Test Environment

Each script creates a comprehensive test directory structure with:
- Regular files and directories
- Hidden files (starting with `.`)
- Files with different extensions
- Nested directory structures
- Files with various sizes
- Symbolic links (Unix systems)
- Files with different modification times

## Docker Images

### `Dockerfile.linux` (Ubuntu-based)
Tests on Ubuntu with latest Rust toolchain for modern Linux compatibility.

### `Dockerfile.alpine` (Alpine-based) 
Tests on Alpine Linux for minimal environment and musl compatibility.

### `Dockerfile.windows` (Windows-based)
Tests on Windows Server Core with both modern Rust and Windows 7 compatibility (Rust 1.75).

## Usage

### Run All Platform Tests (Default)
```bash
cd tests/qa
./qa-test.sh
```

### Run Specific Platform
```bash
# Test Ubuntu Linux only
./qa-test.sh --linux

# Test Alpine Linux only
./qa-test.sh --alpine

# Test multiple specific platforms
./qa-test.sh --linux --alpine

# Test Windows (requires Windows Docker)
./qa-test.sh --windows

# Explicitly run all platforms
./qa-test.sh --all
```

### Run with Docker Compose
```bash
docker-compose up --build
```

## Test Reports

Scripts generate detailed logs in the `temp/` directory showing:
- Test results (PASS/FAIL) for each feature
- Error messages and outputs
- Performance timing
- Binary information (version, size, platform)
- Summary report with pass/fail counts

All temporary files (logs, test artifacts, Docker build cache) are consolidated in `tests/qa/temp/` and ignored by Git.

## Requirements

### Host System
- Docker Engine
- Docker Compose (optional, for multi-platform testing)
- Bash (for orchestration scripts)

### Container Environments
- All Rust toolchains and dependencies are installed in containers
- No host Rust installation required
- Windows containers require Windows Docker host for Windows testing

## Notes

- Tests are designed to be run manually, not in CI/CD
- Each platform test runs in complete isolation
- All build artifacts and test files remain in containers
- Host filesystem stays clean - no test pollution
- Tests use release build binaries for realistic performance testing
- Containers are automatically cleaned up after testing