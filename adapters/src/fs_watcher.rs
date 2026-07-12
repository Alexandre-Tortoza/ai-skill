//! Filesystem watcher that emits a signal when skill directories change.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

/// Watches one or more directories and emits a unit signal on a channel
/// whenever a filesystem event occurs (debounced to at most one per 300 ms).
pub struct FsWatcher {
    _watcher: RecommendedWatcher,
    /// Receiving end — a `()` is sent whenever a change is detected.
    pub rx: Receiver<()>,
    watched_paths: usize,
}

impl FsWatcher {
    /// Creates a new watcher for the given paths.
    pub fn new(paths: &[PathBuf]) -> Result<Self, Box<dyn std::error::Error>> {
        let (event_tx, event_rx) = mpsc::channel::<notify::Result<Event>>();
        let (debounce_tx, rx) = mpsc::channel::<()>();

        let mut watcher = notify::recommended_watcher(event_tx)?;
        let mut watched_paths = 0;
        for path in paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                watched_paths += 1;
            }
        }

        // Debounce thread: drain events into a single () signal, at most one per 300ms.
        std::thread::spawn(move || {
            while let Ok(_event) = event_rx.recv() {
                let _ = debounce_tx.send(());
                // drain any further events in the next 300ms
                let deadline = std::time::Instant::now() + Duration::from_millis(300);
                while std::time::Instant::now() < deadline {
                    let timeout = deadline.saturating_duration_since(std::time::Instant::now());
                    if timeout.is_zero() {
                        break;
                    }
                    match event_rx.recv_timeout(timeout) {
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            rx,
            watched_paths,
        })
    }

    /// Returns how many requested roots were present and are actively watched.
    pub fn watched_paths(&self) -> usize {
        self.watched_paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn new_with_temp_dir_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let watcher = FsWatcher::new(&[dir.path().to_path_buf()]);
        assert!(watcher.is_ok());
    }

    #[test]
    fn creating_file_triggers_event() {
        let dir = TempDir::new().unwrap();
        let watcher = FsWatcher::new(&[dir.path().to_path_buf()]).unwrap();

        std::thread::sleep(Duration::from_millis(50)); // let watcher settle
        std::fs::write(dir.path().join("test.txt"), "hello").unwrap();

        let received = watcher.rx.recv_timeout(Duration::from_secs(2));
        assert!(received.is_ok(), "expected watcher event within 2s");
    }

    #[test]
    fn empty_paths_does_not_panic() {
        let watcher = FsWatcher::new(&[]);
        assert!(watcher.is_ok());
    }

    #[test]
    fn watched_paths_counts_existing_roots_only() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("missing");
        let watcher = FsWatcher::new(&[dir.path().to_path_buf(), missing]).unwrap();

        assert_eq!(watcher.watched_paths(), 1);
    }
}
