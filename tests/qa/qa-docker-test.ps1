# Docker Container Test Script for Tree CLI (Windows PowerShell)
# This script runs inside Windows Docker containers to perform the actual QA testing

param(
    [switch]$Win7Test = $false,
    [switch]$Verbose = $false
)

# Test counters
$script:TotalTests = 0
$script:PassedTests = 0
$script:FailedTests = 0

function Write-Log {
    param($Message)
    $timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
    $logEntry = "$timestamp - $Message"
    Write-Output $logEntry
}

function Write-TestLog {
    param($Message)
    $timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
    $logEntry = "$timestamp - TEST: $Message"
    if ($Verbose) { Write-Output $logEntry }
}

function Write-TestResult {
    param(
        [string]$Result,
        [string]$TestName,
        [string]$Details = ""
    )
    
    $script:TotalTests++
    
    if ($Result -eq "PASS") {
        $script:PassedTests++
        Write-Host "✓ PASS: $TestName" -ForegroundColor Green
    } else {
        $script:FailedTests++
        Write-Host "✗ FAIL: $TestName" -ForegroundColor Red
        if ($Details) {
            Write-Host "  Details: $Details" -ForegroundColor Yellow
        }
    }
    
    # Results are output to console for Docker logging
}

function Invoke-Test {
    param(
        [string]$TestName,
        [string]$Command,
        [string]$ExpectedPattern = "",
        [bool]$ShouldFail = $false
    )
    
    Write-TestLog $TestName
    Write-Host "  Command: $Command"
    
    try {
        if ($ShouldFail) {
            $output = Invoke-Expression $Command 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-TestResult "FAIL" $TestName "Command should have failed but succeeded"
                Write-Host "  Exit code: $LASTEXITCODE"
                Write-Host "  Output: $($output | Select-Object -First 3)"
            } else {
                Write-TestResult "PASS" $TestName
                Write-Host "  Exit code: $LASTEXITCODE"
            }
        } else {
            $output = Invoke-Expression $Command 2>&1
            if ($LASTEXITCODE -eq 0) {
                if ($ExpectedPattern -and $output -notmatch $ExpectedPattern) {
                    Write-TestResult "FAIL" $TestName "Output doesn't match expected pattern: $ExpectedPattern"
                    Write-Host "  Exit code: $LASTEXITCODE"
                    Write-Host "  Expected pattern: '$ExpectedPattern'"
                    Write-Host "  Actual output:"
                    ($output | Select-Object -First 5) | ForEach-Object { Write-Host "    $_" }
                } else {
                    Write-TestResult "PASS" $TestName
                }
            } else {
                Write-TestResult "FAIL" $TestName "Command failed unexpectedly"
                Write-Host "  Exit code: $LASTEXITCODE"
                Write-Host "  Output: $($output | Select-Object -First 3)"
            }
        }
    }
    catch {
        Write-TestResult "FAIL" $TestName "Exception: $($_.Exception.Message)"
        Write-Host "  Exception details: $($_.Exception.Message)"
    }
}

function Setup-TestEnvironment {
    Write-Log "Setting up test environment..."
    
    # Create test directory structure in container temp
    $script:TestDir = "C:\temp\qa_test"
    New-Item -ItemType Directory -Path $TestDir -Force | Out-Null
    Set-Location $TestDir
    
    # Create various test files and directories
    New-Item -ItemType Directory -Path "dir1\subdir1", "dir1\subdir2", "dir2" -Force | Out-Null
    New-Item -ItemType File -Path "file1.txt", "file2.log", "file3.py" | Out-Null
    New-Item -ItemType File -Path "dir1\file_in_dir1.txt", "dir1\subdir1\deep_file.txt" | Out-Null
    
    # Create files with different sizes
    "small file" | Out-File -FilePath "small.txt" -Encoding ASCII
    $largeContent = "0" * 10240  # 10KB file
    $largeContent | Out-File -FilePath "large.bin" -Encoding ASCII
    
    # Set different modification times
    $oldDate = Get-Date "2023-01-01"
    $oldFile = New-Item -ItemType File -Path "old_file.txt"
    $oldFile.LastWriteTime = $oldDate
    
    New-Item -ItemType File -Path "new_file.txt" | Out-Null
    
    # Create files for pattern testing
    New-Item -ItemType File -Path "script.ps1", "config.ini", "README.md" | Out-Null
    
    Write-Log "Test environment created in: $(Get-Location)"
}

function Cleanup-TestEnvironment {
    Write-Log "Cleaning up test environment..."
    Set-Location C:\app
    Remove-Item -Path $TestDir -Recurse -Force -ErrorAction SilentlyContinue
    Write-Log "Test environment cleaned up"
}

function Build-Binary {
    param([bool]$UseWin7Lock = $false)
    
    if ($UseWin7Lock) {
        Write-Log "Building tree binary with Windows 7 compatibility..."
        # Check if Rust 1.75.0 is available
        $rustVersions = rustup toolchain list
        if ($rustVersions -notmatch "1\.75\.0") {
            Write-Log "Installing Rust 1.75.0 for Windows 7 compatibility..."
            rustup install 1.75.0
        }
        
        # Backup current Cargo.lock and use Win7 lock
        if (Test-Path "Cargo.lock") {
            Copy-Item "Cargo.lock" "Cargo.lock.backup"
        }
        Copy-Item "Cargo-win7.lock" "Cargo.lock"
        
        $buildResult = rustup run 1.75.0 cargo build --release 2>&1
        
        # Restore original Cargo.lock
        if (Test-Path "Cargo.lock.backup") {
            Move-Item "Cargo.lock.backup" "Cargo.lock" -Force
        }
    } else {
        Write-Log "Building tree binary..."
        $buildResult = cargo build --release 2>&1
    }
    
    if ($LASTEXITCODE -eq 0) {
        $script:TreeBinary = "C:\app\target\release\tree.exe"
        Write-Log "Binary built successfully: $TreeBinary"
        
        # Get binary info
        if (Test-Path $TreeBinary) {
            $binaryInfo = Get-Item $TreeBinary | Select-Object Name, Length, LastWriteTime
            Write-Log "Binary info: $($binaryInfo | Out-String)"
        }
        
        return $true
    } else {
        Write-Log "Failed to build binary"
        return $false
    }
}

function Test-BasicFunctionality {
    Write-Log "Testing basic functionality..."
    
    Invoke-Test "Basic tree display" "& '$TreeBinary' ." "dir1"
    Invoke-Test "Help flag" "& '$TreeBinary' --help" "USAGE"
    Invoke-Test "Version flag" "& '$TreeBinary' --version" ""
    
    # Test with non-existent directory
    Invoke-Test "Non-existent directory" "& '$TreeBinary' 'C:\non\existent\path'" "" $true
}

function Test-DepthAndStructure {
    Write-Log "Testing depth and structure options..."
    
    Invoke-Test "Depth level 1" "& '$TreeBinary' -L 1 ." "dir1"
    Invoke-Test "Depth level 2" "& '$TreeBinary' --level=2 ." "subdir1"
    Invoke-Test "No indentation" "& '$TreeBinary' -i ." ""
    Invoke-Test "Full path" "& '$TreeBinary' -f ." ".\"
    Invoke-Test "ASCII characters" "& '$TreeBinary' -A ." ""
}

function Test-FileFiltering {
    Write-Log "Testing file filtering options..."
    
    Invoke-Test "Directories only" "& '$TreeBinary' -d ." "dir1"
    Invoke-Test "Pattern include .txt files" "& '$TreeBinary' -P '*.txt' ." "file1.txt"
    Invoke-Test "Pattern exclude .log files" "& '$TreeBinary' -I '*.log' ." ""
    Invoke-Test "File limit" "& '$TreeBinary' --filelimit=5 ." ""
}

function Test-DisplayOptions {
    Write-Log "Testing display options..."
    
    Invoke-Test "Show file sizes" "& '$TreeBinary' -s ." ""
    Invoke-Test "Human readable sizes" "& '$TreeBinary' -H ." ""
    Invoke-Test "Color output" "& '$TreeBinary' -C ." ""
    Invoke-Test "No color output" "& '$TreeBinary' -n ." ""
    Invoke-Test "File type indicators" "& '$TreeBinary' -F ." ""
    Invoke-Test "Show permissions" "& '$TreeBinary' -p ." ""
    Invoke-Test "Show modification dates" "& '$TreeBinary' -D ." ""
}

function Test-SortingOptions {
    Write-Log "Testing sorting options..."
    
    Invoke-Test "Sort by time" "& '$TreeBinary' -t ." ""
    Invoke-Test "Reverse sort" "& '$TreeBinary' -r ." ""
    Invoke-Test "Directories first" "& '$TreeBinary' --dirsfirst ." ""
    Invoke-Test "No report" "& '$TreeBinary' --noreport ." ""
}

function Test-OutputOptions {
    Write-Log "Testing output options..."
    
    $outputFile = "test_output.txt"
    Invoke-Test "Output to file" "& '$TreeBinary' -o $outputFile ." ""
    
    if (Test-Path $outputFile) {
        $fileSize = (Get-Item $outputFile).Length
        if ($fileSize -gt 0) {
            Write-TestResult "PASS" "Output file created and has content"
        } else {
            Write-TestResult "FAIL" "Output file created but is empty"
        }
        Remove-Item $outputFile -Force
    } else {
        Write-TestResult "FAIL" "Output file was not created"
    }
}

function Test-FromfileFunctionality {
    Write-Log "Testing fromfile functionality..."
    
    # Create a simple file listing
    @"
dir1/
dir1/file1.txt
dir2/
file1.txt
file2.txt
"@ | Out-File -FilePath "file_list.txt" -Encoding ASCII
    
    Invoke-Test "Read from file" "& '$TreeBinary' --fromfile file_list.txt" "dir1"
    
    # Test reading from stdin (PowerShell equivalent)
    $stdinContent = "test/`ntest/file.txt"
    Invoke-Test "Read from stdin" "echo '$stdinContent' | & '$TreeBinary' --fromfile" "test"
    
    Remove-Item "file_list.txt" -Force
}

function Test-CombinedOptions {
    Write-Log "Testing combined options..."
    
    Invoke-Test "Multiple flags combination" "& '$TreeBinary' -f -s -L 2 ." ""
    Invoke-Test "Long flags combination" "& '$TreeBinary' --full-path --size --level=2 ." ""
    Invoke-Test "Pattern with exclusion" "& '$TreeBinary' -P '*.txt' -I 'large*' ." ""
}

function Test-ErrorConditions {
    Write-Log "Testing error conditions..."
    
    Invoke-Test "Invalid flag" "& '$TreeBinary' --invalid-flag ." "" $true
    Invoke-Test "Invalid level value" "& '$TreeBinary' -L abc ." "" $true
    Invoke-Test "Invalid file limit" "& '$TreeBinary' --filelimit=-1 ." "" $true
}

function Show-Summary {
    Write-Log ""
    Write-Log "========================================="
    Write-Log "QA Test Summary"
    Write-Log "========================================="
    Write-Log "Total tests: $TotalTests"
    Write-Log "Passed: $PassedTests"
    Write-Log "Failed: $FailedTests"
    if ($TotalTests -gt 0) {
        $successRate = [math]::Round(($PassedTests * 100 / $TotalTests), 2)
        Write-Log "Success rate: $successRate%"
    }
    Write-Log "Log file: $LogFile"
    Write-Log "========================================="
    
    Write-Host ""
    Write-Host "=========================================" -ForegroundColor Blue
    Write-Host "QA Test Summary" -ForegroundColor Blue
    Write-Host "=========================================" -ForegroundColor Blue
    Write-Host "Total tests: " -NoNewline; Write-Host $TotalTests -ForegroundColor Yellow
    Write-Host "Passed: " -NoNewline; Write-Host $PassedTests -ForegroundColor Green
    Write-Host "Failed: " -NoNewline; Write-Host $FailedTests -ForegroundColor Red
    if ($TotalTests -gt 0) {
        $successRate = [math]::Round(($PassedTests * 100 / $TotalTests), 2)
        Write-Host "Success rate: " -NoNewline; Write-Host "$successRate%" -ForegroundColor Yellow
    }
    Write-Host "Log file: " -NoNewline; Write-Host $LogFile -ForegroundColor Blue
    Write-Host "=========================================" -ForegroundColor Blue
    
    if ($FailedTests -gt 0) {
        exit 1
    }
}

function Main {
    Write-Log "Starting QA tests for tree CLI (Windows)"
    Write-Log "Platform: $([System.Environment]::OSVersion.VersionString)"
    Write-Log "PowerShell Version: $($PSVersionTable.PSVersion)"
    Write-Log "Date: $(Get-Date)"
    if ($Win7Test) {
        Write-Log "Windows 7 compatibility testing enabled"
    }
    Write-Log ""
    
    # Build the binary
    if (-not (Build-Binary -UseWin7Lock $Win7Test)) {
        Write-Log "Failed to build binary, exiting"
        exit 1
    }
    
    # Setup test environment
    Setup-TestEnvironment
    
    # Run all test suites
    Test-BasicFunctionality
    Test-DepthAndStructure
    Test-FileFiltering
    Test-DisplayOptions
    Test-SortingOptions
    Test-OutputOptions
    Test-FromfileFunctionality
    Test-CombinedOptions
    Test-ErrorConditions
    
    # Cleanup
    Cleanup-TestEnvironment
    
    # Print summary
    Show-Summary
}

# Run main function
try {
    Main
}
catch {
    Write-Log "Unhandled exception: $($_.Exception.Message)"
    Write-Host "Unhandled exception: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}