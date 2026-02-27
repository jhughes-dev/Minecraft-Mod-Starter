use crate::error::{McmodError, Result};
use std::path::{Path, PathBuf};

/// Returns the platform-specific standard install directory for the mcmod binary.
/// - Windows: %LOCALAPPDATA%\mcmod
/// - Unix: ~/.local/bin
pub fn install_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return Ok(PathBuf::from(local).join("mcmod"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local").join("bin"));
        }
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| McmodError::Other("Could not determine home directory".to_string()))?;

    #[cfg(target_os = "windows")]
    {
        Ok(PathBuf::from(&home).join("AppData").join("Local").join("mcmod"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(PathBuf::from(&home).join(".local").join("bin"))
    }
}

/// Returns the full path to the standard mcmod binary location.
pub fn install_path() -> Result<PathBuf> {
    let dir = install_dir()?;
    if cfg!(target_os = "windows") {
        Ok(dir.join("mcmod.exe"))
    } else {
        Ok(dir.join("mcmod"))
    }
}

/// Returns whether the given directory is present on the system PATH.
pub fn is_on_path(dir: &Path) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        let separator = if cfg!(target_os = "windows") { ';' } else { ':' };
        for entry in path_var.split(separator) {
            if Path::new(entry) == dir {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_dir_returns_ok() {
        // Should succeed on any platform with HOME or LOCALAPPDATA set
        let dir = install_dir();
        assert!(dir.is_ok());
        let dir = dir.unwrap();
        #[cfg(target_os = "windows")]
        assert!(dir.to_string_lossy().contains("mcmod"));
        #[cfg(not(target_os = "windows"))]
        assert!(dir.to_string_lossy().contains(".local"));
    }

    #[test]
    fn test_install_path_has_correct_filename() {
        let path = install_path().unwrap();
        let filename = path.file_name().unwrap().to_string_lossy();
        if cfg!(target_os = "windows") {
            assert_eq!(filename, "mcmod.exe");
        } else {
            assert_eq!(filename, "mcmod");
        }
    }

    #[test]
    fn test_is_on_path_with_known_dir() {
        // The system PATH should contain at least one directory
        if let Ok(path_var) = std::env::var("PATH") {
            let sep = if cfg!(target_os = "windows") { ';' } else { ':' };
            if let Some(first) = path_var.split(sep).next() {
                assert!(is_on_path(Path::new(first)));
            }
        }
    }

    #[test]
    fn test_is_on_path_with_nonexistent_dir() {
        assert!(!is_on_path(Path::new("/this/path/definitely/does/not/exist/anywhere")));
    }
}
