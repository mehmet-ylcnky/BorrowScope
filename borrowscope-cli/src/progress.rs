//! Progress indicators for long-running operations

#![allow(dead_code)]

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

/// Progress bar for building projects
pub fn build_progress(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Progress bar for file operations
pub fn file_progress(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Progress bar for downloads or network operations
pub fn download_progress(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.green/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
            .progress_chars("█▓▒░ "),
    );
    pb
}

/// Multi-progress for concurrent operations
pub fn multi_progress() -> MultiProgress {
    MultiProgress::new()
}

/// Simple spinner for indeterminate operations
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Progress bar for tests
pub fn test_progress(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("Running tests [{bar:40.yellow/blue}] {pos}/{len}")
            .unwrap()
            .progress_chars("##-"),
    );
    pb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_progress_creation() {
        let pb = build_progress("Building project");
        assert!(!pb.is_finished());
        pb.finish_and_clear();
    }

    #[test]
    fn test_file_progress_creation() {
        let pb = file_progress(100, "Processing files");
        assert_eq!(pb.length(), Some(100));
        assert!(!pb.is_finished());
        pb.finish_and_clear();
    }

    #[test]
    fn test_download_progress_creation() {
        let pb = download_progress(1024);
        assert_eq!(pb.length(), Some(1024));
        pb.finish_and_clear();
    }

    #[test]
    fn test_spinner_creation() {
        let pb = spinner("Loading");
        assert!(!pb.is_finished());
        pb.finish_and_clear();
    }

    #[test]
    fn test_test_progress_creation() {
        let pb = test_progress(50);
        assert_eq!(pb.length(), Some(50));
        pb.finish_and_clear();
    }

    #[test]
    fn test_multi_progress_creation() {
        let mp = multi_progress();
        let pb1 = mp.add(ProgressBar::new(10));
        let pb2 = mp.add(ProgressBar::new(20));

        assert_eq!(pb1.length(), Some(10));
        assert_eq!(pb2.length(), Some(20));

        pb1.finish_and_clear();
        pb2.finish_and_clear();
    }

    #[test]
    fn test_progress_increment() {
        let pb = file_progress(10, "Test");
        assert_eq!(pb.position(), 0);

        pb.inc(1);
        assert_eq!(pb.position(), 1);

        pb.inc(5);
        assert_eq!(pb.position(), 6);

        pb.finish_and_clear();
    }

    #[test]
    fn test_progress_set_position() {
        let pb = file_progress(100, "Test");

        pb.set_position(50);
        assert_eq!(pb.position(), 50);

        pb.finish_and_clear();
    }

    #[test]
    fn test_progress_finish() {
        let pb = file_progress(10, "Test");
        pb.inc(5);

        pb.finish();
        assert!(pb.is_finished());
    }

    #[test]
    fn test_progress_finish_and_clear() {
        let pb = spinner("Test");
        pb.finish_and_clear();
        assert!(pb.is_finished());
    }

    #[test]
    fn test_progress_finish_with_message() {
        let pb = file_progress(10, "Test");
        pb.finish_with_message("Done!");
        assert!(pb.is_finished());
    }

    #[test]
    fn test_progress_message_update() {
        let pb = spinner("Initial");
        pb.set_message("Updated");
        pb.finish_and_clear();
    }

    #[test]
    fn test_build_progress_with_long_message() {
        let long_msg = "Building a very long project name with many characters";
        let pb = build_progress(long_msg);
        pb.finish_and_clear();
    }

    #[test]
    fn test_file_progress_zero_total() {
        let pb = file_progress(0, "Empty");
        assert_eq!(pb.length(), Some(0));
        pb.finish_and_clear();
    }

    #[test]
    fn test_file_progress_large_total() {
        let pb = file_progress(1_000_000, "Large");
        assert_eq!(pb.length(), Some(1_000_000));
        pb.finish_and_clear();
    }

    #[test]
    fn test_multiple_spinners() {
        let pb1 = spinner("Task 1");
        let pb2 = spinner("Task 2");
        let pb3 = spinner("Task 3");

        pb1.finish_and_clear();
        pb2.finish_and_clear();
        pb3.finish_and_clear();
    }

    #[test]
    fn test_progress_reset() {
        let pb = file_progress(100, "Test");
        pb.set_position(50);
        pb.reset();
        assert_eq!(pb.position(), 0);
        pb.finish_and_clear();
    }

    #[test]
    fn test_progress_with_eta() {
        let pb = file_progress(1000, "Processing");
        for _ in 0..100 {
            pb.inc(1);
        }
        pb.finish_and_clear();
    }

    #[test]
    fn test_download_progress_bytes() {
        let pb = download_progress(1024 * 1024);
        pb.inc(512 * 1024);
        assert_eq!(pb.position(), 512 * 1024);
        pb.finish_and_clear();
    }

    #[test]
    fn test_concurrent_progress_bars() {
        let mp = multi_progress();
        let bars: Vec<_> = (0..5)
            .map(|i| mp.add(file_progress(10, &format!("Task {}", i))))
            .collect();

        for bar in bars {
            bar.finish_and_clear();
        }
    }
}
