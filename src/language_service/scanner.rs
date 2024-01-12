use super::ffi::{Source, SymbolInfoMap, SymbolType};
use super::symbol_table::SymbolTable;
use crate::dictionary::DictNumByString;
use crate::language_service::server::CACHE_FILE_SYMBOLS;
use crate::utils::compiler_const::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn file_walk(
    file_name: &String,
    reserved_words: &DictNumByString,
    visited: &mut HashSet<String>,
    class_names: &Vec<String>,
    table: &mut SymbolTable,
    scope_stack: &mut Vec<u32>,
    line_number: Option<usize>,
) {
    // ignore cyclic paths
    if !visited.insert(file_name.clone()) {
        return;
    }

    // if present, use file's cached symbols (all symbols from the file and all included files)
    if let Some(symbols) = CACHE_FILE_SYMBOLS.lock().unwrap().get(file_name) {
        log::debug!("Using cached symbols for file {}", file_name);
        table.extend(symbols);
        return;
    }

    log::debug!("Symbol cache not found. Reading file {}", file_name);
    let Ok(content) = fs::read_to_string(&file_name) else { return };

    // create a new table for this file and its descendants
    let mut local_table = SymbolTable::new();
    find_constants(
        &content,
        reserved_words,
        class_names,
        &Source::File(file_name.clone()),
        visited,
        scope_stack,
        &mut local_table,
        line_number,
    );

    // use found symbols and cache them
    table.extend(&local_table);
    CACHE_FILE_SYMBOLS
        .lock()
        .unwrap()
        .insert(file_name.clone(), local_table);
}

fn resolve_path(p: String, parent_file: &Option<String>) -> Option<String> {
    let path = Path::new(&p);

    if path.is_absolute() {
        return Some(p);
    }

    match parent_file {
        Some(x) => {
            // todo:
            // If the file path is relative, the compiler scans directories in the following order to find the file:
            // 1. directory of the file with the directive
            // 2. data folder for the current edit mode
            // 3. Sanny Builder root directory
            // 4. the game directory
            let dir_name = Path::new(&x).parent()?;
            let abs_name = dir_name.join(path);

            Some(String::from(abs_name.to_str()?))
        }
        None => None,
    }
}

/// read the source code and extract all constants and variables
/// if the file contains an include directive, recursively scan the included file
/// also, scan all implicit includes (constants.txt)
pub fn scan_document<'a>(
    text: &String,
    reserved_words: &DictNumByString,
    implicit_includes: &Vec<String>,
    source: &Source,
    class_names: &Vec<String>,
    table: &mut SymbolTable,
    visited: &mut HashSet<String>,
    scope_stack: &mut Vec<u32>,
) {
    for file_name in implicit_includes {
        file_walk(
            file_name,
            reserved_words,
            visited,
            class_names,
            table,
            scope_stack,
            Some(0),
        );
    }

    find_constants(
        text,
        reserved_words,
        class_names,
        source,
        visited,
        scope_stack,
        table,
        None, // line number to be determined as we parse the source code
    );
}

pub fn find_constants<'a>(
    content: &String,
    reserved_words: &DictNumByString,
    class_names: &Vec<String>,
    source: &Source,
    visited: &mut HashSet<String>,
    scope_stack: &mut Vec<u32>,
    table: &mut SymbolTable,
    line_number: Option<usize>,
) {
    let mut lines: Vec<String> = vec![];
    let mut line = String::new();
    let mut chars = content.chars().peekable();

    'outer: while let Some(c) = chars.next() {
        match c {
            '\n' => {
                lines.push(line);
                line = String::new();
            }
            '{' if chars.peek() != Some(&'$') => {
                // directives {$...}
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
    let mut inside_const = false;
    let file_name = match source {
        Source::File(path) => Some(path.clone()),
        Source::Memory => None,
    };
    for (_index, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        let mut words = line.split_ascii_whitespace();
        let first = match words.next() {
            Some(word) => word.to_ascii_lowercase(),
            None => continue,
        };
        /*
            if the file is a $include in the current document, then we need all its symbols (including deep $include's)
            to have a line number of the $include statement in the current document

            if the file is an implicit include (constants.txt), then all its symbols have a line number of 0
        */
        let line_number = line_number.unwrap_or(_index);

        let stack_id = scope_stack.len() as u32;

        match reserved_words.map.get(&first) {
            Some(token) => match *token {
                TOKEN_INCLUDE | TOKEN_INCLUDE_ONCE => {
                    let mut include_path = words.collect::<String>();

                    if include_path.ends_with('}') {
                        include_path.pop();
                    }

                    let Some(path) = resolve_path(include_path, &file_name) else { continue };

                    file_walk(
                        &path,
                        reserved_words,
                        visited,
                        class_names,
                        table,
                        scope_stack,
                        Some(line_number),
                    );
                }
                TOKEN_CONST => {
                    let rest = words.collect::<String>();

                    if !rest.is_empty() {
                        let declarations = split_const_line(&rest);

                        for declaration in declarations.iter() {
                            process_const_declaration(&declaration, table, line_number, stack_id);
                        }
                    } else {
                        inside_const = true;
                    }
                }
                TOKEN_END if inside_const => inside_const = false,
                TOKEN_END => {
                    log::debug!("Found end of block in line {}", line_number + 1);
                    if stack_id < 2 {
                        // global scope, only ends when the file ends
                        continue;
                    }
                    // number of nested blocks in the current function
                    let mut fn_blocks = scope_stack.last_mut().unwrap();
                    if *fn_blocks == 0 {
                        // there are no other open blocks in this function, this is the function's end

                        // find all symbols with stack_id
                        for (_, symbol) in table.symbols.iter_mut() {
                            if symbol.stack_id == stack_id {
                                symbol.end_line_number = line_number as u32;
                                symbol.stack_id = 0; // mark as processed
                            }
                        }

                        // delete function scope
                        scope_stack.pop();
                    } else {
                        // exit block
                        *fn_blocks -= 1;
                    }
                }
                TOKEN_INT | TOKEN_FLOAT | TOKEN_STRING | TOKEN_LONGSTRING | TOKEN_HANDLE
                | TOKEN_BOOL => {
                    // inline variable declaration

                    let rest = words.collect::<String>();
                    let names = split_const_line(&rest);

                    for name in names {
                        process_var_declaration(&name, table, line_number, stack_id)
                    }
                }
                TOKEN_FUNCTION => {
                    // push new scope
                    scope_stack.push(0);
                }

                TOKEN_IF | TOKEN_FOR | TOKEN_WHILE | TOKEN_SWITCH => {
                    if stack_id < 2 {
                        // global scope, only ends when the file ends
                        continue;
                    }
                    let function_scope = scope_stack.last_mut().unwrap();
                    // enter block
                    *function_scope += 1;
                }
                _ => {}
            },
            _ if inside_const => {
                let declarations = split_const_line(line);

                for declaration in declarations.iter() {
                    process_const_declaration(&declaration, table, line_number, stack_id);
                }
            }
            _ if class_names.contains(&first) => {
                // class declaration
                let rest = words.collect::<String>();
                let names = split_const_line(&rest);

                for name in names {
                    process_var_declaration(&name, table, line_number, stack_id)
                }
            }
            _ => {
                // ignore other lines
            }
        }
    }
}

pub fn process_const_declaration(
    line: &str,
    table: &mut SymbolTable,
    line_number: usize,
    stack_id: u32,
) {
    let mut tokens = line.split('=');

    let Some(name) = tokens.next() else { return };
    let name = name.trim();
    let name_lower = name.to_ascii_lowercase();
    if let Some(symbol) = table.symbols.get(&name_lower) {
        if symbol.stack_id == stack_id {
            log::debug!(
                "Found duplicate const declaration {} in line {}",
                name,
                line_number + 1
            );
            return;
        }
    }

    let Some(value) = tokens.next() else { return };
    let value = value.trim();
    let Some(_type) = get_type(value).or_else(|| table.symbols.get(value).map(|x| x._type)) else { return };

    log::debug!(
        "Found const declaration {} in line {}",
        name,
        line_number + 1
    );
    table.symbols.insert(
        name_lower,
        SymbolInfoMap {
            line_number: line_number as u32,
            _type,
            stack_id,
            end_line_number: 0,
            value: Some(String::from(value)),
            name_no_format: name.to_string(),
        },
    );
}

pub fn process_var_declaration(
    line: &str,
    table: &mut SymbolTable,
    line_number: usize,
    stack_id: u32,
) {
    let mut tokens = line.split('=');

    let Some(mut name) = tokens.next() else { return };

    if let Some(pos) = name.find('[') {
        name = &name[..pos];
    }

    let name = name.trim();
    let name_lower = name.to_ascii_lowercase();
    if let Some(symbol) = table.symbols.get(&name_lower) {
        if symbol.stack_id == stack_id {
            log::debug!(
                "Found duplicate var declaration {} in line {}",
                name,
                line_number + 1
            );
            return;
        }
    }
    // todo: try_insert
    // todo: value should be vector of SymbolInfoMap for each possible scope (functions may declare the same variable name)
    table.symbols.insert(
        name_lower,
        SymbolInfoMap {
            line_number: line_number as u32,
            _type: SymbolType::Var,
            stack_id,
            end_line_number: 0,
            value: None,
            name_no_format: name.to_string(),
        },
    );
}

pub fn split_const_line(line: &str) -> Vec<String> {
    // iterate over chars, split by , ignore commas inside parentheses
    let mut result = vec![];
    let mut current: usize = 0;
    let mut inside_parentheses = 0;

    for (i, c) in line.chars().enumerate() {
        match c {
            '(' => {
                inside_parentheses += 1;
            }
            ')' => {
                inside_parentheses -= 1;
            }
            ',' => {
                if inside_parentheses == 0 {
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
        let p = resolve_path(String::from("2.txt"), &Some(String::from("C:/dev/1.txt"))).unwrap();
        assert_eq!(p, String::from("C:/dev\\2.txt"));
    }
}
