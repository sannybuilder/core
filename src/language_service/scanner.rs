use super::ffi::{Source, SymbolInfoMap, SymbolType};
use crate::dictionary::dictionary_num_by_str::DictNumByStr;
use crate::language_service::server::{CACHE_FILE_SYMBOLS, CACHE_FILE_TREE};
use crate::utils::compiler_const::*;
use std::fs;
use std::path::Path;

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
                if token == &TOKEN_INCLUDE || token == &TOKEN_INCLUDE_ONCE {
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

    // todo: 
    // If the file path is relative, the compiler scans directories in the following order to find the file:
    // 1. directory of the file with the directive
    // 2. data folder for the current edit mode
    // 3. Sanny Builder root directory
    // 4. the game directory
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
    let mut lines: Vec<String> = vec![];
    let mut line = String::new();
    let mut chars = content.chars();

    'outer: while let Some(c) = chars.next() {
        match c {
            '\n' => {
                lines.push(line);
                line = String::new();
            }
            '{' => {
                // { } block
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(_) => {} // ignore other chars inside block
                        None => break 'outer,
                    }
                }
            }
            '/' => match chars.next() {
                // /* */ comment
                Some('*') => loop {
                    match chars.next() {
                        Some('*') => {
                            if chars.next() == Some('/') {
                                break;
                            }
                        }
                        Some(_) => {} // ignore other chars inside comment
                        None => break 'outer,
                    }
                },
                // // comment
                Some('/') => {
                    loop {
                        match chars.next() {
                            Some('\n') => {
                                lines.push(line);
                                line = String::new();
                                break;
                            }
                            Some(_) => {} // ignore other chars inside comment
                            None => break 'outer,
                        }
                    }
                }
                Some(c) => {
                    line.push('/');
                    line.push(c);
                }
                None => {
                    break 'outer;
                }
            },

            c => {
                // trim left
                if !c.is_ascii_whitespace() || !line.is_empty() {
                    line.push(c);
                }
            }
        }
    }
    lines.push(line);

    // let mut lines = content.lines().enumerate();
    let mut found_constants = vec![];
    let mut inside_const = false;
    let file_name = match source {
        Source::File(path) => Some(path.clone()),
        Source::Memory => None,
    };
    for (line_number, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        let mut words = line.split_ascii_whitespace();
        let first = match words.next() {
            Some(word) => word.to_ascii_lowercase(),
            None => continue,
        };
        match reserved_words.map.get(&first) {
            Some(token) => match *token {
                TOKEN_CONST => {
                    let rest = words.collect::<String>();

                    if !rest.is_empty() {
                        let declarations = split_const_line(&rest);

                        for declaration in declarations.iter() {
                            process_const_declaration(
                                &declaration,
                                &mut found_constants,
                                line_number,
                                &file_name,
                            );
                        }
                    } else {
                        inside_const = true;
                    }
                }
                TOKEN_END if inside_const => inside_const = false,
                TOKEN_INT | TOKEN_FLOAT | TOKEN_STRING | TOKEN_LONGSTRING | TOKEN_HANDLE
                | TOKEN_BOOL => {
                    // inline variable declaration

                    let rest = words.collect::<String>();
                    let names = split_const_line(&rest);

                    for name in names {
                        process_var_declaration(
                            &name,
                            &mut found_constants,
                            line_number,
                            &file_name,
                        )
                    }
                }
                _ => {}
            },
            _ if inside_const => {
                let declarations = split_const_line(line);

                for declaration in declarations.iter() {
                    process_const_declaration(
                        &declaration,
                        &mut found_constants,
                        line_number,
                        &file_name,
                    );
                }
            }
            _ => {
                // ignore other lines
            }
        }
    }

    Some(found_constants)
}

pub fn process_const_declaration(
    line: &str,
    found_constants: &mut Vec<(String, SymbolInfoMap)>,
    line_number: usize,
    file_name: &Option<String>,
) {
    let mut tokens = line.split('=');

    let Some(name) = tokens.next() else { return };
    let name = name.trim();
    let name_lower = name.to_ascii_lowercase();
    if found_constants.iter().any(|(n, _)| n == &name_lower) {
        log::debug!(
            "Found duplicate const declaration {} in line {}",
            name,
            line_number + 1
        );
        return;
    }

    let Some(value) = tokens.next() else { return };
    let value = value.trim();

    macro_rules! add_to_constants {
        ($type:expr) => {
            found_constants.push((
                name_lower,
                SymbolInfoMap {
                    line_number: line_number as u32,
                    _type: $type,
                    file_name: file_name.clone(),
                    value: Some(String::from(value)),
                    name_no_format: name.to_string(),
                },
            ));
        };
    }

    match get_type(value) {
        Some(_type) => {
            add_to_constants!(_type);
        }
        None => {
            if let Some((_, symbol)) = found_constants
                .iter()
                .find(|x| x.0 == value.to_ascii_lowercase())
            {
                add_to_constants!(symbol._type);
            };
        }
    }
}

pub fn process_var_declaration(
    line: &str,
    found_constants: &mut Vec<(String, SymbolInfoMap)>,
    line_number: usize,
    file_name: &Option<String>,
) {
    let mut tokens = line.split('=');

    let Some(mut name) = tokens.next() else { return };

    if let Some(pos) = name.find('[') {
        name = &name[..pos];
    }

    let name = name.trim();
    let name_lower = name.to_ascii_lowercase();
    if found_constants.iter().any(|(n, _)| n == &name_lower) {
        log::debug!(
            "Found duplicate const declaration {} in line {}",
            name,
            line_number + 1
        );
        return;
    }
    found_constants.push((
        name_lower,
        SymbolInfoMap {
            line_number: line_number as u32,
            _type: SymbolType::Var,
            file_name: file_name.clone(),
            value: None,
            name_no_format: name.to_string(),
        },
    ))
}

pub fn split_const_line(line: &str) -> Vec<String> {
    // iterate over chars, split by , ignore commas inside parentheses
    let mut result = vec![];
    let mut current: usize = 0;
    let mut inside_parentheses = false;

    for (i, c) in line.chars().enumerate() {
        match c {
            '(' => {
                inside_parentheses = true;
            }
            ')' => {
                inside_parentheses = false;
            }
            ',' => {
                if !inside_parentheses {
                    result.push(line[current..i].to_string());
                    current = i + 1;
                }
            }
            _ => {}
        }
    }

    if current < line.len() {
        result.push(line[current..].to_string());
    }

    result
}

pub fn get_type(value: &str) -> Option<SymbolType> {
    if value.len() > 1 {
        if value.starts_with('$')
            || value.starts_with("v$")
            || value.starts_with("s$")
            || value.ends_with('@')
            || value.ends_with("@s")
            || value.ends_with("@v")
            || (value.ends_with(")") && value.contains("@("))
        // arrays 0@(1@,2i)
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
        if value.starts_with("0x")
            || value.starts_with("-0x")
            || value.starts_with("+0x")
            || value.starts_with("0b")
            || value.starts_with("-0b")
            || value.starts_with("+0b")
        {
            return Some(SymbolType::Number);
        }
    }
    if let Some(_) = value.parse::<f32>().ok() {
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
