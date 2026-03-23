/// Self-update module using GitHub Releases API.
///
/// Checks for new versions of `demodatagen` on GitHub and optionally
/// downloads and installs updates.
use crate::error::AppResult;
use log::{info, warn};

/// GitHub repository owner for update checks.
const REPO_OWNER: &str = "youruser";
/// GitHub repository name for update checks.
const REPO_NAME: &str = "demodatagen";
/// Current version of the application.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Checks for available updates via GitHub Releases.
///
/// Returns `Ok(true)` if an update is available, `Ok(false)` if the
/// current version is up to date.
///
/// # Errors
/// Returns an error if the update check fails (network issues, etc.).
pub fn check_for_update() -> AppResult<bool> {
    info!("Checking for updates (current version: v{CURRENT_VERSION})...");

    match self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name("demodatagen")
        .current_version(CURRENT_VERSION)
        .build()
    {
        Ok(updater) => match updater.get_latest_release() {
            Ok(release) => {
                let latest = release.version.trim_start_matches('v');
                if latest != CURRENT_VERSION {
                    info!("New version available: v{latest} (current: v{CURRENT_VERSION})");
                    println!(
                        "Update available: v{CURRENT_VERSION} -> v{latest}. \
                         Download from https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/latest"
                    );
                    Ok(true)
                } else {
                    info!("Already running the latest version (v{CURRENT_VERSION})");
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("Could not check for updates: {e}");
                Ok(false)
            }
        },
        Err(e) => {
            warn!("Could not configure updater: {e}");
            Ok(false)
        }
    }
}

/// Performs a full self-update, downloading and replacing the current binary.
///
/// # Errors
/// Returns an error if the update process fails.
#[allow(dead_code)]
pub fn perform_update() -> AppResult<()> {
    info!("Attempting self-update...");

    match self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name("demodatagen")
        .current_version(CURRENT_VERSION)
        .show_download_progress(true)
        .build()
    {
        Ok(updater) => match updater.update() {
            Ok(status) => {
                if status.updated() {
                    println!("Successfully updated to v{}", status.version());
                } else {
                    println!("Already running the latest version (v{CURRENT_VERSION})");
                }
                Ok(())
            }
            Err(e) => {
                warn!("Update failed: {e}");
                Err(crate::error::AppError::Update(e.to_string()))
            }
        },
        Err(e) => {
            warn!("Could not configure updater: {e}");
            Err(crate::error::AppError::Update(e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_constants() {
        assert!(!REPO_OWNER.is_empty());
        assert!(!REPO_NAME.is_empty());
        assert!(!CURRENT_VERSION.is_empty());
    }

    #[test]
    fn test_current_version_semver() {
        // Verify the version looks like semver
        let parts: Vec<&str> = CURRENT_VERSION.split('.').collect();
        assert!(parts.len() >= 2, "Version should be semver-like");
        assert!(parts[0].parse::<u32>().is_ok());
        assert!(parts[1].parse::<u32>().is_ok());
    }
}
