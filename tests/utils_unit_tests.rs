use rust_tree::rust_tree::utils::{bytes_to_human_readable, is_broken_pipe_error};
use std::io::{Error, ErrorKind};

#[test]
fn test_is_broken_pipe_error_direct() {
    // Test direct BrokenPipe error
    let broken_pipe_err = Error::new(ErrorKind::BrokenPipe, "pipe broken");
    assert!(is_broken_pipe_error(&broken_pipe_err));

    // Test other error kinds
    let other_err = Error::new(ErrorKind::NotFound, "file not found");
    assert!(!is_broken_pipe_error(&other_err));

    let permission_err = Error::new(ErrorKind::PermissionDenied, "access denied");
    assert!(!is_broken_pipe_error(&permission_err));
}

#[test]
fn test_bytes_to_human_readable() {
    // Test zero bytes
    assert_eq!(bytes_to_human_readable(0), "0 B");

    // Test bytes
    assert_eq!(bytes_to_human_readable(512), "512 B");
    assert_eq!(bytes_to_human_readable(1023), "1023 B");

    // Test KB
    assert_eq!(bytes_to_human_readable(1024), "1.0 KB");
    assert_eq!(bytes_to_human_readable(1536), "1.5 KB");
    assert_eq!(bytes_to_human_readable(2048), "2.0 KB");

    // Test MB
    assert_eq!(bytes_to_human_readable(1024 * 1024), "1.0 MB");
    assert_eq!(bytes_to_human_readable(1024 * 1024 + 512 * 1024), "1.5 MB");

    // Test GB
    assert_eq!(bytes_to_human_readable(1024 * 1024 * 1024), "1.0 GB");
    assert_eq!(bytes_to_human_readable(2_147_483_648), "2.0 GB");

    // Test TB
    assert_eq!(bytes_to_human_readable(1024_u64.pow(4)), "1.0 TB");
}
