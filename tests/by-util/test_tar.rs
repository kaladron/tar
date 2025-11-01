// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#![allow(unused_imports)] // Will be used as tests are implemented

use std::io::Cursor;
use tar_rs_crate::Archive;
use uutests::{at_and_ucmd, new_ucmd};

// =============================================================================
// Test Categories:
// 1. Basic CLI tests (help, version, invalid args)
// 2. Create operation tests
// 3. Extract operation tests
// 4. Round-trip tests
// 5. Error handling tests
// 6. Platform-specific tests (Unix permissions)
// =============================================================================

// -----------------------------------------------------------------------------
// 1. Basic CLI Tests
// -----------------------------------------------------------------------------

#[test]
fn test_invalid_arg() {
    new_ucmd!()
        .arg("--definitely-invalid")
        .fails()
        .code_is(1)
        .stderr_contains("unexpected argument");
}

#[test]
fn test_help() {
    new_ucmd!()
        .arg("--help")
        .succeeds()
        .code_is(0)
        .stdout_contains("an archiving utility");
}

#[test]
fn test_version() {
    new_ucmd!()
        .arg("--version")
        .succeeds()
        .code_is(0)
        .stdout_contains("tar");
}

#[test]
fn test_missing_f_option_create() {
    new_ucmd!()
        .args(&["-c", "file.txt"])
        .fails()
        .code_is(1)
        .stderr_contains("requires an argument");
}

#[test]
fn test_missing_f_option_extract() {
    new_ucmd!()
        .arg("-x")
        .fails()
        .code_is(1)
        .stderr_contains("requires an argument");
}

#[test]
fn test_conflicting_operations() {
    new_ucmd!()
        .args(&["-c", "-x", "-f", "archive.tar"])
        .fails()
        .code_is(1);
}

#[test]
fn test_no_operation_specified() {
    new_ucmd!()
        .args(&["-f", "archive.tar"])
        .fails()
        .code_is(1)
        .stderr_contains("must specify one");
}

// -----------------------------------------------------------------------------
// 2. Create Operation Tests
// -----------------------------------------------------------------------------

#[test]
fn test_create_single_file() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    at.write("file1.txt", "test content");
    
    ucmd.args(&["-cf", "archive.tar", "file1.txt"])
        .succeeds()
        .no_stderr();
    
    assert!(at.file_exists("archive.tar"));
    
    // Verify archive contents using tar-rs
    let archive_data = at.read_bytes("archive.tar");
    let mut ar = Archive::new(Cursor::new(archive_data));
    let entries: Vec<_> = ar.entries().unwrap()
        .map(|e| e.unwrap().path().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(entries, vec!["file1.txt"]);
}

#[test]
fn test_create_multiple_files() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");
    at.write("file3.txt", "content3");
    
    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt", "file3.txt"])
        .succeeds()
        .no_stderr();
    
    assert!(at.file_exists("archive.tar"));
    
    // Verify archive contents
    let archive_data = at.read_bytes("archive.tar");
    let mut ar = Archive::new(Cursor::new(archive_data));
    let mut entries: Vec<_> = ar.entries().unwrap()
        .map(|e| e.unwrap().path().unwrap().to_string_lossy().to_string())
        .collect();
    entries.sort();
    assert_eq!(entries, vec!["file1.txt", "file2.txt", "file3.txt"]);
}

#[test]
fn test_create_directory() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    at.mkdir("dir1");
    at.write("dir1/file1.txt", "content1");
    at.write("dir1/file2.txt", "content2");
    at.mkdir("dir1/subdir");
    at.write("dir1/subdir/file3.txt", "content3");
    
    ucmd.args(&["-cf", "archive.tar", "dir1"])
        .succeeds()
        .no_stderr();
    
    assert!(at.file_exists("archive.tar"));
    
    // Verify archive contains directory and files
    let archive_data = at.read_bytes("archive.tar");
    let mut ar = Archive::new(Cursor::new(archive_data));
    let entries: Vec<_> = ar.entries().unwrap()
        .map(|e| e.unwrap().path().unwrap().to_string_lossy().to_string())
        .collect();
    
    // Should contain the directory and its contents recursively
    assert!(entries.iter().any(|e| e.contains("dir1")));
    assert!(entries.iter().any(|e| e.contains("file1.txt")));
    assert!(entries.iter().any(|e| e.contains("file2.txt")));
    assert!(entries.iter().any(|e| e.contains("subdir")));
    assert!(entries.iter().any(|e| e.contains("file3.txt")));
}

#[test]
fn test_create_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    at.write("file1.txt", "content");
    
    ucmd.args(&["-cvf", "archive.tar", "file1.txt"])
        .succeeds()
        .stdout_contains("file1.txt");
    
    assert!(at.file_exists("archive.tar"));
}

#[test]
fn test_create_empty_archive_fails() {
    new_ucmd!()
        .args(&["-cf", "archive.tar"])
        .fails()
        .code_is(1)
        .stderr_contains("empty archive");
}

#[test]
fn test_create_nonexistent_file_fails() {
    let (_at, mut ucmd) = at_and_ucmd!();
    
    ucmd.args(&["-cf", "archive.tar", "nonexistent.txt"])
        .fails()
        .code_is(1)
        .stderr_contains("nonexistent.txt");
}

// -----------------------------------------------------------------------------
// 3. Extract Operation Tests
// -----------------------------------------------------------------------------

#[test]
fn test_extract_basic() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create an archive first
    at.write("original.txt", "test content");
    ucmd.args(&["-cf", "archive.tar", "original.txt"])
        .succeeds();
    
    // Remove original and extract (extracts to current directory)
    at.remove("original.txt");
    
    new_ucmd!()
        .arg(&at.plus("archive.tar"))
        .arg("-xf")
        .arg(&at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds()
        .no_stderr();
    
    assert!(at.file_exists("original.txt"));
    assert_eq!(at.read("original.txt"), "test content");
}

#[test]
fn test_extract_verbose() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create an archive
    at.write("file1.txt", "content");
    ucmd.args(&["-cf", "archive.tar", "file1.txt"])
        .succeeds();
    
    at.remove("file1.txt");
    
    // Extract with verbose (extracts to current directory)
    new_ucmd!()
        .arg("-xvf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds()
        .stdout_contains("file1.txt");
    
    assert!(at.file_exists("file1.txt"));
}

#[test]
fn test_extract_nonexistent_archive() {
    new_ucmd!()
        .args(&["-xf", "nonexistent.tar"])
        .fails()
        .code_is(1)
        .stderr_contains("nonexistent.tar");
}

#[test]
fn test_extract_directory_structure() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create directory structure
    at.mkdir("testdir");
    at.write("testdir/file1.txt", "content1");
    at.mkdir("testdir/subdir");
    at.write("testdir/subdir/file2.txt", "content2");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "testdir"])
        .succeeds();
    
    // Remove directory contents and directory itself
    at.remove("testdir/subdir/file2.txt");
    at.remove("testdir/file1.txt");
    std::fs::remove_dir(at.plus("testdir/subdir")).unwrap();
    std::fs::remove_dir(at.plus("testdir")).unwrap();
    
    // Extract (extracts to current directory)
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify structure
    assert!(at.dir_exists("testdir"));
    assert!(at.file_exists("testdir/file1.txt"));
    assert!(at.dir_exists("testdir/subdir"));
    assert!(at.file_exists("testdir/subdir/file2.txt"));
    assert_eq!(at.read("testdir/file1.txt"), "content1");
    assert_eq!(at.read("testdir/subdir/file2.txt"), "content2");
}

// -----------------------------------------------------------------------------
// 4. Round-trip Tests
// -----------------------------------------------------------------------------

// TODO: Implement round-trip tests

// -----------------------------------------------------------------------------
// 5. Error Handling Tests
// -----------------------------------------------------------------------------

// TODO: Implement error handling tests

// -----------------------------------------------------------------------------
// 6. Platform-specific Tests
// -----------------------------------------------------------------------------

// TODO: Implement platform-specific tests (Unix permissions, etc.)
