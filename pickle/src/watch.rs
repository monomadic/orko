use notify::{RecommendedWatcher, Watcher, RecursiveMode, RawEvent};
use std::sync::mpsc::{channel, Receiver};
use std::path::{Path};

pub type ChangeEvent = RawEvent;

pub struct FileWatcher {
    pub watcher : RecommendedWatcher,
    pub change_events: Receiver<RawEvent>,
}

pub fn watch(path:&Path) -> FileWatcher {
    let (tx, notifier_rx) = channel::<RawEvent>();
    let mut resource_file_watcher : RecommendedWatcher = Watcher::new_raw(tx).expect("a watcher");
    resource_file_watcher.watch(path, RecursiveMode::Recursive).expect("watching resources path");

    FileWatcher {
        watcher: resource_file_watcher,
        change_events: notifier_rx,
    }
}
