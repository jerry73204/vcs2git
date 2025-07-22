use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tracing::info;

pub struct ProgressReporter {
    enabled: bool,
    bar: Option<ProgressBar>,
}

impl ProgressReporter {
    pub fn new(enabled: bool, total_operations: u64) -> Self {
        if enabled && total_operations > 0 {
            let bar = ProgressBar::new(total_operations);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("##-"),
            );
            bar.enable_steady_tick(Duration::from_millis(100));

            Self {
                enabled,
                bar: Some(bar),
            }
        } else {
            Self { enabled, bar: None }
        }
    }

    pub fn set_message(&self, msg: &str) {
        if let Some(bar) = &self.bar {
            bar.set_message(msg.to_string());
        }
    }

    pub fn inc(&self, delta: u64) {
        if let Some(bar) = &self.bar {
            bar.inc(delta);
        }
    }

    pub fn finish_with_message(&self, msg: &str) {
        if let Some(bar) = &self.bar {
            bar.finish_with_message(msg.to_string());
        }
    }

    pub fn println(&self, msg: &str) {
        if self.enabled {
            if let Some(bar) = &self.bar {
                bar.println(msg);
            } else {
                info!("{msg}");
            }
        } else {
            info!("{msg}");
        }
    }
}

impl Drop for ProgressReporter {
    fn drop(&mut self) {
        if let Some(bar) = &self.bar {
            bar.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_reporter_disabled() {
        let progress = ProgressReporter::new(false, 10);
        assert!(!progress.enabled);
        assert!(progress.bar.is_none());

        // These should not panic
        progress.set_message("test");
        progress.inc(1);
        progress.finish_with_message("done");
        progress.println("message");
    }

    #[test]
    fn test_progress_reporter_enabled() {
        let progress = ProgressReporter::new(true, 10);
        assert!(progress.enabled);
        assert!(progress.bar.is_some());

        // These should not panic
        progress.set_message("test");
        progress.inc(1);
        progress.inc(2);
        progress.finish_with_message("done");
    }

    #[test]
    fn test_progress_reporter_zero_operations() {
        let progress = ProgressReporter::new(true, 0);
        assert!(progress.enabled);
        assert!(progress.bar.is_none());

        // Should work like disabled
        progress.println("message");
    }
}
