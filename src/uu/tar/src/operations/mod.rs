// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

pub mod compression;
pub mod create;
pub mod extract;
pub mod list;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TarInput {
    File(PathBuf),
    ChangeDir(PathBuf),
}

pub struct CwdGuard {
    original_cwd: PathBuf,
}

impl CwdGuard {
    pub fn new() -> Result<Self, std::io::Error> {
        let original_cwd = std::env::current_dir()?;
        Ok(Self { original_cwd })
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        if let Err(e) = std::env::set_current_dir(&self.original_cwd) {
            eprintln!("tar: Failed to restore directory: {}", e);
        }
    }
}

#[cfg(test)]
pub(crate) fn test_cwd_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
pub(crate) struct TestDirGuard {
    old_dir: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

#[cfg(test)]
impl TestDirGuard {
    pub(crate) fn enter(path: &Path) -> Self {
        let guard = test_cwd_lock().lock().unwrap();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        Self {
            old_dir,
            _guard: guard,
        }
    }
}

#[cfg(test)]
impl Drop for TestDirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.old_dir);
    }
}
