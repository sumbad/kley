pub mod commands;
pub mod lockfile;
pub mod package;
pub mod registry;
pub mod utils;

/// Emoji prefixes for CLI output.
/// On Windows, the default terminal often cannot render emoji,
/// so we fall back to ASCII alternatives.
#[cfg(not(windows))]
pub mod emoji {
    pub const SUCCESS: &str = "✅";
    pub const ERROR: &str = "❌";
    pub const WARNING: &str = "⚠️";
    pub const PUBLISH: &str = "🚀";
    pub const PACKAGE: &str = "📦";
    pub const UPDATED: &str = "✔️";
    pub const UNPUBLISH: &str = "🧹";
    pub const WAITING: &str = "⏳";
}

#[cfg(windows)]
pub mod emoji {
    pub const SUCCESS: &str = "[OK]";
    pub const ERROR: &str = "[ERR]";
    pub const WARNING: &str = "[!]";
    pub const PUBLISH: &str = "[>>>]";
    pub const PACKAGE: &str = "[PKG]";
    pub const UPDATED: &str = "[+]";
    pub const UNPUBLISH: &str = "[-]";
    pub const WAITING: &str = "[...]";
}
