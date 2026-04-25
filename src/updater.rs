use std::time::Duration;
use is_terminal::IsTerminal;
use update_informer::{registry, Check};

use crate::config::ToriiConfig;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Print a banner if a newer version is available on crates.io.
/// Silent on errors, non-tty stderr, or when the user opted out.
pub fn maybe_notify() {
    if !std::io::stderr().is_terminal() {
        return;
    }
    let cfg = ToriiConfig::load_global().unwrap_or_default();
    if !cfg.update.check {
        return;
    }

    let interval = Duration::from_secs(cfg.update.interval_hours.saturating_mul(3600));
    let informer = update_informer::new(registry::Crates, PKG_NAME, PKG_VERSION)
        .interval(interval)
        .timeout(Duration::from_secs(2));

    if let Ok(Some(new_version)) = informer.check_version() {
        let cmd = update_command();
        eprintln!();
        eprintln!("💡 New version of torii available: {} → {}", PKG_VERSION, new_version);
        eprintln!("   Update: {}", cmd);
        eprintln!("   Disable: torii config set update.check false");
    }
}

/// Best-effort guess at the install method based on the binary's path.
fn update_command() -> &'static str {
    let exe = std::env::current_exe().ok();
    let path = exe.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

    if path.contains("/.cargo/bin/") {
        "cargo install gitorii"
    } else if path.starts_with("/usr/local/bin")
        || path.starts_with("/usr/bin")
        || path.starts_with("/opt/")
    {
        "curl -fsSL https://gitorii.com/install.sh | sh"
    } else {
        "cargo install gitorii  (or re-run your installer)"
    }
}
