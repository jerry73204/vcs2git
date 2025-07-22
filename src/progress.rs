use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct ProgressReporter {
    bar: ProgressBar,
}

impl ProgressReporter {
    pub fn new(total_operations: u64) -> Self {
        assert!(total_operations > 0, "total_operations must be non-zero");

        let bar = ProgressBar::new(total_operations);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        bar.enable_steady_tick(Duration::from_millis(100));

        Self { bar }
    }

    pub fn set_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());
    }

    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn finish_with_message(&self, msg: &str) {
        self.bar.finish_with_message(msg.to_string());
    }

    pub fn println(&self, msg: &str) {
        self.bar.println(msg);
    }
}

impl Drop for ProgressReporter {
    fn drop(&mut self) {
        self.bar.finish_and_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_reporter() {
        let progress = ProgressReporter::new(10);

        // These should not panic
        progress.set_message("test");
        progress.inc(1);
        progress.inc(2);
        progress.finish_with_message("done");
    }
}
