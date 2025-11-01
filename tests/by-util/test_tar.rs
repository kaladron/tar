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

// -----------------------------------------------------------------------------
// 7. Edge Case Tests
// -----------------------------------------------------------------------------

#[test]
fn test_very_long_filename() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create a file with a very long name (100 characters)
    let long_name = "a".repeat(100) + ".txt";
    at.write(&long_name, "content with long filename");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", &long_name])
        .succeeds();
    
    // Remove original
    at.remove(&long_name);
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify file was extracted with correct content
    assert!(at.file_exists(&long_name));
    assert_eq!(at.read(&long_name), "content with long filename");
}

#[test]
fn test_deeply_nested_path() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create a deeply nested directory structure (10 levels)
    let mut path = String::new();
    for i in 0..10 {
        if !path.is_empty() {
            path.push('/');
        }
        path.push_str(&format!("dir{i}"));
        at.mkdir(&path);
    }
    
    let file_path = format!("{path}/deep_file.txt");
    at.write(&file_path, "deeply nested content");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "dir0"])
        .succeeds();
    
    // Remove the directory structure
    std::fs::remove_file(at.plus(&file_path)).unwrap();
    for i in (0..10).rev() {
        let mut dir_path = String::new();
        for j in 0..=i {
            if !dir_path.is_empty() {
                dir_path.push('/');
            }
            dir_path.push_str(&format!("dir{j}"));
        }
        std::fs::remove_dir(at.plus(&dir_path)).unwrap();
    }
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify deeply nested file exists with correct content
    assert!(at.file_exists(&file_path));
    assert_eq!(at.read(&file_path), "deeply nested content");
}

#[test]
fn test_file_with_spaces_in_name() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create files with spaces in names
    at.write("file with spaces.txt", "content 1");
    at.write("another file.txt", "content 2");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "file with spaces.txt", "another file.txt"])
        .succeeds();
    
    // Remove originals
    at.remove("file with spaces.txt");
    at.remove("another file.txt");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify files extracted correctly
    assert!(at.file_exists("file with spaces.txt"));
    assert!(at.file_exists("another file.txt"));
    assert_eq!(at.read("file with spaces.txt"), "content 1");
    assert_eq!(at.read("another file.txt"), "content 2");
}

#[test]
fn test_large_number_of_files() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create 100 files
    let num_files = 100;
    for i in 0..num_files {
        at.write(&format!("file{i}.txt"), &format!("content {i}"));
    }
    
    // Collect file names for archive creation
    let files: Vec<String> = (0..num_files).map(|i| format!("file{i}.txt")).collect();
    let mut args = vec!["-cf", "archive.tar"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);
    
    // Create archive
    ucmd.args(&args).succeeds();
    
    // Verify archive was created
    assert!(at.file_exists("archive.tar"));
    
    // Verify archive contains all files
    let archive_data = at.read_bytes("archive.tar");
    let mut ar = Archive::new(Cursor::new(archive_data));
    let entry_count = ar.entries().unwrap().count();
    assert_eq!(entry_count, num_files, "Archive should contain {num_files} files");
}

#[test]
fn test_unicode_filenames() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create files with unicode characters
    at.write("文件.txt", "Chinese characters");    // 文件 = "file"
    at.write("файл.txt", "Russian characters");    // файл = "file"
    at.write("αρχείο.txt", "Greek characters");    // αρχείο = "file"
    at.write("ファイル.txt", "Japanese characters"); // ファイル = "file"
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "文件.txt", "файл.txt", "αρχείο.txt", "ファイル.txt"])
        .succeeds();
    
    // Remove originals
    at.remove("文件.txt");
    at.remove("файл.txt");
    at.remove("αρχείο.txt");
    at.remove("ファイル.txt");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify files extracted correctly
    assert!(at.file_exists("文件.txt"));
    assert!(at.file_exists("файл.txt"));
    assert!(at.file_exists("αρχείο.txt"));
    assert!(at.file_exists("ファイル.txt"));
    assert_eq!(at.read("文件.txt"), "Chinese characters");
    assert_eq!(at.read("файл.txt"), "Russian characters");
    assert_eq!(at.read("αρχείο.txt"), "Greek characters");
    assert_eq!(at.read("ファイル.txt"), "Japanese characters");
}

#[test]
fn test_binary_file_content() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create a file with binary content (non-UTF8 bytes)
    let binary_content: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD, 0x89, 0x50, 0x4E, 0x47];
    std::fs::write(at.plus("binary.dat"), &binary_content).unwrap();
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "binary.dat"])
        .succeeds();
    
    // Remove original
    at.remove("binary.dat");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify binary content is preserved exactly
    let extracted_content = std::fs::read(at.plus("binary.dat")).unwrap();
    assert_eq!(extracted_content, binary_content);
}

#[test]
fn test_file_with_no_extension() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create files without extensions
    at.write("README", "readme content");
    at.write("Makefile", "makefile content");
    at.write("LICENSE", "license content");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "README", "Makefile", "LICENSE"])
        .succeeds();
    
    // Remove originals
    at.remove("README");
    at.remove("Makefile");
    at.remove("LICENSE");
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify files extracted correctly
    assert!(at.file_exists("README"));
    assert!(at.file_exists("Makefile"));
    assert!(at.file_exists("LICENSE"));
    assert_eq!(at.read("README"), "readme content");
    assert_eq!(at.read("Makefile"), "makefile content");
    assert_eq!(at.read("LICENSE"), "license content");
}

#[test]
fn test_hidden_files() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create hidden files (files starting with dot)
    at.write(".hidden", "hidden content");
    at.write(".gitignore", "*.tmp");
    at.mkdir(".config");
    at.write(".config/settings", "settings content");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", ".hidden", ".gitignore", ".config"])
        .succeeds();
    
    // Remove originals
    at.remove(".hidden");
    at.remove(".gitignore");
    at.remove(".config/settings");
    std::fs::remove_dir(at.plus(".config")).unwrap();
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify hidden files extracted correctly
    assert!(at.file_exists(".hidden"));
    assert!(at.file_exists(".gitignore"));
    assert!(at.dir_exists(".config"));
    assert!(at.file_exists(".config/settings"));
    assert_eq!(at.read(".hidden"), "hidden content");
    assert_eq!(at.read(".gitignore"), "*.tmp");
    assert_eq!(at.read(".config/settings"), "settings content");
}

#[test]
fn test_mixed_empty_and_non_empty_directories() {
    let (at, mut ucmd) = at_and_ucmd!();
    
    // Create directory structure with both empty and non-empty directories
    at.mkdir("parent");
    at.mkdir("parent/empty_dir");
    at.mkdir("parent/non_empty_dir");
    at.write("parent/non_empty_dir/file.txt", "content");
    at.mkdir("parent/nested");
    at.mkdir("parent/nested/also_empty");
    
    // Create archive
    ucmd.args(&["-cf", "archive.tar", "parent"])
        .succeeds();
    
    // Remove directory structure
    at.remove("parent/non_empty_dir/file.txt");
    std::fs::remove_dir(at.plus("parent/nested/also_empty")).unwrap();
    std::fs::remove_dir(at.plus("parent/nested")).unwrap();
    std::fs::remove_dir(at.plus("parent/non_empty_dir")).unwrap();
    std::fs::remove_dir(at.plus("parent/empty_dir")).unwrap();
    std::fs::remove_dir(at.plus("parent")).unwrap();
    
    // Extract
    new_ucmd!()
        .arg("-xf")
        .arg(at.plus("archive.tar"))
        .current_dir(at.as_string())
        .succeeds();
    
    // Verify all directories exist
    assert!(at.dir_exists("parent"));
    assert!(at.dir_exists("parent/empty_dir"));
    assert!(at.dir_exists("parent/non_empty_dir"));
    assert!(at.dir_exists("parent/nested"));
    assert!(at.dir_exists("parent/nested/also_empty"));
    assert!(at.file_exists("parent/non_empty_dir/file.txt"));
    assert_eq!(at.read("parent/non_empty_dir/file.txt"), "content");
}
