//! Self-update via the GitHub Releases API.
//!
//! Checks for newer `demodatagen` releases and, on request, downloads and
//! atomically replaces the running binary. Networking uses rustls (no OpenSSL)
//! so it works on static musl builds too.
use crate::error::{AppError, AppResult};
use log::{info, warn};

/// GitHub repository owner for update checks.
const REPO_OWNER: &str = "j-pfalzgraf";
/// GitHub repository name for update checks.
const REPO_NAME: &str = "demodatagen";
/// Binary name to locate inside release archives.
const BIN_NAME: &str = "demodatagen";
/// Current version of the application.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the URL of the repository's latest-release page.
fn releases_url() -> String {
    format!("https://github.com/{REPO_OWNER}/{REPO_NAME}/releases")
}

/// Builds a configured self-update builder shared by check/perform.
fn updater_builder() -> self_update::backends::github::UpdateBuilder {
    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(CURRENT_VERSION);
    builder
}

/// Fetches the latest released version tag (without a leading `v`), if any.
///
/// Returns `Ok(None)` when the lookup fails (no releases yet, offline, …) so
/// callers can degrade gracefully instead of erroring.
pub fn latest_version() -> AppResult<Option<String>> {
    let updater = updater_builder()
        .build()
        .map_err(|e| AppError::Update(e.to_string()))?;
    match updater.get_latest_release() {
        Ok(release) => Ok(Some(release.version.trim_start_matches('v').to_string())),
        Err(e) => {
            warn!("Could not query latest release: {e}");
            Ok(None)
        }
    }
}

/// Checks for available updates via GitHub Releases.
///
/// Returns `Ok(true)` if a newer version is available, `Ok(false)` if the
/// current version is up to date (or the check could not be completed).
pub fn check_for_update() -> AppResult<bool> {
    info!("Checking for updates (current: v{CURRENT_VERSION})…");

    let Some(latest) = latest_version()? else {
        println!(
            "Could not determine the latest version. Check {} manually.",
            releases_url()
        );
        return Ok(false);
    };

    if version_is_newer(&latest, CURRENT_VERSION) {
        println!(
            "Update available: v{CURRENT_VERSION} -> v{latest}\n  \
             Run `demodatagen update` to upgrade, or download from {}/latest",
            releases_url()
        );
        Ok(true)
    } else {
        println!("demodatagen is up to date (v{CURRENT_VERSION}).");
        Ok(false)
    }
}

/// Performs a self-update to the latest release (non-interactive).
///
/// Equivalent to `update_to(None)`.
pub fn perform_update() -> AppResult<()> {
    update_to(None)
}

/// Performs a self-update, optionally pinning to a specific tag (e.g.
/// `"v0.2.0"`). Runs non-interactively and shows download progress.
///
/// # Errors
/// Returns [`AppError::Update`] if the update process fails.
pub fn update_to(target_tag: Option<&str>) -> AppResult<()> {
    info!("Starting self-update (current: v{CURRENT_VERSION})…");

    let mut builder = updater_builder();
    builder.show_download_progress(true).no_confirm(true);
    if let Some(tag) = target_tag {
        builder.target_version_tag(tag);
    }

    let updater = builder
        .build()
        .map_err(|e| AppError::Update(format!("could not configure updater: {e}")))?;

    match updater.update() {
        Ok(status) if status.updated() => {
            println!(
                "Updated demodatagen v{CURRENT_VERSION} -> v{}",
                status.version()
            );
            println!("Restart the command to use the new version.");
            Ok(())
        }
        Ok(_) => {
            println!("Already running the latest version (v{CURRENT_VERSION}).");
            Ok(())
        }
        Err(e) => {
            warn!("Update failed: {e}");
            Err(AppError::Update(format!(
                "{e}. You can download a release manually from {}/latest",
                releases_url()
            )))
        }
    }
}

/// Returns `true` if `candidate` is a strictly newer semver than `current`.
///
/// Falls back to a string inequality when either side is not parseable, so a
/// non-release dev build still sees published releases as "newer".
fn version_is_newer(candidate: &str, current: &str) -> bool {
    match (parse_semver(candidate), parse_semver(current)) {
        (Some(c), Some(cur)) => c > cur,
        _ => candidate != current,
    }
}

/// Parses a `major.minor.patch` string into a comparable tuple, ignoring any
/// pre-release/build suffix.
fn parse_semver(v: &str) -> Option<(u64, u64, u64)> {
    let core = v.trim_start_matches('v');
    let core = core.split(['-', '+']).next().unwrap_or(core);
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_constants() {
        assert_eq!(REPO_OWNER, "j-pfalzgraf");
        assert_eq!(REPO_NAME, "demodatagen");
        assert!(!CURRENT_VERSION.is_empty());
        assert!(releases_url().ends_with("/releases"));
    }

    #[test]
    fn test_current_version_semver() {
        assert!(parse_semver(CURRENT_VERSION).is_some());
    }

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("v0.2.0"), Some((0, 2, 0)));
        assert_eq!(parse_semver("1.2"), Some((1, 2, 0)));
        assert_eq!(parse_semver("2"), Some((2, 0, 0)));
        assert_eq!(parse_semver("1.2.3-rc1"), Some((1, 2, 3)));
        assert_eq!(parse_semver("1.2.3+build5"), Some((1, 2, 3)));
        assert_eq!(parse_semver("nightly"), None);
    }

    #[test]
    fn test_version_is_newer() {
        assert!(version_is_newer("0.2.0", "0.1.0"));
        assert!(version_is_newer("1.0.0", "0.9.9"));
        assert!(version_is_newer("0.2.1", "0.2.0"));
        assert!(!version_is_newer("0.1.0", "0.2.0"));
        assert!(!version_is_newer("0.2.0", "0.2.0"));
        // Unparseable current (dev build) treats any different release as newer.
        assert!(version_is_newer("0.2.0", "dev"));
    }
}
