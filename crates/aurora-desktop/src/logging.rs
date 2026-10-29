//! The log file in the user's data directory (plan: observability). The level comes from
//! `AURORA_LOG` (default `info`). Engine output bodies are never logged.

use std::path::{Path, PathBuf};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;

pub const LOG_FILE_PREFIX: &str = "aurora";
pub const KEPT_LOG_FILES: usize = 5;

pub const LINUX_WEBVIEW_HINT: &str = "On Linux, Aurora needs WebKitGTK 4.1: install \
    `libwebkit2gtk-4.1-0` (Debian, Ubuntu) or `webkit2gtk4.1` (Fedora) and try again.";

/// Where the log files go: `<data dir>/logs`, `None` without a home directory.
pub fn log_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("digital", "Levix", "Aurora")
        .map(|dirs| log_dir_in(dirs.data_local_dir()))
}

pub fn log_dir_in(data_dir: &Path) -> PathBuf {
    data_dir.join("logs")
}

/// Starts logging to a daily file, keeping the last five. Returns the guard that flushes
/// the file on exit; without a usable directory, logs go to standard error.
pub fn init() -> Option<WorkerGuard> {
    let filter = EnvFilter::try_from_env("AURORA_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let appender = log_dir().and_then(|dir| {
        std::fs::create_dir_all(&dir).ok()?;
        RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix(LOG_FILE_PREFIX)
            .filename_suffix("log")
            .max_log_files(KEPT_LOG_FILES)
            .build(dir)
            .ok()
    });
    match appender {
        Some(appender) => {
            let (writer, guard) = tracing_appender::non_blocking(appender);
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(writer)
                .with_ansi(false)
                .init();
            Some(guard)
        }
        None => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
                .init();
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logs_go_to_a_logs_folder_of_the_data_directory() {
        let data = Path::new("/home/levi/.local/share/aurora");
        assert_eq!(log_dir_in(data), data.join("logs"));
    }

    #[test]
    fn the_log_directory_is_aurora_s_own() {
        if let Some(dir) = log_dir() {
            let text = dir.to_string_lossy().to_lowercase();
            assert!(text.contains("aurora"), "{text}");
            assert!(dir.ends_with("logs"));
        }
    }
}
