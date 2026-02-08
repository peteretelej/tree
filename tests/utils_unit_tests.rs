use rstest::rstest;
use rust_tree::rust_tree::utils::{bytes_to_human_readable, is_broken_pipe_error};
use std::error::Error as StdError;
use std::fmt;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
struct WrappedError {
    inner: Error,
}

impl fmt::Display for WrappedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wrapped: {}", self.inner)
    }
}

impl StdError for WrappedError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.inner)
    }
}

#[test]
fn test_is_broken_pipe_error_direct() {
    let broken_pipe_err = Error::new(ErrorKind::BrokenPipe, "pipe broken");
    assert!(is_broken_pipe_error(&broken_pipe_err));

    let other_err = Error::new(ErrorKind::NotFound, "file not found");
    assert!(!is_broken_pipe_error(&other_err));

    let permission_err = Error::new(ErrorKind::PermissionDenied, "access denied");
    assert!(!is_broken_pipe_error(&permission_err));
}

#[rstest]
#[case::nested_broken_pipe(ErrorKind::BrokenPipe, true)]
#[case::nested_other_error(ErrorKind::ConnectionReset, false)]
fn test_is_broken_pipe_error_chain(#[case] inner_kind: ErrorKind, #[case] expected: bool) {
    let inner = Error::new(inner_kind, "inner error");
    let wrapped = WrappedError { inner };
    let outer = Error::new(ErrorKind::Other, wrapped);
    assert_eq!(is_broken_pipe_error(&outer), expected);
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
