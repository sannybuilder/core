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
    static ref DOCUMENT_MAP: Mutex<HashMap<EditorHandle, String>> = Mutex::new(HashMap::new());
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

    pub fn connect(&mut self, file_name: &str, handle: EditorHandle) {
        let dict = RESERVED_WORDS.lock().unwrap();
        let mut documents = DOCUMENT_MAP.lock().unwrap();
        let main_file = String::from(file_name);
        documents.insert(handle, main_file.clone());
        drop(documents);

        // todo: maybe merge this with initial scan
        if let Some(tree) = scanner::document_tree(file_name, &dict) {
            for file in tree {
                self.start_watching(file, handle);
            }
        }

        // needed to unlock the RESERVED_WORDS mutex for scan routine
        drop(dict);

        // spawn initial scan for the opened document
        let notify = self.notify;
        thread::spawn(move || {
            let mut symbol_table = SYMBOL_TABLES.lock().unwrap();
            let dict = RESERVED_WORDS.lock().unwrap();
            LanguageServer::scan_file(handle, &main_file, &mut symbol_table, &dict, &notify)
        });
    }

    fn start_watching(&mut self, file_name: String, handle: EditorHandle) {
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

        self.watcher
            .watch(file_name.as_str(), move |event| match event {
                hotwatch::Event::Write(_) => LanguageServer::scan(&file_name1, notify),
                _ => {
                    // todo: check out other events, e.g. file delete or move
                }
            });
    }

    fn scan(file_name: &String, notify: NotifyCallback) {
        let files = WATCHED_FILES.lock().unwrap();
        let documents = DOCUMENT_MAP.lock().unwrap();
        let mut symbol_table = SYMBOL_TABLES.lock().unwrap();
        let dict = RESERVED_WORDS.lock().unwrap();
        if let Some(handles) = files.get(file_name.as_str()) {
            for &handle in handles {
                // todo: parallel scan?
                if let Some(file_name) = documents.get(&handle) {
                    LanguageServer::scan_file(handle, file_name, &mut symbol_table, &dict, &notify);
                }
            }
        }
    }

    fn scan_file(
        handle: EditorHandle,
        file_name: &String,
        symbol_table: &mut HashMap<EditorHandle, SymbolTable>,
        reserved_words: &DictNumByStr,
        notify: &NotifyCallback,
    ) {
        let mut table = SymbolTable::new();
        if let Some(tree) = scanner::document_tree(file_name.as_str(), &reserved_words) {
            for file in tree {
                if let Some(constants) = scanner::find_constants(file, &reserved_words) {
                    table.add(constants);
                }
            }
        }

        symbol_table.insert(handle, table);
        notify(handle);
    }

    pub fn disconnect(&mut self, handle: EditorHandle) {
        SYMBOL_TABLES.lock().unwrap().remove(&handle);
        DOCUMENT_MAP.lock().unwrap().remove(&handle);

        // disconnect editor from all files and stop watching orphan references
        let _drained = WATCHED_FILES
            .lock()
            .unwrap()
            .drain_filter(|k, v| {
                v.remove(&handle);
                if v.len() == 0 {
                    self.watcher.unwatch(k);
                    return true;
                }
                return false;
            })
            .collect::<Vec<_>>();
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
