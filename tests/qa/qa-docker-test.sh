#!/bin/bash

# Docker Container Test Script for Tree CLI
# This script runs inside Docker containers to perform the actual QA testing

set -e

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_test() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - TEST: $1"
}

log_result() {
    local result="$1"
    local test_name="$2"
    local details="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$result" = "PASS" ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo "✓ PASS: $test_name"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "✗ FAIL: $test_name"
        if [ -n "$details" ]; then
            echo "  Details: $details"
        fi
    fi
}

run_test() {
    local test_name="$1"
    local command="$2"
    local expected_pattern="$3"
    local should_fail="$4"
    
    log_test "$test_name"
    echo "  Command: $command"
    
    if [ "$should_fail" = "true" ]; then
        if output=$($command 2>&1); then
            exit_code=$?
            log_result "FAIL" "$test_name" "Command should have failed but succeeded"
            echo "  Exit code: $exit_code"
            echo "  Output: $output" | head -3
        else
            exit_code=$?
            log_result "PASS" "$test_name"
            echo "  Exit code: $exit_code"
        fi
    else
        if output=$($command 2>&1); then
            exit_code=$?
            if [ -n "$expected_pattern" ]; then
                if echo "$output" | grep -q "$expected_pattern"; then
                    log_result "PASS" "$test_name"
                else
                    log_result "FAIL" "$test_name" "Output doesn't match expected pattern: $expected_pattern"
                    echo "  Exit code: $exit_code"
                    echo "  Expected pattern: '$expected_pattern'"
                    echo "  Actual output:"
                    echo "$output" | head -5 | sed 's/^/    /'
                fi
            else
                log_result "PASS" "$test_name"
            fi
        else
            exit_code=$?
            log_result "FAIL" "$test_name" "Command failed unexpectedly"
            echo "  Exit code: $exit_code"
            echo "  Output: $output" | head -3
        fi
    fi
}

setup_test_environment() {
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
    
    log "Test environment created in: $(pwd)"
}

build_binary() {
    log "Building tree binary..."
    cd /app
    
    # Clean and build release binary
    cargo clean >/dev/null 2>&1
    if cargo build --release >/dev/null 2>&1; then
        TREE_BINARY="/app/target/release/tree"
        log "Binary built successfully: $TREE_BINARY"
        
        # Get binary info
        ls -la "$TREE_BINARY"
        
        return 0
    else
        log "Failed to build binary"
        return 1
    fi
}

test_basic_functionality() {
    log "Testing basic functionality..."
    
    run_test "Basic tree display" "$TREE_BINARY ." "dir1"
    run_test "Help flag" "$TREE_BINARY --help" "tree"
    run_test "Version flag" "$TREE_BINARY --version" ""
    
    # Test with non-existent directory - should fail like standard tree
    run_test "Non-existent directory exit code" "$TREE_BINARY /non/existent/path" "" "true"
}

test_depth_and_structure() {
    log "Testing depth and structure options..."
    
    run_test "Depth level 1" "$TREE_BINARY -L 1 ." "dir1"
    run_test "Depth level 2" "$TREE_BINARY --level=2 ." "subdir1"
    run_test "No indentation" "$TREE_BINARY -i ." ""
    run_test "Full path" "$TREE_BINARY -f ." "./"
    run_test "ASCII characters" "$TREE_BINARY -A ." ""
}

test_file_filtering() {
    log "Testing file filtering options..."
    
    run_test "All files (including hidden)" "$TREE_BINARY -a ." ".hidden"
    run_test "Directories only" "$TREE_BINARY -d ." "dir1"
    run_test "Pattern include .txt files" "$TREE_BINARY -P '*.txt' ." "file1.txt"
    run_test "Pattern exclude .log files" "$TREE_BINARY -I '*.log' ." ""
    run_test "File limit" "$TREE_BINARY --filelimit=5 ." ""
}

test_display_options() {
    log "Testing display options..."
    
    run_test "Show file sizes" "$TREE_BINARY -s ." ""
    run_test "Human readable sizes" "$TREE_BINARY -H ." ""
    run_test "Color output" "$TREE_BINARY -C ." ""
    run_test "No color output" "$TREE_BINARY -n ." ""
    run_test "File type indicators" "$TREE_BINARY -F ." ""
    run_test "Show permissions" "$TREE_BINARY -p ." ""
    run_test "Show modification dates" "$TREE_BINARY -D ." ""
}

test_sorting_options() {
    log "Testing sorting options..."
    
    run_test "Sort by time" "$TREE_BINARY -t ." ""
    run_test "Reverse sort" "$TREE_BINARY -r ." ""
    run_test "Directories first" "$TREE_BINARY --dirsfirst ." ""
    run_test "No report" "$TREE_BINARY --noreport ." ""
}

test_output_options() {
    log "Testing output options..."
    
    OUTPUT_FILE="/tmp/test_output.txt"
    run_test "Output to file" "$TREE_BINARY -o $OUTPUT_FILE ." ""
    
    if [ -f "$OUTPUT_FILE" ]; then
        if [ -s "$OUTPUT_FILE" ]; then
            log_result "PASS" "Output file created and has content"
        else
            log_result "FAIL" "Output file created but is empty"
        fi
        rm -f "$OUTPUT_FILE"
    else
        log_result "FAIL" "Output file was not created"
    fi
}

test_fromfile_functionality() {
    log "Testing fromfile functionality..."
    
    # Create a simple file listing
    cat > /tmp/file_list.txt << 'EOF'
dir1/
dir1/file1.txt
dir2/
file1.txt
file2.txt
EOF
    
    run_test "Read from file" "$TREE_BINARY --fromfile /tmp/file_list.txt" "dir1"
    
    # Test reading from stdin
    run_test "Read from stdin" "echo -e 'test/\\ntest/file.txt' | $TREE_BINARY --fromfile" "test"
    
    rm -f /tmp/file_list.txt
}

test_combined_options() {
    log "Testing combined options..."
    
    run_test "Multiple flags combination" "$TREE_BINARY -f -s -L 2 ." ""
    run_test "Long flags combination" "$TREE_BINARY --full-path --size --level=2 ." ""
    run_test "Pattern with exclusion" "$TREE_BINARY -P '*.txt' -I 'large*' ." ""
}

test_error_conditions() {
    log "Testing error conditions..."
    
    run_test "Invalid flag" "$TREE_BINARY --invalid-flag ." "" "true"
    run_test "Invalid level value" "$TREE_BINARY -L abc ." "" "true"
    run_test "Invalid file limit" "$TREE_BINARY --filelimit=-1 ." "" "true"
}

print_summary() {
    log ""
    log "========================================="
    log "QA Test Summary"
    log "========================================="
    log "Platform: $(uname -a)"
    log "Rust version: $(rustc --version)"
    log "Total tests: $TOTAL_TESTS"
    log "Passed: $PASSED_TESTS"
    log "Failed: $FAILED_TESTS"
    if [ $TOTAL_TESTS -gt 0 ]; then
        log "Success rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    fi
    log "========================================="
    
    if [ $FAILED_TESTS -gt 0 ]; then
        exit 1
    fi
}

main() {
    log "Starting QA tests for tree CLI (Docker container)"
    log "Platform: $(uname -a)"
    log "Date: $(date)"
    log ""
    
    # Build the binary
    if ! build_binary; then
        log "Failed to build binary, exiting"
        exit 1
    fi
    
    # Setup test environment
    setup_test_environment
    
    # Run all test suites
    test_basic_functionality
    test_depth_and_structure
    test_file_filtering
    test_display_options
    test_sorting_options
    test_output_options
    test_fromfile_functionality
    test_combined_options
    test_error_conditions
    
    # Print summary
    print_summary
}

# Run main function
main "$@"