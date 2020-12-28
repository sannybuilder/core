use hotwatch::{Event, Hotwatch};
use std::collections::HashMap;

pub struct FileWatcher {
    watcher: Option<Hotwatch>,
    files: HashMap<String, i32>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: Hotwatch::new().ok(),
            files: HashMap::new(),
        }
    }

    pub fn watch<F>(&mut self, file_name: &str, handler: F)
    where
        F: 'static + FnMut(Event) + Send,
    {
        if let Some(watcher) = &mut self.watcher {
            let key = String::from(file_name);
            self.files
                .raw_entry_mut()
                .from_key(&key)
                .and_modify(|_k, v| *v += 1)
                .or_insert_with(|| {
                    watcher.watch(file_name, handler);
                    (key, 1)
                });
        };
    }

    pub fn unwatch(&mut self, file_name: &str) {
        if let Some(watcher) = &mut self.watcher {
            let key = String::from(file_name);

            if let Some(v) = self.files.get_mut(&key) {
                *v -= 1;
                if *v == 0 {
                    self.files.remove(&key);
                    watcher.unwatch(file_name);
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let mut f = FileWatcher::new();
        f.watch("1.txt", |_| {});
        f.watch("1.txt", |_| {});

        assert_eq!(f.files.len(), 1);

        f.unwatch("1.txt");
        f.unwatch("1.txt");

        assert_eq!(f.files.len(), 0);
    }
}
