use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn setup_test_environment() -> Result<String> {
    let setup_script = r#"#!/bin/bash
set -e

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log "Setting up test environment..."

# Create test directory structure in container temp
TEST_DIR="/tmp/qa_test"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Create various test files and directories
mkdir -p dir1/subdir1 dir1/subdir2 dir2 .hidden_dir
touch file1.txt file2.log file3.py .hidden_file
touch dir1/file_in_dir1.txt dir1/subdir1/deep_file.txt
touch .hidden_dir/hidden_file.txt

# Create files with different sizes
echo "small file" > small.txt
dd if=/dev/zero of=large.bin bs=1024 count=10 2>/dev/null

# Create symlinks (if supported)
if ln -s file1.txt symlink.txt 2>/dev/null; then
    log "Created symbolic links for testing"
fi

# Set different modification times
touch -t 202301010000 old_file.txt
touch new_file.txt

# Create files for pattern testing
touch script.sh config.ini README.md

# Create file listing for fromfile tests
cat > /tmp/file_list.txt << 'EOF'
dir1/
dir1/file_in_dir1.txt
dir2/
file1.txt
file2.log
EOF

log "Test environment created in: $(pwd)"
log "Files created:"
ls -la

# Build the binary
log "Building tree binary..."
cd /app

# Clean and build release binary
cargo clean >/dev/null 2>&1
if cargo build --release >/dev/null 2>&1; then
    TREE_BINARY="/app/target/release/tree"
    log "Binary built successfully: $TREE_BINARY"
    
    # Get binary info
    ls -la "$TREE_BINARY"
    
    # Return to test directory
    cd "$TEST_DIR"
    
    log "Setup completed successfully"
else
    log "ERROR: Failed to build binary"
    exit 1
fi
"#;

    Ok(setup_script.to_string())
}

pub fn create_windows_setup_script() -> Result<String> {
    let setup_script = r#"# PowerShell test environment setup
$ErrorActionPreference = "Stop"

function Log($message) {
    Write-Host "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') - $message"
}

Log "Setting up Windows test environment..."

# Create test directory structure
$TEST_DIR = "C:\temp\qa_test"
if (Test-Path $TEST_DIR) {
    Remove-Item -Recurse -Force $TEST_DIR
}
New-Item -ItemType Directory -Force -Path $TEST_DIR | Out-Null
Set-Location $TEST_DIR

# Create various test files and directories
New-Item -ItemType Directory -Force -Path "dir1\subdir1", "dir1\subdir2", "dir2" | Out-Null
New-Item -ItemType File -Force -Path "file1.txt", "file2.log", "file3.py" | Out-Null
New-Item -ItemType File -Force -Path "dir1\file_in_dir1.txt", "dir1\subdir1\deep_file.txt" | Out-Null

# Create files with different sizes  
"small file" | Out-File -FilePath "small.txt" -Encoding ASCII
$bytes = New-Object byte[] 10240
[System.IO.File]::WriteAllBytes("$TEST_DIR\large.bin", $bytes)

# Set different modification times
$oldTime = Get-Date "2023-01-01 00:00:00"
New-Item -ItemType File -Force -Path "old_file.txt" | Out-Null
(Get-Item "old_file.txt").LastWriteTime = $oldTime
New-Item -ItemType File -Force -Path "new_file.txt" | Out-Null

# Create files for pattern testing
New-Item -ItemType File -Force -Path "script.ps1", "config.ini", "README.md" | Out-Null

# Create file listing for fromfile tests
@"
dir1\
dir1\file_in_dir1.txt
dir2\
file1.txt
file2.log
"@ | Out-File -FilePath "C:\temp\file_list.txt" -Encoding ASCII

Log "Test environment created in: $TEST_DIR"
Log "Files created:"
Get-ChildItem -Force

# Build the binary
Log "Building tree binary..."
Set-Location C:\app

# Clean and build release binary
& cargo clean 2>&1 | Out-Null
if (& cargo build --release 2>&1) {
    $TREE_BINARY = "C:\app\target\release\tree.exe"
    Log "Binary built successfully: $TREE_BINARY"
    
    # Get binary info
    Get-Item $TREE_BINARY
    
    # Return to test directory  
    Set-Location $TEST_DIR
    
    Log "Setup completed successfully"
} else {
    Log "ERROR: Failed to build binary"
    exit 1
}
"#;

    Ok(setup_script.to_string())
}