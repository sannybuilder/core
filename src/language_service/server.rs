use super::{
    ffi::{DocumentInfo, EditorHandle, NotifyCallback, Status, StatusChangeCallback, SymbolInfo},
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
    static ref FILE_WATCHER: Mutex<FileWatcher> = Mutex::new(FileWatcher::new());
}

pub struct LanguageServer {
    notify: NotifyCallback,
    status_change: StatusChangeCallback,
}

impl LanguageServer {
    pub fn new(notify: NotifyCallback, status_change: StatusChangeCallback) -> Self {
        RESERVED_WORDS
            .lock()
            .unwrap()
            .load_file("data\\compiler.ini");
        Self {
            notify,
            status_change,
        }
    }

    pub fn connect(&mut self, file_name: &str, handle: EditorHandle) {
        let mut documents = DOCUMENT_MAP.lock().unwrap();
        let main_file = String::from(file_name);
        documents.insert(handle, main_file.clone());
        drop(documents);

        // spawn initial scan for the opened document
        let notify = self.notify;
        let status_change = self.status_change;

        thread::spawn(move || {
            status_change(handle, Status::Scanning as i32);
            let dict = RESERVED_WORDS.lock().unwrap();

            if let Some(tree) = scanner::document_tree(&main_file, &dict) {
                let mut watcher = FILE_WATCHER.lock().unwrap();
                let mut table = SymbolTable::new();

                for file in tree {
                    LanguageServer::start_watching(
                        &mut watcher,
                        &file,
                        handle,
                        notify,
                        status_change,
                    );
                    if let Some(constants) = scanner::find_constants(&file, &dict) {
                        table.add(constants);
                    }
                }
                SYMBOL_TABLES.lock().unwrap().insert(handle, table);
                notify(handle);
            }
            status_change(handle, Status::Ready as i32);
        });
    }

    fn start_watching(
        watcher: &mut FileWatcher,
        file_name: &String,
        handle: EditorHandle,
        notify: NotifyCallback,
        status_change: StatusChangeCallback,
    ) {
        let mut files = WATCHED_FILES.lock().unwrap();
        match files.get_mut(file_name) {
            Some(v) => {
                v.insert(handle);
            }
            None => {
                let mut set = HashSet::new();
                set.insert(handle);
                files.insert(file_name.clone(), set);
            }
        }
        let file_name1 = file_name.clone();

        watcher.watch(file_name.as_str(), move |event| match event {
            hotwatch::Event::Write(_) => LanguageServer::rescan(&file_name1, notify, status_change),
            _ => {
                // todo: check out other events, e.g. file delete or move
            }
        });
    }

    fn rescan(file_name: &String, notify: NotifyCallback, status_change: StatusChangeCallback) {
        let files = WATCHED_FILES.lock().unwrap();
        let documents = DOCUMENT_MAP.lock().unwrap();
        let dict = RESERVED_WORDS.lock().unwrap();
        let mut symbol_table = SYMBOL_TABLES.lock().unwrap();

        if let Some(handles) = files.get(file_name.as_str()) {
            for &handle in handles {
                // todo: parallel scan?
                status_change(handle, Status::Scanning as i32);
                if let Some(file_name) = documents.get(&handle) {
                    let mut table = SymbolTable::new();
                    if let Some(tree) = scanner::document_tree(file_name.as_str(), &dict) {
                        for file in tree {
                            if let Some(constants) = scanner::find_constants(&file, &dict) {
                                table.add(constants);
                            }
                        }
                    }

                    symbol_table.insert(handle, table);
                    notify(handle);
                    status_change(handle, Status::Ready as i32);
                }
            }
        }
    }

    pub fn disconnect(&mut self, handle: EditorHandle) {
        SYMBOL_TABLES.lock().unwrap().remove(&handle);
        DOCUMENT_MAP.lock().unwrap().remove(&handle);
        let mut watcher = FILE_WATCHER.lock().unwrap();

        // disconnect editor from all files and stop watching orphan references
        let _drained = WATCHED_FILES
            .lock()
            .unwrap()
            .drain_filter(|k, v| {
                v.remove(&handle);
                if v.len() == 0 {
                    watcher.unwatch(k);
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

    pub fn get_document_info(&self, handle: EditorHandle) -> DocumentInfo {
        DocumentInfo {
            is_active: SYMBOL_TABLES.lock().unwrap().get(&handle).is_some(),
        }
    }
}
