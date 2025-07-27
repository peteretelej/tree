#!/bin/bash

# Docker-based QA Test Orchestrator for Tree CLI
# Runs comprehensive tests across multiple platforms using Docker containers

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
QA_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMP_DIR="$QA_DIR/temp"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Test platforms
PLATFORMS=("linux" "alpine")
SELECTED_PLATFORMS=()

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Docker-based QA testing for tree CLI across multiple platforms"
    echo ""
    echo "Options:"
    echo "  --all           Run tests on all platforms (linux, alpine)"
    echo "  --linux         Run tests on Ubuntu Linux"
    echo "  --alpine        Run tests on Alpine Linux"
    echo "  --windows       Run tests on Windows (requires Windows Docker)"
    echo "  --clean         Clean up Docker images and containers"
    echo "  --logs          Show test logs directory"
    echo "  --help, -h      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                          # Test all platforms (default)"
    echo "  $0 --linux --alpine        # Test specific platforms"
    echo "  $0 --all                   # Explicitly test all platforms"
    echo "  $0 --clean                 # Clean up Docker resources"
    echo ""
}

log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[$(date '+%H:%M:%S')] ✓${NC} $1"
}

log_error() {
    echo -e "${RED}[$(date '+%H:%M:%S')] ✗${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[$(date '+%H:%M:%S')] ⚠${NC} $1"
}

cleanup_docker() {
    log "Cleaning up Docker resources..."
    
    # Remove containers
    for platform in "${PLATFORMS[@]}" "windows"; do
        container_name="tree-qa-$platform"
        if docker ps -a --format "table {{.Names}}" | grep -q "^$container_name$"; then
            log "Removing container: $container_name"
            docker rm -f "$container_name" >/dev/null 2>&1 || true
        fi
    done
    
    # Remove images
    for platform in "${PLATFORMS[@]}" "windows"; do
        image_name="tree-qa:$platform"
        if docker images --format "table {{.Repository}}:{{.Tag}}" | grep -q "^$image_name$"; then
            log "Removing image: $image_name"
            docker rmi "$image_name" >/dev/null 2>&1 || true
        fi
    done
    
    log_success "Docker cleanup completed"
}

build_docker_image() {
    local platform="$1"
    local dockerfile="Dockerfile.$platform"
    local image_name="tree-qa:$platform"
    
    log "Building Docker image for $platform..."
    
    if ! docker build -t "$image_name" -f "$QA_DIR/$dockerfile" "$PROJECT_ROOT"; then
        log_error "Failed to build Docker image for $platform"
        return 1
    fi
    
    log_success "Built Docker image: $image_name"
    return 0
}

run_platform_test() {
    local platform="$1"
    local image_name="tree-qa:$platform"
    local container_name="tree-qa-$platform-$TIMESTAMP"
    local log_file="$TEMP_DIR/qa-$platform-$TIMESTAMP.log"
    
    # Ensure temp directory exists
    mkdir -p "$TEMP_DIR"
    
    log "Running QA tests for $platform..."
    
    # Run the container
    if docker run --name "$container_name" --rm "$image_name" > "$log_file" 2>&1; then
        log_success "QA tests passed for $platform"
        
        # Show summary from log
        if grep -q "QA Test Summary" "$log_file"; then
            echo ""
            echo -e "${BLUE}=== $platform Test Summary ===${NC}"
            grep -A 10 "QA Test Summary" "$log_file" | head -10
            echo ""
        fi
        
        return 0
    else
        log_error "QA tests failed for $platform"
        log_error "Check log file: $log_file"
        
        # Show last few lines of log for immediate debugging
        echo ""
        echo -e "${RED}=== Last 10 lines of $platform test log ===${NC}"
        tail -10 "$log_file"
        echo ""
        
        return 1
    fi
}

run_windows_test() {
    local platform="windows"
    local image_name="tree-qa:$platform"
    local container_name="tree-qa-$platform-$TIMESTAMP"
    local log_file="$TEMP_DIR/qa-$platform-$TIMESTAMP.log"
    
    # Check if we're on Windows or have Windows containers enabled
    if ! docker system info | grep -q "OSType.*windows"; then
        log_warning "Windows containers not available. Skipping Windows tests."
        log_warning "To run Windows tests, use a Windows Docker host or enable Windows containers."
        return 0
    fi
    
    # Ensure temp directory exists
    mkdir -p "$TEMP_DIR"
    
    log "Building Docker image for Windows..."
    if ! docker build -t "$image_name" -f "$QA_DIR/Dockerfile.windows" "$PROJECT_ROOT"; then
        log_error "Failed to build Docker image for Windows"
        return 1
    fi
    
    log "Running QA tests for Windows..."
    
    # Run the container
    if docker run --name "$container_name" --rm "$image_name" > "$log_file" 2>&1; then
        log_success "QA tests passed for Windows"
        
        # Show summary from log
        if grep -q "QA Test Summary" "$log_file"; then
            echo ""
            echo -e "${BLUE}=== Windows Test Summary ===${NC}"
            grep -A 10 "QA Test Summary" "$log_file" | head -10
            echo ""
        fi
        
        return 0
    else
        log_error "QA tests failed for Windows"
        log_error "Check log file: $log_file"
        
        # Show last few lines of log for immediate debugging
        echo ""
        echo -e "${RED}=== Last 10 lines of Windows test log ===${NC}"
        tail -10 "$log_file"
        echo ""
        
        return 1
    fi
}

main() {
    local run_all=false
    local run_windows=false
    local do_cleanup=false
    local show_logs=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --all)
                run_all=true
                shift
                ;;
            --linux)
                SELECTED_PLATFORMS+=("linux")
                shift
                ;;
            --alpine)
                SELECTED_PLATFORMS+=("alpine")
                shift
                ;;
            --windows)
                run_windows=true
                shift
                ;;
            --clean)
                do_cleanup=true
                shift
                ;;
            --logs)
                show_logs=true
                shift
                ;;
            --help|-h)
                usage
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    # Handle special commands
    if [ "$do_cleanup" = true ]; then
        cleanup_docker
        exit 0
    fi
    
    if [ "$show_logs" = true ]; then
        echo "Test temp directory: $TEMP_DIR"
        if [ -d "$TEMP_DIR" ]; then
            echo "Available files:"
            ls -la "$TEMP_DIR"
        else
            echo "No temp directory found."
        fi
        exit 0
    fi
    
    # Determine which platforms to test
    if [ "$run_all" = true ]; then
        SELECTED_PLATFORMS=("${PLATFORMS[@]}")
    fi
    
    # Default to running all platforms if none specified
    if [ ${#SELECTED_PLATFORMS[@]} -eq 0 ] && [ "$run_windows" = false ]; then
        SELECTED_PLATFORMS=("${PLATFORMS[@]}")
        log "No specific platforms selected, running all platforms by default"
    fi
    
    log "Starting Docker-based QA testing for tree CLI"
    log "Timestamp: $TIMESTAMP"
    log "Project root: $PROJECT_ROOT"
    log "Temp directory: $TEMP_DIR"
    echo ""
    
    # Ensure we're in the right directory
    cd "$PROJECT_ROOT"
    
    # Test selected platforms
    local overall_success=true
    
    for platform in "${SELECTED_PLATFORMS[@]}"; do
        echo ""
        log "=== Testing platform: $platform ==="
        
        if ! build_docker_image "$platform"; then
            overall_success=false
            continue
        fi
        
        if ! run_platform_test "$platform"; then
            overall_success=false
        fi
    done
    
    # Test Windows if requested
    if [ "$run_windows" = true ]; then
        echo ""
        log "=== Testing platform: Windows ==="
        
        if ! run_windows_test; then
            overall_success=false
        fi
    fi
    
    # Final summary
    echo ""
    echo "========================================"
    if [ "$overall_success" = true ]; then
        log_success "All QA tests completed successfully!"
    else
        log_error "Some QA tests failed. Check individual platform logs."
    fi
    
    echo ""
    log "Test logs available in: $TEMP_DIR"
    echo "========================================"
    
    if [ "$overall_success" = false ]; then
        exit 1
    fi
}

# Run main function
main "$@"