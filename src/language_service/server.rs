use super::{
    ffi::{DocumentInfo, EditorHandle, Source, Status},
    watcher::FileWatcher,
    {
        scanner,
        symbol_table::{SymbolInfoMap, SymbolTable},
    },
};
use crate::{
    dictionary::{config, ffi::CaseFormat, DictNumByString},
    namespaces::namespaces::Namespaces,
};
use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf,
    sync::{
        mpsc::{channel, Sender, TryRecvError},
        Mutex,
    },
    thread,
    time::Duration,
};

lazy_static! {
    static ref SYMBOL_TABLES: Mutex<HashMap<EditorHandle, SymbolTable>> =
        Mutex::new(HashMap::new());
    static ref WATCHED_FILES: Mutex<HashMap<String, HashSet<EditorHandle>>> =
        Mutex::new(HashMap::new());
    static ref SOURCE_MAP: Mutex<HashMap<EditorHandle, Source>> = Mutex::new(HashMap::new());
    static ref RESERVED_WORDS: Mutex<DictNumByString> = Mutex::new(DictNumByString::new(
        config::ConfigBuilder::new()
            .set_case_format(CaseFormat::LowerCase)
            .build()
    ));
    static ref FILE_WATCHER: Mutex<FileWatcher> = Mutex::new(FileWatcher::new());
    static ref IMPLICIT_INCLUDES: Mutex<HashMap<EditorHandle, Vec<String>>> =
        Mutex::new(HashMap::new());
    static ref CLASS_NAMES: Mutex<HashMap<EditorHandle, Vec<String>>> = Mutex::new(HashMap::new());
    pub static ref CACHE_FILE_SYMBOLS: Mutex<HashMap</*file name*/String, SymbolTable>> =
        Mutex::new(HashMap::new());
}

pub struct LanguageServer {
    pub message_queue: Sender<(EditorHandle, String)>,
}

impl LanguageServer {
    pub fn new() -> Self {
        let compiler_ini_path = LanguageServer::cwd()
            .unwrap_or(PathBuf::from(""))
            .join("data\\compiler.ini");

        if let Some(path) = compiler_ini_path.to_str() {
            RESERVED_WORDS.lock().unwrap().load_file(path);
        }

        log::debug!("Language service created");

        let message_queue = LanguageServer::setup_message_queue();
        Self { message_queue }
    }

    pub fn connect(
        &mut self,
        source: Source,
        handle: EditorHandle,
        static_constants_file: &str,
        classes_file: &str,
    ) {
        log::debug!("New client {} connected with source {:?}", handle, source);

        SOURCE_MAP.lock().unwrap().insert(handle, source.clone());

        IMPLICIT_INCLUDES
            .lock()
            .unwrap()
            .insert(handle, vec![String::from(static_constants_file)]);

        let mut ns = Namespaces::new();
        ns.load_classes(classes_file);

        CLASS_NAMES.lock().unwrap().insert(
            handle,
            ns.map_op_by_name.keys().cloned().collect::<Vec<_>>(),
        );

        status_change(handle, Status::PendingScan);
    }

    pub fn disconnect(&mut self, handle: EditorHandle) {
        log::debug!("Client {} disconnected", handle);
        SYMBOL_TABLES.lock().unwrap().remove(&handle);
        SOURCE_MAP.lock().unwrap().remove(&handle);
        IMPLICIT_INCLUDES.lock().unwrap().remove(&handle);
        CLASS_NAMES.lock().unwrap().remove(&handle);
        let mut watcher = FILE_WATCHER.lock().unwrap();

        // disconnect editor from all files and stop watching orphan references
        WATCHED_FILES.lock().unwrap().retain(|k, v| {
            v.remove(&handle);
            if v.is_empty() {
                LanguageServer::invalidate_file_cache(k);
                watcher.unwatch(k);
                return false;
            }
            return true;
        });
    }

    pub fn find(
        &mut self,
        symbol: &str,
        handle: EditorHandle,
        line_number: usize,
    ) -> Option<SymbolInfoMap> {
        let st = SYMBOL_TABLES.lock().unwrap();
        let table = st.get(&handle)?;
        let symbol_infos = table.symbols.get(&symbol.to_ascii_lowercase())?;

        for symbol_info in symbol_infos {
            if symbol_info.is_visible_at(line_number) {
                // check if symbol is visible in current scope in current line
                return Some(symbol_info.clone());
            }
        }
        None
    }

    pub fn get_document_info(&self, handle: EditorHandle) -> DocumentInfo {
        DocumentInfo {
            is_active: SYMBOL_TABLES.lock().unwrap().get(&handle).is_some(),
        }
    }

    pub fn filter_constants_by_name(
        &self,
        needle: &str,
        handle: EditorHandle,
        line_number: usize,
    ) -> Option<Vec<String>> {
        let st = SYMBOL_TABLES.lock().unwrap();
        let table = st.get(&handle)?;
        let needle = needle.to_ascii_lowercase().replace("_", "");

        let list = table
            .symbols
            .iter()
            .filter_map(|(name, map)| {
                if name.to_ascii_lowercase().replace("_", "").contains(&needle) {
                    for symbol_info in map {
                        if symbol_info.is_visible_at(line_number) {
                            return Some(name.clone());
                        }
                    }
                }
                return None;
            })
            .collect::<Vec<_>>();
        // list.sort_by(|v1, v2| v1.0.cmp(&v2.0));
        Some(list)
    }

    fn setup_message_queue() -> Sender<(EditorHandle, String)> {
        let (message_queue, receiver) = channel();
        thread::spawn(move || loop {
            let message = loop {
                match receiver.try_recv() {
                    Ok((handle, text)) => {
                        log::debug!("Got message from client {}", handle);
                        break Some((handle, text));
                    }
                    Err(TryRecvError::Empty) => break None,
                    Err(TryRecvError::Disconnected) => return,
                }
            };
            if let Some((current, text)) = message {
                LanguageServer::scan_client(current, text);
            }
            thread::sleep(Duration::from_millis(300));
        });

        message_queue
    }

    fn update_watchers(tree: &HashSet<String>, handle: EditorHandle) {
        log::debug!("Updating {} file watchers for handle {handle}", tree.len());

        let mut watched_files = WATCHED_FILES.lock().unwrap();
        let mut watcher = FILE_WATCHER.lock().unwrap();

        // remove handle from dereferenced files
        watched_files.retain(|k, v| {
            if !tree.contains(k) && v.contains(&handle) {
                v.remove(&handle);
                if v.is_empty() {
                    LanguageServer::invalidate_file_cache(k);
                    watcher.unwatch(k);
                    return false;
                }
            }
            return true;
        });

        // add new references
        for file_name in tree {
            match watched_files.get_mut(file_name) {
                Some(handles) => {
                    handles.insert(handle);
                }
                None => {
                    let mut set = HashSet::new();
                    set.insert(handle);
                    watched_files.insert(file_name.clone(), set);

                    let file_name1 = file_name.clone();
                    watcher.watch(file_name.as_str(), move |event| match event {
                        hotwatch::Event::Write(_) => {
                            LanguageServer::invalidate_file_cache(&file_name1);
                            LanguageServer::rescan(&file_name1)
                        }
                        _ => {
                            // todo: check out other events, e.g. file delete or move
                        }
                    });
                }
            }
        }
    }

    fn invalidate_file_cache(file_name: &str) {
        CACHE_FILE_SYMBOLS
            .lock()
            .unwrap()
            .remove(file_name)
            .and_then(|_| {
                log::debug!("Invalidating symbol cache for file {}", file_name);
                Some(())
            });
    }

    /// Schedule scan for all clients referencing this file
    fn rescan(file_name: &str) {
        log::debug!("File {} has changed", file_name);
        let files = WATCHED_FILES.lock().unwrap();

        if let Some(handles) = files.get(file_name) {
            log::debug!("Found {} dependent clients", handles.len());
            for &handle in handles {
                status_change(handle, Status::PendingScan)
            }
        }
    }

    fn scan_client(handle: EditorHandle, text: String) {
        log::debug!("Spawn scan for client {}", handle);
        let sources = SOURCE_MAP.lock().unwrap();
        let dict = RESERVED_WORDS.lock().unwrap();
        let classes = CLASS_NAMES.lock().unwrap();
        let mut symbol_table = SYMBOL_TABLES.lock().unwrap();
        let implicit_includes = IMPLICIT_INCLUDES.lock().unwrap();

        if let Some(source) = sources.get(&handle) {
            log::debug!("Reading source {:?} to build document tree", source);
            let mut table = SymbolTable::new();

            let v = vec![];
            let includes = implicit_includes.get(&handle).unwrap_or(&v);
            let v = vec![];
            let classes = classes.get(&handle).unwrap_or(&v);

            let mut visited = HashSet::new();
            let mut scope_stack = vec![(0, 0)];
            scanner::scan_document(
                &text,
                &dict,
                includes,
                source,
                &classes,
                &mut table,
                &mut visited,
                &mut scope_stack,
            );
            LanguageServer::update_watchers(&visited, handle);

            symbol_table.insert(handle, table);
            status_change(handle, Status::Idle);
        }

        log::debug!("Finalize scan for client: {}", handle);
    }

    fn cwd() -> Option<PathBuf> {
        Some(env::current_exe().ok()?.parent()?.to_path_buf())
    }
}

/// Send status change message to Sanny Builder
fn status_change(handle: u32, status: Status) {
    use crate::sdk::messages::{send_message, WM_ONSTATUSCHANGE};
    send_message(WM_ONSTATUSCHANGE, handle as _, status as _)
}
