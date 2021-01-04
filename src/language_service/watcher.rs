use hotwatch::{Event, Hotwatch};

pub struct FileWatcher {
    watcher: Option<Hotwatch>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: Hotwatch::new().ok(),
        }
    }

    pub fn watch<F>(&mut self, file_name: &str, handler: F)
    where
        F: 'static + FnMut(Event) + Send,
    {
        if let Some(watcher) = &mut self.watcher {
            log::debug!("Start watching file {}", file_name);
            match watcher.watch(file_name, handler) {
                Ok(_) => log::debug!("Start watching file {}", file_name),
                Err(_) => log::debug!("Can't start watching file {}", file_name),
            }
        };
    }

    pub fn unwatch(&mut self, file_name: &str) {
        if let Some(watcher) = &mut self.watcher {
            match watcher.unwatch(file_name) {
                Ok(_) => log::debug!("Stop watching file {}", file_name),
                Err(_) => log::debug!("Can't stop watching file {}", file_name),
            }
        };
    }
}
