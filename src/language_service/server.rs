use super::{
    ffi::{DocumentInfo, EditorHandle, Source, Status, SymbolInfo, SymbolInfoMap},
    watcher::FileWatcher,
    {scanner, symbol_table::SymbolTable},
};
use crate::dictionary::{config, dictionary_num_by_str::DictNumByStr, ffi::CaseFormat};
use lazy_static::lazy_static;
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
    static ref RESERVED_WORDS: Mutex<DictNumByStr> = Mutex::new(DictNumByStr::new(
        config::ConfigBuilder::new()
            .set_case_format(CaseFormat::LowerCase)
            .build()
    ));
    static ref FILE_WATCHER: Mutex<FileWatcher> = Mutex::new(FileWatcher::new());
    static ref IMPLICIT_INCLUDES: Mutex<HashMap<EditorHandle, Vec<String>>> =
        Mutex::new(HashMap::new());
    pub static ref CACHE_FILE_TREE: Mutex<HashMap<String, Vec<String>>> =
        Mutex::new(HashMap::new());
    pub static ref CACHE_FILE_SYMBOLS: Mutex<HashMap<String, Vec<(String, SymbolInfoMap)>>> =
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

    pub fn connect(&mut self, source: Source, handle: EditorHandle, static_constants_file: &str) {
        log::debug!("New client {} connected with source {:?}", handle, source);

        SOURCE_MAP.lock().unwrap().insert(handle, source.clone());

        let static_constants_file = String::from(static_constants_file);
        IMPLICIT_INCLUDES
            .lock()
            .unwrap()
            .insert(handle, vec![static_constants_file.clone()]);

        status_change(handle, Status::PendingScan);
    }

    pub fn disconnect(&mut self, handle: EditorHandle) {
        log::debug!("Client {} disconnected", handle);
        SYMBOL_TABLES.lock().unwrap().remove(&handle);
        SOURCE_MAP.lock().unwrap().remove(&handle);
        IMPLICIT_INCLUDES.lock().unwrap().remove(&handle);
        let mut watcher = FILE_WATCHER.lock().unwrap();

        // disconnect editor from all files and stop watching orphan references
        let _drained = WATCHED_FILES
            .lock()
            .unwrap()
            .drain_filter(|k, v| {
                v.remove(&handle);
                if v.is_empty() {
                    LanguageServer::invalidate_file_cache(k);
                    watcher.unwatch(k);
                    return true;
                }
                return false;
            })
            .collect::<Vec<_>>();
    }

    pub fn find(&mut self, symbol: &str, handle: EditorHandle) -> Option<SymbolInfo> {
        let st = SYMBOL_TABLES.lock().unwrap();
        let table = st.get(&handle)?;
        let map = table.symbols.get(&symbol.to_ascii_lowercase())?;
        Some(SymbolInfo {
            line_number: if map.file_name.is_some() {
                0
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

    pub fn filter_constants_by_name(
        &self,
        needle: &str,
        handle: EditorHandle,
    ) -> Option<Vec<(String, String)>> {
        let st = SYMBOL_TABLES.lock().unwrap();
        let table = st.get(&handle)?;
        let needle = needle.to_ascii_lowercase();
        Some(
            table
                .symbols
                .iter()
                .filter_map(|(name, map)| {
                    name.to_ascii_lowercase()
                        .starts_with(&needle)
                        .then_some((map.name_no_format.clone(), map.value.clone()?.clone()))
                })
                .collect::<Vec<_>>(),
        )
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

    fn update_watchers(tree: &Vec<String>, handle: EditorHandle) {
        log::debug!("Updating file watchers");

        let mut watched_files = WATCHED_FILES.lock().unwrap();
        let mut watcher = FILE_WATCHER.lock().unwrap();

        // remove handle from dereferenced files
        let _drained = watched_files
            .drain_filter(|k, v| {
                if !tree.contains(k) && v.contains(&handle) {
                    v.remove(&handle);
                    if v.is_empty() {
                        LanguageServer::invalidate_file_cache(k);
                        watcher.unwatch(k);
                        return true;
                    }
                }
                return false;
            })
            .collect::<Vec<_>>();

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
                            // todo: check if possible to use file name from event payload
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

    fn invalidate_file_cache(file_name: &String) {
        let mut cache = CACHE_FILE_TREE.lock().unwrap();

        // invalidate cache for all files referencing this file
        // todo: change file cache to only store its own references and not children
        // then use cache.remove(file_name)
        let _drained = cache
            .drain_filter(|k, v| {
                if k == file_name || v.contains(file_name) {
                    log::debug!("Invalidating tree cache for file {}", k);
                    return true;
                }
                return false;
            })
            .collect::<Vec<_>>();

        CACHE_FILE_SYMBOLS
            .lock()
            .unwrap()
            .remove(file_name)
            .and_then(|_| {
                log::debug!("Invalidating symbol cache for file {}", file_name);
                Some(())
            });
    }

    fn rescan(file_name: &String) {
        log::debug!("File {} has changed", file_name);
        let files = WATCHED_FILES.lock().unwrap();

        if let Some(handles) = files.get(file_name.as_str()) {
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
        let mut symbol_table = SYMBOL_TABLES.lock().unwrap();
        let implicit_includes = IMPLICIT_INCLUDES.lock().unwrap();

        log::debug!("Reading source to build document tree");
        if let Some(source) = sources.get(&handle) {
            let mut table = SymbolTable::new();

            let v = vec![];
            let includes = implicit_includes.get(&handle).unwrap_or(&v);

            if let Some(tree) = scanner::document_tree(&text, &dict, includes, source) {
                log::debug!("Document tree is ready: {} child entries", tree.len());

                LanguageServer::update_watchers(&tree, handle);
                for file in tree {
                    log::debug!("Fetch symbols from file {}", file);
                    if let Some(constants) = scanner::find_constants_from_file(&file, &dict) {
                        log::debug!("Found {} symbols", constants.len());
                        table.add(constants);
                    }
                }
            }

            log::debug!("Fetch symbols from opened document");
            if let Some(constants) = scanner::find_constants_from_memory(&text, &dict) {
                log::debug!("Found {} symbols", constants.len());
                table.add(constants);
            }

            symbol_table.insert(handle, table);
            status_change(handle, Status::Idle);
        }

        log::debug!("Finalize scan for client: {}", handle);
    }

    fn cwd() -> Option<PathBuf> {
        Some(env::current_exe().ok()?.parent()?.to_path_buf())
    }
}

fn status_change(handle: u32, status: Status) {
    use crate::sdk::messages::{send_message, WM_ONSTATUSCHANGE};
    send_message(WM_ONSTATUSCHANGE, handle as _, status as _)
}
