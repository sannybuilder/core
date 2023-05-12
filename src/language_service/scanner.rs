use super::ffi::{Source, SymbolInfoMap, SymbolType};
use crate::dictionary::dictionary_num_by_str::DictNumByStr;
use crate::language_service::server::{CACHE_FILE_SYMBOLS, CACHE_FILE_TREE};
use std::fs;
use std::path::Path;

// from compiler.ini
const TOKEN_INCLUDE: i32 = 103;
const TOKEN_CONST: i32 = 65;
const TOKEN_END: i32 = 255;
const TOKEN_INT: i32 = 1;
const TOKEN_FLOAT: i32 = 2;
const TOKEN_STRING: i32 = 3;
const TOKEN_LONGSTRING: i32 = 4;
const TOKEN_HANDLE: i32 = 5;
const TOKEN_BOOL: i32 = 6;

fn document_tree_walk<'a>(
    content: &String,
    file_name: &String,
    reserved_words: &DictNumByStr,
    mut refs: &mut Vec<String>,
) -> Vec<String> {
    content
        .lines()
        .filter_map(|x| {
            // todo: use nom parser
            let mut words = x.split_ascii_whitespace();
            let first = words.next()?.to_ascii_lowercase();

            if let Some(token) = reserved_words.map.get(&first) {
                if token == &TOKEN_INCLUDE {
                    let mut include_path = words.collect::<String>();

                    if include_path.ends_with('}') {
                        include_path.pop();
                    }

                    let path = resolve_path(include_path, file_name)?;

                    // ignore cyclic paths
                    if refs.contains(&path) {
                        return None;
                    } else {
                        refs.push(path.clone());
                    }

                    let mut tree = match get_cached_tree(&path) {
                        Some(tree) => {
                            log::debug!("Using cached tree for file {}", path);
                            tree
                        }
                        None => {
                            log::debug!("Tree cache not found. Reading file {}", path);
                            file_walk(path.clone(), reserved_words, &mut refs)?
                        }
                    };
                    tree.push(path);
                    return Some(tree);
                }
            }
            None
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn file_walk(
    file_name: String,
    reserved_words: &DictNumByStr,
    mut refs: &mut Vec<String>,
) -> Option<Vec<String>> {
    let content = fs::read_to_string(&file_name).ok()?;
    let tree = document_tree_walk(&content, &file_name, reserved_words, &mut refs);

    log::debug!("Caching file tree {}", file_name);
    let mut cache = CACHE_FILE_TREE.lock().unwrap();
    cache.insert(file_name.clone(), tree.clone());

    Some(tree)
}

fn get_cached_tree(file_name: &String) -> Option<Vec<String>> {
    let cache = CACHE_FILE_TREE.lock().unwrap();
    cache.get(file_name).cloned()
}

fn resolve_path(p: String, parent_file: &String) -> Option<String> {
    let path = Path::new(&p);

    if path.is_absolute() {
        return Some(p);
    }

    let dir_name = Path::new(&parent_file).parent()?;
    let abs_name = dir_name.join(path);

    Some(String::from(abs_name.to_str()?))
}

pub fn document_tree<'a>(
    text: &String,
    reserved_words: &DictNumByStr,
    implicit_includes: &Vec<String>,
    source: &Source,
) -> Option<Vec<String>> {
    match source {
        Source::File(file_name) => {
            let mut refs: Vec<String> = vec![];
            let mut tree = document_tree_walk(text, file_name, reserved_words, &mut refs);

            tree.extend(implicit_includes.iter().filter_map(|include_path| {
                Some(resolve_path(
                    include_path.to_owned(),
                    &file_name.to_string(),
                )?)
            }));

            Some(tree)
        }
        Source::Memory => Some(implicit_includes.clone()),
    }
}

pub fn find_constants_from_file(
    file_name: &String,
    reserved_words: &DictNumByStr,
) -> Option<Vec<(String, SymbolInfoMap)>> {
    let mut cache = CACHE_FILE_SYMBOLS.lock().unwrap();
    match cache.get(file_name) {
        Some(symbols) => {
            log::debug!("Using cached symbols for file {}", file_name);
            Some(symbols.clone())
        }
        None => {
            log::debug!("Symbol cache not found. Reading file {}", file_name);
            let content = fs::read_to_string(file_name).ok()?;
            let symbols =
                find_constants(&content, reserved_words, &Source::File(file_name.clone()))?;
            cache.insert(file_name.clone(), symbols.clone());
            Some(symbols)
        }
    }
}

pub fn find_constants_from_memory(
    content: &String,
    reserved_words: &DictNumByStr,
) -> Option<Vec<(String, SymbolInfoMap)>> {
    find_constants(&content, reserved_words, &Source::Memory)
}

pub fn find_constants<'a>(
    content: &String,
    reserved_words: &DictNumByStr,
    source: &Source,
) -> Option<Vec<(String, SymbolInfoMap)>> {
    let mut lines = content.lines().enumerate();
    let mut res = vec![];
    let mut inside_const = false;
    let file_name = match source {
        Source::File(path) => Some(path.clone()),
        Source::Memory => None,
    };
    while let Some((line_number, mut line)) = lines.next() {
        line = line.trim();
        if line.contains("//") {
            line = line.split("//").next().unwrap();
        }
        if line.is_empty() {
            continue;
        }
        let mut words = line.split_ascii_whitespace();
        let first = words.next().unwrap_or("");
        match reserved_words.map.get(first) {
            Some(token) => match *token {
                TOKEN_CONST => inside_const = true,
                TOKEN_END if inside_const => inside_const = false,
                TOKEN_INT | TOKEN_FLOAT | TOKEN_STRING | TOKEN_LONGSTRING | TOKEN_HANDLE
                | TOKEN_BOOL => {
                    // inline variable declaration
                    let name = words.next().unwrap_or("");
                    if !name.is_empty() {
                        res.push((
                            name.to_ascii_lowercase(),
                            SymbolInfoMap {
                                line_number: line_number as u32,
                                _type: SymbolType::Var,
                                file_name: file_name.clone(),
                                value: None,
                            },
                        ))
                    }
                }
                _ => {}
            },
            _ if inside_const => {
                // todo: try nom parser
                let mut tokens = line.split('=');

                if let Some(name) = tokens.next() {
                    if let Some(value) = tokens.next() {
                        if let Some(_type) = get_type(value.trim()) {
                            res.push((
                                name.trim().to_ascii_lowercase(),
                                SymbolInfoMap {
                                    line_number: line_number as u32,
                                    _type,
                                    file_name: file_name.clone(),
                                    value: Some(String::from(value.trim())),
                                },
                            ))
                        }
                    }
                }
            }
            _ => {
                // ignore other lines
            }
        }
    }

    Some(res)
}

pub fn get_type(value: &str) -> Option<SymbolType> {
    if value.len() > 1 {
        if value.starts_with('$')
            || value.starts_with("v$")
            || value.starts_with("s$")
            || value.ends_with('@')
            || value.ends_with("@s")
            || value.ends_with("@v")
        {
            return Some(SymbolType::Var);
        }
        if value.starts_with('"') || value.starts_with('\'') {
            return Some(SymbolType::String);
        }
        if value.starts_with('#') {
            return Some(SymbolType::ModelName);
        }
        if value.starts_with('@') {
            return Some(SymbolType::Label);
        }
    }
    if let Some(_) = value.parse::<f32>().ok() {
        return Some(SymbolType::Number);
    }
    if value.starts_with("0x") || value.starts_with("-0x") || value.starts_with("+0x") {
        return Some(SymbolType::Number);
    }
    return None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let p = resolve_path(String::from("2.txt"), &String::from("C:/dev/1.txt")).unwrap();
        assert_eq!(p, String::from("C:/dev\\2.txt"));
    }
}
