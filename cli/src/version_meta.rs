/// Per-Minecraft-version metadata for all upstream dependency versions.
///
/// Each entry pins known-good versions of Fabric Loader, Fabric API, and NeoForge
/// for a specific Minecraft release. When the CLI targets a particular MC version
/// (via `--minecraft` or interactive prompt), this table provides the offline
/// defaults and guides online fetching.

/// Dependency versions for a specific Minecraft release.
#[derive(Debug, Clone)]
pub struct VersionMeta {
    pub minecraft: &'static str,
    pub fabric_loader: &'static str,
    pub fabric_api: &'static str,
    pub neoforge: &'static str,
}

/// All supported Minecraft versions and their known-good dependency versions.
///
/// Versions are ordered oldest → newest. When multiple MC versions share the
/// same dependency version (e.g. Fabric Loader), that's expected — the loader
/// is generally forward-compatible across minor MC releases.
///
/// NeoForge versions marked with `-beta` indicate that no stable release exists
/// for that MC version yet. The CLI will still use them as defaults.
pub const VERSION_TABLE: &[VersionMeta] = &[
    // --- 1.21.1 ---
    VersionMeta {
        minecraft: "1.21.1",
        fabric_loader: "0.18.5",
        fabric_api: "0.116.9+1.21.1",
        neoforge: "21.1.221",

    },
    // --- 1.21.2 ---
    // NeoForge only has beta releases for this MC version.
    VersionMeta {
        minecraft: "1.21.2",
        fabric_loader: "0.18.5",
        fabric_api: "0.106.1+1.21.2",
        neoforge: "21.2.1-beta",

    },
    // --- 1.21.3 ---
    VersionMeta {
        minecraft: "1.21.3",
        fabric_loader: "0.18.5",
        fabric_api: "0.114.1+1.21.3",
        neoforge: "21.3.96",

    },
    // --- 1.21.4 ---
    VersionMeta {
        minecraft: "1.21.4",
        fabric_loader: "0.18.5",
        fabric_api: "0.119.4+1.21.4",
        neoforge: "21.4.157",

    },
    // --- 1.21.5 ---
    VersionMeta {
        minecraft: "1.21.5",
        fabric_loader: "0.18.5",
        fabric_api: "0.128.2+1.21.5",
        neoforge: "21.5.97",

    },
    // --- 1.21.6 ---
    // NeoForge only has beta releases for this MC version.
    VersionMeta {
        minecraft: "1.21.6",
        fabric_loader: "0.18.5",
        fabric_api: "0.128.2+1.21.6",
        neoforge: "21.6.20-beta",

    },
    // --- 1.21.7 ---
    // NeoForge only has beta releases for this MC version.
    VersionMeta {
        minecraft: "1.21.7",
        fabric_loader: "0.18.5",
        fabric_api: "0.128.2+1.21.7",
        neoforge: "21.7.25-beta",

    },
    // --- 1.21.8 ---
    VersionMeta {
        minecraft: "1.21.8",
        fabric_loader: "0.18.5",
        fabric_api: "0.136.1+1.21.8",
        neoforge: "21.8.53",

    },
    // --- 1.21.9 ---
    // NeoForge only has beta releases for this MC version.
    VersionMeta {
        minecraft: "1.21.9",
        fabric_loader: "0.18.5",
        fabric_api: "0.134.1+1.21.9",
        neoforge: "21.9.16-beta",

    },
    // --- 1.21.10 ---
    VersionMeta {
        minecraft: "1.21.10",
        fabric_loader: "0.18.5",
        fabric_api: "0.138.4+1.21.10",
        neoforge: "21.10.64",

    },
    // --- 1.21.11 ---
    // NeoForge only has beta releases for this MC version.
    VersionMeta {
        minecraft: "1.21.11",
        fabric_loader: "0.18.5",
        fabric_api: "0.141.3+1.21.11",
        neoforge: "21.11.40-beta",

    },
];

/// Look up version metadata for a given Minecraft version.
pub fn get_version_meta(mc_version: &str) -> Option<&'static VersionMeta> {
    VERSION_TABLE.iter().find(|v| v.minecraft == mc_version)
}

/// Returns all supported Minecraft version strings, oldest first.
pub fn supported_versions() -> Vec<&'static str> {
    VERSION_TABLE.iter().map(|v| v.minecraft).collect()
}

/// Given a sorted list of target MC versions, compute `VersionTarget`s with ranges.
///
/// Each target covers from its own MC version up to (but not including) the next
/// target. The last target covers up to the latest supported version.
///
/// Example: `["1.21.1", "1.21.7"]` →
///   - target "1.21.1" covers 1.21.1–1.21.6
///   - target "1.21.7" covers 1.21.7–1.21.11
pub fn targets_to_ranges(targets: &[&str]) -> Vec<crate::config::VersionTarget> {
    let supported = supported_versions();
    let mut result = Vec::new();

    for (i, &target) in targets.iter().enumerate() {
        let meta = match get_version_meta(target) {
            Some(m) => m,
            None => continue,
        };

        // Upper bound: one version before the next target, or the last supported version
        let max_mc = if i + 1 < targets.len() {
            // Find the version just before the next target
            let next_target = targets[i + 1];
            let next_idx = supported.iter().position(|&v| v == next_target);
            match next_idx {
                Some(idx) if idx > 0 => supported[idx - 1],
                _ => target, // fallback to single-version range
            }
        } else {
            // Last target: extends to the latest supported version
            supported.last().copied().unwrap_or(target)
        };

        result.push(crate::config::VersionTarget {
            minecraft: target.to_string(),
            max_minecraft: max_mc.to_string(),
            fabric_loader: meta.fabric_loader.to_string(),
            fabric_api: meta.fabric_api.to_string(),
            neoforge: meta.neoforge.to_string(),
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_known_version() {
        let meta = get_version_meta("1.21.1").unwrap();
        assert_eq!(meta.minecraft, "1.21.1");
        assert_eq!(meta.neoforge, "21.1.221");
    }

    #[test]
    fn test_lookup_current_default() {
        let meta = get_version_meta("1.21.4").unwrap();
        assert_eq!(meta.fabric_api, "0.119.4+1.21.4");
    }

    #[test]
    fn test_lookup_latest_version() {
        let meta = get_version_meta("1.21.11").unwrap();
        assert_eq!(meta.fabric_api, "0.141.3+1.21.11");
        assert!(meta.neoforge.contains("beta"));
    }

    #[test]
    fn test_lookup_unknown_version() {
        assert!(get_version_meta("1.20.4").is_none());
        assert!(get_version_meta("1.21.99").is_none());
    }

    #[test]
    fn test_supported_versions_complete() {
        let versions = supported_versions();
        assert_eq!(versions.len(), 11);
        assert_eq!(versions[0], "1.21.1");
        assert_eq!(*versions.last().unwrap(), "1.21.11");
    }

    #[test]
    fn test_all_entries_have_neoforge_prefix_matching_mc() {
        for meta in VERSION_TABLE {
            let mc_parts: Vec<&str> = meta.minecraft.splitn(3, '.').collect();
            let expected_prefix = if mc_parts.len() == 3 {
                format!("{}.{}.", mc_parts[1], mc_parts[2])
            } else {
                format!("{}.", mc_parts[1])
            };
            // Strip -beta suffix for prefix check
            let nf_version = meta.neoforge.split('-').next().unwrap();
            assert!(
                nf_version.starts_with(&expected_prefix),
                "NeoForge version {} should start with {} for MC {}",
                meta.neoforge,
                expected_prefix,
                meta.minecraft
            );
        }
    }

    #[test]
    fn test_targets_to_ranges_two_targets() {
        let ranges = targets_to_ranges(&["1.21.1", "1.21.7"]);
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].minecraft, "1.21.1");
        assert_eq!(ranges[0].max_minecraft, "1.21.6");
        assert_eq!(ranges[1].minecraft, "1.21.7");
        assert_eq!(ranges[1].max_minecraft, "1.21.11");
    }

    #[test]
    fn test_targets_to_ranges_single_target() {
        let ranges = targets_to_ranges(&["1.21.4"]);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].minecraft, "1.21.4");
        assert_eq!(ranges[0].max_minecraft, "1.21.11"); // extends to latest
    }

    #[test]
    fn test_targets_to_ranges_three_targets() {
        let ranges = targets_to_ranges(&["1.21.1", "1.21.4", "1.21.8"]);
        assert_eq!(ranges.len(), 3);
        assert_eq!(ranges[0].max_minecraft, "1.21.3");
        assert_eq!(ranges[1].max_minecraft, "1.21.7");
        assert_eq!(ranges[2].max_minecraft, "1.21.11");
    }

    #[test]
    fn test_all_entries_have_fabric_api_suffix_matching_mc() {
        for meta in VERSION_TABLE {
            let expected_suffix = format!("+{}", meta.minecraft);
            assert!(
                meta.fabric_api.ends_with(&expected_suffix),
                "Fabric API version {} should end with {} for MC {}",
                meta.fabric_api,
                expected_suffix,
                meta.minecraft
            );
        }
    }
}
