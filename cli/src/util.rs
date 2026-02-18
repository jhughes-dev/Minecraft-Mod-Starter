use crate::error::{McmodError, Result};
use std::path::Path;

/// Validates a mod ID: must match ^[a-z][a-z0-9_]*$
pub fn validate_mod_id(id: &str) -> Result<()> {
    let re = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_';
    if id.is_empty()
        || !id.starts_with(|c: char| c.is_ascii_lowercase())
        || !id.chars().all(re)
    {
        return Err(McmodError::InvalidModId(id.to_string()));
    }
    Ok(())
}

/// Validates a Java package name: ^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*$
pub fn validate_package(pkg: &str) -> Result<()> {
    let valid_segment = |s: &str| -> bool {
        !s.is_empty()
            && s.starts_with(|c: char| c.is_ascii_lowercase())
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    };
    if pkg.is_empty() || !pkg.split('.').all(valid_segment) {
        return Err(McmodError::InvalidPackage(pkg.to_string()));
    }
    Ok(())
}

/// Converts a snake_case string to PascalCase.
/// e.g. "my_cool_mod" -> "MyCoolMod"
pub fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => {
                    let mut result = c.to_uppercase().to_string();
                    result.extend(chars);
                    result
                }
                None => String::new(),
            }
        })
        .collect()
}

/// Converts a package name to a directory path.
/// e.g. "com.example.mymod" -> "com/example/mymod"
pub fn package_to_path(pkg: &str) -> String {
    pkg.replace('.', "/")
}

/// Derives the class name from a mod ID.
/// e.g. "my_mod" -> "MyModMod", "testmod" -> "TestmodMod"
pub fn derive_class_name(mod_id: &str) -> String {
    format!("{}Mod", to_pascal_case(mod_id))
}

/// Extracts the NeoForge major version from a full version string.
/// e.g. "21.4.156" -> "21.4"
pub fn neoforge_major(version: &str) -> String {
    let parts: Vec<&str> = version.splitn(3, '.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[0], parts[1])
    } else {
        version.to_string()
    }
}

/// Ensures a directory exists, creating it if necessary.
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Writes content to a file, creating parent directories as needed.
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

/// Writes binary content to a file, creating parent directories as needed.
pub fn write_binary(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_mod_id() {
        assert!(validate_mod_id("mymod").is_ok());
        assert!(validate_mod_id("my_mod").is_ok());
        assert!(validate_mod_id("mod123").is_ok());
        assert!(validate_mod_id("a").is_ok());

        assert!(validate_mod_id("").is_err());
        assert!(validate_mod_id("MyMod").is_err());
        assert!(validate_mod_id("1mod").is_err());
        assert!(validate_mod_id("my-mod").is_err());
        assert!(validate_mod_id("_mod").is_err());
    }

    #[test]
    fn test_validate_package() {
        assert!(validate_package("com.example.mymod").is_ok());
        assert!(validate_package("com.example").is_ok());
        assert!(validate_package("mymod").is_ok());

        assert!(validate_package("").is_err());
        assert!(validate_package("Com.example").is_err());
        assert!(validate_package("com..example").is_err());
        assert!(validate_package(".com").is_err());
        assert!(validate_package("com.").is_err());
        assert!(validate_package("com.1example").is_err());
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("my_cool_mod"), "MyCoolMod");
        assert_eq!(to_pascal_case("testmod"), "Testmod");
        assert_eq!(to_pascal_case("a_b_c"), "ABC");
        assert_eq!(to_pascal_case("hello"), "Hello");
    }

    #[test]
    fn test_package_to_path() {
        assert_eq!(package_to_path("com.example.mymod"), "com/example/mymod");
        assert_eq!(package_to_path("mymod"), "mymod");
    }

    #[test]
    fn test_derive_class_name() {
        assert_eq!(derive_class_name("my_mod"), "MyModMod");
        assert_eq!(derive_class_name("testmod"), "TestmodMod");
        assert_eq!(derive_class_name("cool_stuff"), "CoolStuffMod");
    }

    #[test]
    fn test_neoforge_major() {
        assert_eq!(neoforge_major("21.4.156"), "21.4");
        assert_eq!(neoforge_major("21.4"), "21.4");
        assert_eq!(neoforge_major("21"), "21");
    }
}
