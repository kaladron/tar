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

#[test]
fn test_roundtrip_single_file() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create a file
    at.write("file.txt", "test content");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "file.txt"])
        .succeeds();
    
    // Remove original
    at.remove("file.txt");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify content is identical
    assert!(at.file_exists("file.txt"));
    assert_eq!(at.read("file.txt"), "test content");
}

#[test]
fn test_roundtrip_multiple_files() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create multiple files with different content
    at.write("file1.txt", "content one");
    at.write("file2.txt", "content two");
    at.write("file3.txt", "content three");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt", "file3.txt"])
        .succeeds();
    
    // Remove originals
    at.remove("file1.txt");
    at.remove("file2.txt");
    at.remove("file3.txt");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify all contents are identical
    assert_eq!(at.read("file1.txt"), "content one");
    assert_eq!(at.read("file2.txt"), "content two");
    assert_eq!(at.read("file3.txt"), "content three");
}

#[test]
fn test_roundtrip_directory_structure() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create complex directory structure
    at.mkdir("dir1");
    at.write("dir1/file1.txt", "content1");
    at.write("dir1/file2.txt", "content2");
    at.mkdir("dir1/subdir");
    at.write("dir1/subdir/file3.txt", "content3");
    at.mkdir("dir1/subdir/deepdir");
    at.write("dir1/subdir/deepdir/file4.txt", "content4");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "dir1"])
        .succeeds();
    
    // Remove directory structure
    at.remove("dir1/subdir/deepdir/file4.txt");
    std::fs::remove_dir(at.plus("dir1/subdir/deepdir")).unwrap();
    at.remove("dir1/subdir/file3.txt");
    std::fs::remove_dir(at.plus("dir1/subdir")).unwrap();
    at.remove("dir1/file1.txt");
    at.remove("dir1/file2.txt");
    std::fs::remove_dir(at.plus("dir1")).unwrap();
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify complete structure and contents
    assert!(at.dir_exists("dir1"));
    assert!(at.file_exists("dir1/file1.txt"));
    assert!(at.file_exists("dir1/file2.txt"));
    assert!(at.dir_exists("dir1/subdir"));
    assert!(at.file_exists("dir1/subdir/file3.txt"));
    assert!(at.dir_exists("dir1/subdir/deepdir"));
    assert!(at.file_exists("dir1/subdir/deepdir/file4.txt"));
    
    assert_eq!(at.read("dir1/file1.txt"), "content1");
    assert_eq!(at.read("dir1/file2.txt"), "content2");
    assert_eq!(at.read("dir1/subdir/file3.txt"), "content3");
    assert_eq!(at.read("dir1/subdir/deepdir/file4.txt"), "content4");
}

#[test]
fn test_roundtrip_empty_files() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create empty files
    at.write("empty1.txt", "");
    at.write("empty2.txt", "");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "empty1.txt", "empty2.txt"])
        .succeeds();
    
    // Remove originals
    at.remove("empty1.txt");
    at.remove("empty2.txt");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify empty files exist and are still empty
    assert!(at.file_exists("empty1.txt"));
    assert!(at.file_exists("empty2.txt"));
    assert_eq!(at.read("empty1.txt"), "");
    assert_eq!(at.read("empty2.txt"), "");
}

#[test]
fn test_roundtrip_special_characters_in_names() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create files with special characters (avoiding problematic ones)
    at.write("file-with-dash.txt", "dash content");
    at.write("file_with_underscore.txt", "underscore content");
    at.write("file.multiple.dots.txt", "dots content");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "file-with-dash.txt", 
                "file_with_underscore.txt", "file.multiple.dots.txt"])
        .succeeds();
    
    // Remove originals
    at.remove("file-with-dash.txt");
    at.remove("file_with_underscore.txt");
    at.remove("file.multiple.dots.txt");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify contents
    assert_eq!(at.read("file-with-dash.txt"), "dash content");
    assert_eq!(at.read("file_with_underscore.txt"), "underscore content");
    assert_eq!(at.read("file.multiple.dots.txt"), "dots content");
}

// -----------------------------------------------------------------------------
// 5. Error Handling Tests
// -----------------------------------------------------------------------------

// TODO: Implement error handling tests

// -----------------------------------------------------------------------------
// 6. Platform-specific Tests
// -----------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn test_preserve_permissions() {
    use std::os::unix::fs::PermissionsExt;
    
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create a file with specific permissions
    at.write("file.txt", "content");
    let file_path = at.plus("file.txt");
    std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    
    // Verify initial permissions
    let metadata = std::fs::metadata(&file_path).unwrap();
    let initial_mode = metadata.permissions().mode();
    assert_eq!(initial_mode & 0o777, 0o755);
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "file.txt"])
        .succeeds();
    
    // Remove original
    at.remove("file.txt");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify permissions are preserved
    let metadata = std::fs::metadata(at.plus("file.txt")).unwrap();
    let extracted_mode = metadata.permissions().mode();
    assert_eq!(extracted_mode & 0o777, 0o755, 
               "Permissions not preserved: expected 0o755, got 0o{:o}", 
               extracted_mode & 0o777);
}

#[cfg(unix)]
#[test]
fn test_preserve_permissions_multiple_files() {
    use std::os::unix::fs::PermissionsExt;
    
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create files with different permissions
    at.write("file1.txt", "content1");
    at.write("file2.txt", "content2");
    at.write("file3.txt", "content3");
    
    std::fs::set_permissions(at.plus("file1.txt"), std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::set_permissions(at.plus("file2.txt"), std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::set_permissions(at.plus("file3.txt"), std::fs::Permissions::from_mode(0o600)).unwrap();
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "file1.txt", "file2.txt", "file3.txt"])
        .succeeds();
    
    // Remove originals
    at.remove("file1.txt");
    at.remove("file2.txt");
    at.remove("file3.txt");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify each file's permissions
    let mode1 = std::fs::metadata(at.plus("file1.txt")).unwrap().permissions().mode();
    let mode2 = std::fs::metadata(at.plus("file2.txt")).unwrap().permissions().mode();
    let mode3 = std::fs::metadata(at.plus("file3.txt")).unwrap().permissions().mode();
    
    assert_eq!(mode1 & 0o777, 0o644, "file1.txt permissions not preserved");
    assert_eq!(mode2 & 0o777, 0o755, "file2.txt permissions not preserved");
    assert_eq!(mode3 & 0o777, 0o600, "file3.txt permissions not preserved");
}

#[cfg(unix)]
#[test]
fn test_preserve_directory_permissions() {
    use std::os::unix::fs::PermissionsExt;
    
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create directory with specific permissions
    at.mkdir("testdir");
    std::fs::set_permissions(at.plus("testdir"), std::fs::Permissions::from_mode(0o750)).unwrap();
    at.write("testdir/file.txt", "content");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "testdir"])
        .succeeds();
    
    // Remove directory
    at.remove("testdir/file.txt");
    std::fs::remove_dir(at.plus("testdir")).unwrap();
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify directory permissions are preserved
    let metadata = std::fs::metadata(at.plus("testdir")).unwrap();
    let mode = metadata.permissions().mode();
    assert_eq!(mode & 0o777, 0o750, 
               "Directory permissions not preserved: expected 0o750, got 0o{:o}", 
               mode & 0o777);
}

#[cfg(unix)]
#[test]
fn test_preserve_executable_bit() {
    use std::os::unix::fs::PermissionsExt;
    
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create an executable file
    at.write("script.sh", "#!/bin/bash\necho 'Hello'");
    let script_path = at.plus("script.sh");
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    
    // Verify it's executable
    let metadata = std::fs::metadata(&script_path).unwrap();
    assert!(metadata.permissions().mode() & 0o111 != 0, "File should be executable");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "script.sh"])
        .succeeds();
    
    // Remove original
    at.remove("script.sh");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify executable bit is preserved
    let metadata = std::fs::metadata(at.plus("script.sh")).unwrap();
    let mode = metadata.permissions().mode();
    assert!(mode & 0o111 != 0, 
            "Executable bit not preserved: mode is 0o{:o}", 
            mode & 0o777);
    assert_eq!(mode & 0o777, 0o755);
}
