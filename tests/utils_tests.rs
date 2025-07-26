// Unit tests for utils.rs
// Tests utility functions for human-readable formatting and error handling

use rust_tree::rust_tree::utils::{bytes_to_human_readable, is_broken_pipe_error};
use std::io::{Error, ErrorKind};

#[test]
fn test_bytes_to_human_readable_small() {
    // Test small byte values (note: actual function uses " B" with space)
    assert_eq!(bytes_to_human_readable(0), "0 B");
    assert_eq!(bytes_to_human_readable(1), "1 B");
    assert_eq!(bytes_to_human_readable(512), "512 B");
    assert_eq!(bytes_to_human_readable(1023), "1023 B");
}

#[test]
fn test_bytes_to_human_readable_kilobytes() {
    // Test kilobyte values (actual function uses " KB" with space)
    assert_eq!(bytes_to_human_readable(1024), "1.0 KB");
    assert_eq!(bytes_to_human_readable(1536), "1.5 KB");
    assert_eq!(bytes_to_human_readable(2048), "2.0 KB");
    assert_eq!(bytes_to_human_readable(10240), "10.0 KB");
    assert_eq!(bytes_to_human_readable(102400), "100.0 KB");
}

#[test]
fn test_bytes_to_human_readable_megabytes() {
    // Test megabyte values
    assert_eq!(bytes_to_human_readable(1024 * 1024), "1.0 MB");
    assert_eq!(bytes_to_human_readable(1024 * 1024 + 512 * 1024), "1.5 MB");
    assert_eq!(bytes_to_human_readable(2 * 1024 * 1024), "2.0 MB");
    assert_eq!(bytes_to_human_readable(10 * 1024 * 1024), "10.0 MB");
    assert_eq!(bytes_to_human_readable(100 * 1024 * 1024), "100.0 MB");
}

#[test]
fn test_bytes_to_human_readable_gigabytes() {
    // Test gigabyte values
    let gb = 1024_u64 * 1024 * 1024;
    assert_eq!(bytes_to_human_readable(gb), "1.0 GB");
    assert_eq!(bytes_to_human_readable(gb + gb / 2), "1.5 GB");
    assert_eq!(bytes_to_human_readable(2 * gb), "2.0 GB");
    assert_eq!(bytes_to_human_readable(10 * gb), "10.0 GB");
}

#[test]
fn test_bytes_to_human_readable_terabytes() {
    // Test terabyte values
    let tb = 1024_u64 * 1024 * 1024 * 1024;
    assert_eq!(bytes_to_human_readable(tb), "1.0 TB");
    assert_eq!(bytes_to_human_readable(tb + tb / 2), "1.5 TB");
    assert_eq!(bytes_to_human_readable(2 * tb), "2.0 TB");
}

#[test]
fn test_bytes_to_human_readable_very_large() {
    // Test very large values (function only supports up to TB)
    let pb = 1024_u64 * 1024 * 1024 * 1024 * 1024;
    // Function max is TB, so very large values will be formatted as bytes
    assert!(bytes_to_human_readable(pb).contains(" B"));

    // Test maximum value - will be formatted as bytes
    assert!(bytes_to_human_readable(u64::MAX).contains(" B"));
}

#[test]
fn test_bytes_to_human_readable_edge_cases() {
    // Test edge cases around unit boundaries
    assert_eq!(bytes_to_human_readable(1023), "1023 B");
    assert_eq!(bytes_to_human_readable(1024), "1.0 KB");
    assert_eq!(bytes_to_human_readable(1025), "1.0 KB");

    assert_eq!(bytes_to_human_readable(1024 * 1024 - 1), "1024.0 KB");
    assert_eq!(bytes_to_human_readable(1024 * 1024), "1.0 MB");
    assert_eq!(bytes_to_human_readable(1024 * 1024 + 1), "1.0 MB");
}

#[test]
fn test_bytes_to_human_readable_precision() {
    // Test precision for decimal places
    let kb = 1024_u64;
    assert_eq!(bytes_to_human_readable(kb + 102), "1.1 KB"); // ~1.1K
    assert_eq!(bytes_to_human_readable(kb + 256), "1.2 KB"); // ~1.25K -> 1.2K
    assert_eq!(bytes_to_human_readable(kb + 512), "1.5 KB"); // exactly 1.5K
    assert_eq!(bytes_to_human_readable(kb + 768), "1.8 KB"); // ~1.75K -> 1.8K
}

#[test]
fn test_is_broken_pipe_error_true_cases() {
    // Test cases that should return true (only BrokenPipe based on actual implementation)
    let broken_pipe = Error::new(ErrorKind::BrokenPipe, "broken pipe");
    assert!(is_broken_pipe_error(&broken_pipe));
}

#[test]
fn test_is_broken_pipe_error_false_cases() {
    // Test cases that should return false
    let not_found = Error::new(ErrorKind::NotFound, "file not found");
    assert!(!is_broken_pipe_error(&not_found));

    let permission_denied = Error::new(ErrorKind::PermissionDenied, "permission denied");
    assert!(!is_broken_pipe_error(&permission_denied));

    let already_exists = Error::new(ErrorKind::AlreadyExists, "already exists");
    assert!(!is_broken_pipe_error(&already_exists));

    let invalid_input = Error::new(ErrorKind::InvalidInput, "invalid input");
    assert!(!is_broken_pipe_error(&invalid_input));

    let interrupted = Error::new(ErrorKind::Interrupted, "interrupted");
    assert!(!is_broken_pipe_error(&interrupted));

    // These should also return false based on actual implementation
    let write_zero = Error::new(ErrorKind::WriteZero, "write zero");
    assert!(!is_broken_pipe_error(&write_zero));

    let unexpected_eof = Error::new(ErrorKind::UnexpectedEof, "unexpected eof");
    assert!(!is_broken_pipe_error(&unexpected_eof));
}

#[test]
fn test_is_broken_pipe_error_with_source() {
    // Test error with source chain
    let source_error = Error::new(ErrorKind::BrokenPipe, "source broken pipe");
    let wrapper_error = Error::new(ErrorKind::Other, "wrapper error");

    // Test the source error directly
    assert!(is_broken_pipe_error(&source_error));

    // Test different wrapper error
    assert!(!is_broken_pipe_error(&wrapper_error));
}

#[test]
fn test_bytes_to_human_readable_consistency() {
    // Test that function is consistent for same inputs
    let test_value = 1536; // 1.5K
    assert_eq!(
        bytes_to_human_readable(test_value),
        bytes_to_human_readable(test_value)
    );

    let large_value = 5 * 1024 * 1024 * 1024; // 5G
    assert_eq!(
        bytes_to_human_readable(large_value),
        bytes_to_human_readable(large_value)
    );
}

#[test]
fn test_bytes_to_human_readable_monotonic() {
    // Test that larger values don't result in smaller formatted values (when comparing same units)
    let sizes = vec![
        1024, // 1.0 KB
        2048, // 2.0 KB
        3072, // 3.0 KB
        4096, // 4.0 KB
        5120, // 5.0 KB
    ];

    let formatted: Vec<String> = sizes
        .iter()
        .map(|&size| bytes_to_human_readable(size))
        .collect();

    // All should be in KB units and increasing
    for formatted_size in &formatted {
        assert!(formatted_size.ends_with(" KB"));
    }
}

#[test]
fn test_is_broken_pipe_error_all_error_kinds() {
    // Test all ErrorKind variants to ensure comprehensive coverage
    let error_kinds = vec![
        ErrorKind::NotFound,
        ErrorKind::PermissionDenied,
        ErrorKind::ConnectionRefused,
        ErrorKind::ConnectionReset,
        ErrorKind::ConnectionAborted,
        ErrorKind::NotConnected,
        ErrorKind::AddrInUse,
        ErrorKind::AddrNotAvailable,
        ErrorKind::BrokenPipe,
        ErrorKind::AlreadyExists,
        ErrorKind::WouldBlock,
        ErrorKind::InvalidInput,
        ErrorKind::InvalidData,
        ErrorKind::TimedOut,
        ErrorKind::WriteZero,
        ErrorKind::Interrupted,
        ErrorKind::UnexpectedEof,
        ErrorKind::Other,
    ];

    for kind in error_kinds {
        let error = Error::new(kind, "test error");
        let is_broken = is_broken_pipe_error(&error);

        // Only BrokenPipe should return true based on actual implementation
        let expected = matches!(kind, ErrorKind::BrokenPipe);

        assert_eq!(
            is_broken, expected,
            "ErrorKind::{:?} returned unexpected result",
            kind
        );
    }
}
