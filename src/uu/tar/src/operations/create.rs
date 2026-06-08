// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::errors::TarError;
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Component::{self, ParentDir, Prefix, RootDir};
use std::path::{self, Path, PathBuf};
use tar::Builder;
use uucore::error::UResult;

/// Create a tar archive from the specified files
///
/// # Arguments
///
/// * `archive_path` - Path where the tar archive should be created
/// * `files` - Slice of file paths to add to the archive
/// * `allow_absolute` - Allow absolute paths while creating archive
/// * `verbose` - Whether to print verbose output during creation
///
/// # Errors
///
/// Returns an error if:
/// - The archive file cannot be created
/// - Any input file cannot be read
/// - Files cannot be added due to I/O or permission errors
pub fn create_archive(
    archive_path: &Path,
    files: &[&Path],
    allow_absolute: bool,
    verbose: bool,
) -> UResult<()> {
    // Create the output file
    let file = File::create(archive_path).map_err(|e| TarError::CannotCreateArchive {
        path: archive_path.to_path_buf(),
        source: e,
    })?;

    // Create Builder instance
    let mut builder = Builder::new(file);
    builder.preserve_absolute(allow_absolute);
    builder.follow_symlinks(false);

    let mut out = BufWriter::new(io::stdout().lock());

    // Add each file or directory to the archive
    for &path in files {
        // Check if path exists (including broken symlinks)
        let metadata = match path.symlink_metadata() {
            Ok(m) => m,
            Err(e) => return Err(TarError::from_io_error(e, path).into()),
        };

        if verbose {
            let to_print = get_tree(path)?
                .iter()
                .map(|(p, is_real_dir)| {
                    let path_str = p.display().to_string();
                    if *is_real_dir {
                        if !path_str.ends_with('/') && !path_str.ends_with(path::MAIN_SEPARATOR) {
                            format!("{}{}", path_str, path::MAIN_SEPARATOR)
                        } else {
                            path_str
                        }
                    } else {
                        path_str
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            writeln!(out, "{to_print}").map_err(TarError::Io)?;
        }

        // Normalize path if needed (so far, handles only absolute paths)
        let normalized_name = if let Some(normalized) = normalize_path(path, allow_absolute) {
            let original_components: Vec<Component> = path.components().collect();
            let normalized_components: Vec<Component> = normalized.components().collect();
            if original_components.len() > normalized_components.len() {
                let removed: PathBuf = original_components
                    [..original_components.len() - normalized_components.len()]
                    .iter()
                    .collect();
                writeln!(
                    out,
                    "Removing leading `{}' from member names",
                    removed.display()
                )
                .map_err(TarError::Io)?;
            }

            normalized
        } else {
            path.to_path_buf()
        };

        // If it's a directory, recursively add all contents
        // Do not recurse into symlinked directories
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            builder.append_dir_all(normalized_name, path).map_err(|e| {
                TarError::CannotAddDirectory {
                    path: path.to_path_buf(),
                    source: e,
                }
            })?;
        } else {
            // For files and symlinks, add them directly
            builder
                .append_path_with_name(path, normalized_name)
                .map_err(|e| TarError::CannotAddFile {
                    path: path.to_path_buf(),
                    source: e,
                })?;
        }
    }

    // Finish writing the archive
    out.flush().map_err(TarError::Io)?;
    builder.finish().map_err(TarError::CannotFinalizeArchive)?;

    Ok(())
}

fn get_tree(path: &Path) -> Result<Vec<(PathBuf, bool)>, std::io::Error> {
    let mut paths = Vec::new();
    let mut stack = VecDeque::new();
    stack.push_back(path.to_path_buf());

    while let Some(current) = stack.pop_back() {
        let metadata = current.symlink_metadata()?;
        let is_real_dir = metadata.is_dir() && !metadata.file_type().is_symlink();
        paths.push((current.clone(), is_real_dir));
        if is_real_dir {
            for entry in fs::read_dir(&current)? {
                let child = entry?.path();
                stack.push_back(child);
            }
        }
    }

    Ok(paths)
}

fn normalize_path(path: &Path, allow_absolute: bool) -> Option<PathBuf> {
    if path.is_absolute() && !allow_absolute {
        Some(
            path.components()
                .filter(|c| !matches!(c, RootDir | ParentDir | Prefix(_)))
                .collect::<PathBuf>(),
        )
    } else {
        None
    }
}
