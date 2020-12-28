use super::{
    ffi::{EditorHandle, NotifyCallback, SymbolInfo},
    watcher::FileWatcher,
    {scanner, symbol_table::SymbolTable},
};
use crate::dictionary::{
    dictionary_num_by_str::DictNumByStr,
    ffi::{CaseFormat, Duplicates},
};
use lazy_static::lazy_static;
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
    thread,
};

lazy_static! {
    static ref SYMBOL_TABLES: Mutex<HashMap<EditorHandle, SymbolTable>> =
        Mutex::new(HashMap::new());
    static ref WATCHED_FILES: Mutex<HashMap<String, HashSet<EditorHandle>>> =
        Mutex::new(HashMap::new());
    static ref RESERVED_WORDS: Mutex<DictNumByStr> = Mutex::new(DictNumByStr::new(
        Duplicates::Replace,
        CaseFormat::LowerCase,
        String::from(";"),
        String::from("=,"),
        true,
        false,
    ));
}

pub struct LanguageServer {
    watcher: FileWatcher,
    notify: NotifyCallback,
}

impl LanguageServer {
    pub fn new(notify: NotifyCallback) -> Self {
        RESERVED_WORDS
            .lock()
            .unwrap()
            .load_file("data\\compiler.ini");
        Self {
            watcher: FileWatcher::new(),
            notify,
        }
    }

    pub fn open(&mut self, file_name: &str, handle: EditorHandle) {
        let dict = RESERVED_WORDS.lock().unwrap();
        if let Some(tree) = scanner::document_tree(file_name, &dict) {
            for file in tree {
                self.start_watching(file, file_name, handle);
            }
        }

        let notify = self.notify;
        let main_file1 = String::from(file_name);

        // needed to unlock the RESERVED_WORDS mutex for scan routine
        drop(dict);

        // spawn initial scan for the opened document
        thread::spawn(move || LanguageServer::scan(&main_file1, &main_file1, notify));
    }

    fn start_watching(&mut self, file_name: String, main_file: &str, handle: EditorHandle) {
        let mut files = WATCHED_FILES.lock().unwrap();
        match files.get_mut(&file_name) {
            Some(v) => {
                v.insert(handle);
            }
            None => {
                let mut set = HashSet::new();
                set.insert(handle);
                files.insert(file_name.clone(), set);
            }
        }
        let notify = self.notify;
        let file_name1 = file_name.clone();
        let main_file1 = String::from(main_file);

        self.watcher
            .watch(file_name.as_str(), move |event| match event {
                hotwatch::Event::Write(_) => LanguageServer::scan(&file_name1, &main_file1, notify),
                _ => {
                    // todo: check out other events, e.g. file delete or move
                }
            });
    }

    fn scan(file_name: &String, main_file: &String, notify: NotifyCallback) {
        let files = WATCHED_FILES.lock().unwrap();
        if let Some(handles) = files.get(file_name.as_str()) {
            for &handle in handles {
                // todo: parallel scan?
                let mut table = SymbolTable::new();

                let dict = RESERVED_WORDS.lock().unwrap();

                if let Some(tree) = scanner::document_tree(main_file.as_str(), &dict) {
                    for file in tree {
                        if let Some(constants) = scanner::find_constants(file, &dict) {
                            table.add(constants);
                        }
                    }
                }

                SYMBOL_TABLES.lock().unwrap().insert(handle, table);
                notify(handle);
            }
        }
    }

    pub fn close(&mut self, file_name: &str, handle: EditorHandle) {
        SYMBOL_TABLES.lock().unwrap().remove(&handle);
        let mut f = WATCHED_FILES.lock().unwrap();
        if let Some(v) = f.get_mut(file_name) {
            v.remove(&handle);
            if v.len() == 0 {
                self.watcher.unwatch(file_name);
            }
        }
    }

    pub fn find(
        &mut self,
        symbol: &str,
        handle: EditorHandle,
        file_name: &str,
    ) -> Option<SymbolInfo> {
        let st = SYMBOL_TABLES.lock().unwrap();
        let table = st.get(&handle)?;
        let map = table.symbols.get(&symbol.to_ascii_lowercase())?;
        Some(SymbolInfo {
            line_number: if !map.file_name.eq(file_name) {
                1
            } else {
                map.line_number
            },
            _type: map._type,
        })
    }
}
