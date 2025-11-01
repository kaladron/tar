// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::fmt;
use std::io;
use uucore::error::UError;

/// Error types for tar operations
#[derive(Debug)]
pub enum TarError {
    /// I/O error occurred
    IoError(io::Error),
    /// Invalid archive format or corrupted archive
    InvalidArchive(String),
    /// File or directory not found
    FileNotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// General tar operation error
    TarOperationError(String),
}

impl fmt::Display for TarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TarError::IoError(err) => write!(f, "I/O error: {}", err),
            TarError::InvalidArchive(msg) => write!(f, "Invalid archive: {}", msg),
            TarError::FileNotFound(path) => write!(f, "File not found: {}", path),
            TarError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            TarError::TarOperationError(msg) => write!(f, "tar: {}", msg),
        }
    }
}

impl std::error::Error for TarError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TarError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl UError for TarError {
    fn code(&self) -> i32 {
        match self {
            TarError::IoError(_) => 1,
            TarError::InvalidArchive(_) => 2,
            TarError::FileNotFound(_) => 1,
            TarError::PermissionDenied(_) => 1,
            TarError::TarOperationError(_) => 1,
        }
    }
}

impl From<io::Error> for TarError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => TarError::FileNotFound(err.to_string()),
            io::ErrorKind::PermissionDenied => TarError::PermissionDenied(err.to_string()),
            _ => TarError::IoError(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tar_error_display() {
        let err = TarError::FileNotFound("test.txt".to_string());
        assert_eq!(err.to_string(), "File not found: test.txt");

        let err = TarError::InvalidArchive("corrupted header".to_string());
        assert_eq!(err.to_string(), "Invalid archive: corrupted header");

        let err = TarError::PermissionDenied("/root/file".to_string());
        assert_eq!(err.to_string(), "Permission denied: /root/file");

        let err = TarError::TarOperationError("failed to write".to_string());
        assert_eq!(err.to_string(), "tar: failed to write");
    }

    #[test]
    fn test_tar_error_code() {
        assert_eq!(TarError::FileNotFound("test".to_string()).code(), 1);
        assert_eq!(TarError::InvalidArchive("test".to_string()).code(), 2);
        assert_eq!(TarError::PermissionDenied("test".to_string()).code(), 1);
        assert_eq!(TarError::TarOperationError("test".to_string()).code(), 1);
    }

    #[test]
    fn test_io_error_conversion_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let tar_err = TarError::from(io_err);
        
        match tar_err {
            TarError::FileNotFound(msg) => assert!(msg.contains("file not found")),
            _ => panic!("Expected FileNotFound variant"),
        }
    }

    #[test]
    fn test_io_error_conversion_permission_denied() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let tar_err = TarError::from(io_err);
        
        match tar_err {
            TarError::PermissionDenied(msg) => assert!(msg.contains("access denied")),
            _ => panic!("Expected PermissionDenied variant"),
        }
    }

    #[test]
    fn test_io_error_conversion_other() {
        let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
        let tar_err = TarError::from(io_err);
        
        match tar_err {
            TarError::IoError(e) => assert_eq!(e.kind(), io::ErrorKind::BrokenPipe),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_error_source() {
        let io_err = io::Error::other("some error");
        let tar_err = TarError::IoError(io_err);
        
        // IoError should have a source
        assert!(std::error::Error::source(&tar_err).is_some());
        
        // Other variants should not have a source
        let tar_err = TarError::FileNotFound("test".to_string());
        assert!(std::error::Error::source(&tar_err).is_none());
    }

    #[test]
    fn test_tar_error_is_debug() {
        let err = TarError::TarOperationError("test".to_string());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("TarOperationError"));
        assert!(debug_str.contains("test"));
    }
}
