use super::ffi::Source;
use super::symbol_table::{SymbolInfoMap, SymbolTable, SymbolType};
use crate::dictionary::DictNumByString;
use crate::language_service::server::CACHE_FILE_SYMBOLS;
use crate::parser::FunctionSignature;
use crate::utils::compiler_const::*;
use crate::utils::visibility_zone::VisibilityZone;
use crate::v4::helpers::token_str;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn file_walk(
    file_name: &str,
    reserved_words: &DictNumByString,
    visited: &mut HashSet<String>,
    class_names: &Vec<String>,
    table: &mut SymbolTable,
    scope_stack: &mut Vec<(
        /*number of open blocks*/ u32,
        /* scope start line*/ u32,
    )>,
    line_number: Option<usize>,
) {
    // ignore cyclic paths
    if !visited.insert(file_name.into()) {
        return;
    }

    // if present, use file's cached symbols (all symbols from the file and all included files)
    if let Some(symbols) = CACHE_FILE_SYMBOLS.lock().unwrap().get(file_name) {
        log::debug!("Using cached symbols for file {}", file_name);
        table.extend(symbols);
        return;
    }

    log::debug!("Symbol cache not found. Reading file {}", file_name);
    let Ok(content) = fs::read_to_string(&file_name) else {
        return;
    };

    // create a new table for this file and its descendants
    let mut local_table = SymbolTable::new();
    scan_text(
        &content,
        reserved_words,
        class_names,
        &Source::File(file_name.into()),
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
        .insert(file_name.into(), local_table);
}

fn resolve_path(p: &str, parent_file: &Option<String>) -> Option<String> {
    let path = Path::new(p);

    if path.is_absolute() {
        return Some(p.to_string());
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
    text: &str,
    reserved_words: &DictNumByString,
    implicit_includes: &Vec<String>,
    source: &Source,
    class_names: &Vec<String>,
    table: &mut SymbolTable,
    visited: &mut HashSet<String>,
    scope_stack: &mut Vec<(u32, u32)>,
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

    scan_text(
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

pub fn scan_text<'a>(
    content: &str,
    reserved_words: &DictNumByString,
    class_names: &Vec<String>,
    source: &Source,
    visited: &mut HashSet<String>,
    scope_stack: &mut Vec<(u32, u32)>,
    table: &mut SymbolTable,
    line_number: Option<usize>,
) {
    let mut inside_const = false;
    let file_name = match source {
        Source::File(path) => Some(path.clone()),
        Source::Memory => None,
    };

    let mut inside_comment = false;
    let mut inside_comment2 = false;
    let mut next_annotation: Option<String> = None;

    let lines = content.lines();
    for (_index, line1) in lines.enumerate() {
        let (first, rest) = strip_comments(line1, &mut inside_comment, &mut inside_comment2);

        if first.is_empty() {
            continue;
        }

        if first.eq("///") {
            //append or create next_annotation

            if let Some(ref mut annotation) = next_annotation {
                annotation.push_str("\n");
                annotation.push_str(rest.as_str());
            } else {
                next_annotation = Some(rest.to_string());
            }

            continue;
        }

        let first_lower = first.to_ascii_lowercase();
        let token_id = reserved_words.map.get(&first_lower);

        // reset annotation if this line is not a function
        if token_id != Some(&TOKEN_FUNCTION) && token_id != Some(&TOKEN_DEFINE) {
            next_annotation = None;
        }

        /*
            if the file is a $include in the current document, then we need all its symbols (including deep $include's)
            to have a line number of the $include statement in the current document

            if the file is an implicit include (constants.txt), then all its symbols have a line number of 0
        */
        let line_number = line_number.unwrap_or(_index);
        let stack_id = scope_stack.len() as u32;

        let mut process_function_signature = |line: &str, signature: &FunctionSignature| {
            let scope_start_line = scope_stack.last().map(|(_, line)| *line).unwrap_or(0); // hoist the scope start line

            register_function(
                table,
                scope_start_line as usize,
                stack_id,
                line,
                signature,
                next_annotation.take(),
            );
            // only local functions create new scope. foreign functions do not
            if signature.cc == crate::parser::FunctionCC::Local {
                // push new scope
                scope_stack.push((0, line_number as u32));

                // end visibility zone for the local variables of the parent scope
                // because parent local variables can not be seen in functions
                // todo: make sure global vars is an exception
                for (_, symbols) in table.symbols.iter_mut() {
                    for symbol in symbols {
                        if symbol.stack_id == stack_id && symbol._type == SymbolType::Var {
                            let Some(last_zone) = symbol.zones.last_mut() else {
                                continue;
                            };

                            if last_zone.end != 0 {
                                // should not happen
                                log::error!(
                                    "Symbol {} does not have an open visibility zone",
                                    symbol.name_no_format
                                );
                                continue;
                            }

                            last_zone.end = line_number;
                        }
                    }
                }
                for param in &signature.parameters {
                    if let Some(ref name) = param.name {
                        register_var(
                            table,
                            line_number,
                            stack_id + 1, // register function parameters in the function's stack
                            token_str(line, name),
                            Some(token_str(line, &param._type).to_string()),
                            None,
                        );
                    }
                }
            }
        };

        match token_id {
            Some(token) => match *token {
                TOKEN_INCLUDE | TOKEN_INCLUDE_ONCE => {
                    let include_path = if rest.ends_with('}') {
                        &rest[..rest.len() - 1]
                    } else {
                        rest.as_str()
                    };

                    let Some(path) = resolve_path(include_path, &file_name) else {
                        continue;
                    };

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
                    if stack_id < 2 {
                        // global scope, only ends when the file ends
                        continue;
                    }
                    // number of nested blocks in the current function
                    let (fn_blocks, _) = scope_stack.last_mut().unwrap();
                    if *fn_blocks == 0 {
                        // there are no other open blocks in this function, this is the function's end

                        // find all symbols defined in the current scope and close their visibility zone
                        for (_, symbols) in table.symbols.iter_mut() {
                            for symbol in symbols {
                                if symbol.stack_id == stack_id {
                                    let Some(last_zone) = symbol.zones.last_mut() else {
                                        continue;
                                    };

                                    if last_zone.end != 0 {
                                        // should not happen
                                        log::error!(
                                            "Symbol {} does not have an open visibility zone",
                                            symbol.name_no_format
                                        );
                                        continue;
                                    }

                                    // this local variable is not visible inside the function
                                    last_zone.end = line_number;

                                    symbol.stack_id = 0; // mark as processed
                                }
                            }
                        }

                        // delete function scope
                        scope_stack.pop();

                        let stack_id = scope_stack.len() as u32;
                        // open visibility zone for the local variables of the parent scope
                        for (_, symbols) in table.symbols.iter_mut() {
                            for symbol in symbols {
                                if symbol.stack_id == stack_id && symbol._type == SymbolType::Var {
                                    symbol.add_zone(line_number)
                                }
                            }
                        }
                    } else {
                        // exit block
                        *fn_blocks -= 1;
                    }
                }
                TOKEN_INT | TOKEN_FLOAT | TOKEN_STRING | TOKEN_LONGSTRING | TOKEN_HANDLE
                | TOKEN_BOOL => {
                    // inline variable declaration
                    let names = split_const_line(&rest);

                    for name in names {
                        process_var_declaration(&name, table, line_number, stack_id, &first)
                    }
                }
                TOKEN_EXPORT => {
                    // export function <signature>
                    use crate::parser::{function_signature, Span};

                    // parse function signature and add its parameters to the symbol table as variables
                    let Ok((_, ref signature)) = function_signature(Span::from(rest.as_str()))
                    else {
                        continue;
                    };
                    process_function_signature(rest.as_str(), signature);
                }
                TOKEN_FUNCTION => {
                    // function <signature>
                    use crate::parser::{function_signature, Span};

                    // parse function signature and add its parameters to the symbol table as variables
                    let line = first + " " + &rest;
                    let line = line.as_str();
                    let Ok((_, ref signature)) = function_signature(Span::from(line)) else {
                        continue;
                    };

                    process_function_signature(line, signature);
                }

                TOKEN_IF | TOKEN_FOR | TOKEN_WHILE | TOKEN_SWITCH => {
                    if stack_id < 2 {
                        // global scope, only ends when the file ends
                        continue;
                    }
                    let (function_scope, _) = scope_stack.last_mut().unwrap();
                    // enter block
                    *function_scope += 1;
                }
                _ => {}
            },
            _ if inside_const => {
                let line = first + " " + &rest;
                let line = line.as_str();
                let declarations = split_const_line(line);

                for declaration in declarations.iter() {
                    process_const_declaration(&declaration, table, line_number, stack_id);
                }
            }
            _ if class_names.contains(&first_lower) => {
                // class declaration
                let names = split_const_line(&rest);

                for name in names {
                    process_var_declaration(&name, table, line_number, stack_id, &first)
                }
            }
            _ => {
                // ignore other lines
            }
        }
    }
}

fn register_symbol(table: &mut SymbolTable, map: SymbolInfoMap) {
    let name_lower = map.name_no_format.to_ascii_lowercase();
    match table.symbols.get_mut(&name_lower) {
        Some(symbols) => {
            for symbol in symbols.iter() {
                if symbol.stack_id == map.stack_id {
                    log::debug!(
                        "Found duplicate symbol declaration {} in line {}",
                        map.name_no_format,
                        // map.line_number + 1
                        map.zones[0].start + 1
                    );
                    return;
                }
            }

            symbols.push(map);
        }
        None => {
            table.symbols.insert(name_lower, vec![map]);
        }
    }
}

fn register_function(
    table: &mut SymbolTable,
    line_number: usize,
    stack_id: u32,
    line: &str,
    signature: &FunctionSignature,
    annotation: Option<String>,
) {
    let map = SymbolInfoMap {
        zones: vec![VisibilityZone {
            start: line_number,
            end: 0,
        }],
        // line_number: line_number as u32,
        _type: SymbolType::Function,
        stack_id, // register function in parent stack
        // end_line_number: 0,
        value: Some(function_params_and_return_types(line, signature)),
        name_no_format: token_str(&line, &signature.name).to_string(),
        annotation,
    };
    register_symbol(table, map);
}

fn register_var(
    table: &mut SymbolTable,
    line_number: usize,
    stack_id: u32,
    name: &str,
    _type: Option<String>,
    annotation: Option<String>,
) {
    register_const(
        table,
        line_number,
        stack_id,
        name,
        _type,
        SymbolType::Var,
        annotation,
    );
}

fn register_const(
    table: &mut SymbolTable,
    line_number: usize,
    stack_id: u32,
    name: &str,
    value: Option<String>,
    _type: SymbolType,
    annotation: Option<String>,
) {
    let map = SymbolInfoMap {
        zones: vec![VisibilityZone {
            start: line_number,
            end: 0,
        }],
        // line_number: line_number as u32,
        _type,
        stack_id,
        // end_line_number: 0,
        value,
        name_no_format: name.to_string(),
        annotation,
    };
    register_symbol(table, map);
}

pub fn strip_comments(
    s: &str,
    inside_comment: &mut bool,
    inside_comment2: &mut bool,
) -> (String, String) {
    let mut chars = s.chars().peekable();
    let mut first_word = String::new();
    let mut rest = String::new();
    let mut buf = &mut first_word;

    // iterate over all chars, skip comment fragments (/* */ and //)
    while let Some(c) = chars.next() {
        match c {
            _ if *inside_comment => {
                // skip until the end of the comment
                if c == '*' {
                    if let Some('/') = chars.next() {
                        *inside_comment = false;
                    }
                }
            }
            _ if *inside_comment2 => {
                // skip until the end of the comment
                if c == '}' {
                    *inside_comment2 = false;
                }
            }
            '/' if chars.peek() == Some(&'/') => {
                chars.next(); // skip /

                // annotation ///
                if chars.peek() == Some(&'/') {
                    chars.next(); // skip /

                    if buf.is_empty() {
                        // start of the line
                        buf.push_str("///"); // first word is ///
                        buf = &mut rest; // the rest of the line is the annotation
                        continue;
                    }
                }

                // line comment //
                // there is nothing left on this line, exiting
                break;
            }
            '/' if chars.peek() == Some(&'*') => {
                // block comment /* */
                *inside_comment = true;
                chars.next(); // skip *
            }

            '{' if chars.peek() != Some(&'$') => {
                // block comment {} but not directives {$...}
                *inside_comment2 = true;
            }
            _ if c.is_ascii_whitespace() => {
                if buf.is_empty() {
                    // skip leading whitespace
                    continue;
                } else {
                    buf = &mut rest;
                    if !buf.is_empty() {
                        buf.push(c);
                    }
                }
            }
            _ => {
                // line_without_comments.push(c);
                buf.push(c);
            }
        }
    }

    return (first_word, rest.trim_end().to_string());
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
    // let name_lower = name.to_ascii_lowercase();
    // if let Some(symbols) = table.symbols.get(&name_lower) {
    //     for symbol in symbols {
    //         if symbol.stack_id == stack_id {
    //             log::debug!(
    //                 "Found duplicate const declaration {} in line {}",
    //                 name,
    //                 line_number + 1
    //             );
    //             return;
    //         }
    //     }
    // }

    let Some(value) = tokens.next() else { return };
    let value = value.trim();
    let value_lower = value.to_ascii_lowercase();
    let Some(_type) = get_type(value_lower.as_str()).or_else(|| {
        table.symbols.get(value_lower.as_str()).and_then(|symbols| {
            symbols
                .iter()
                .find(|symbol| symbol.stack_id == stack_id)
                .map(|symbol| symbol._type)
        })
    }) else {
        return;
    };

    log::debug!(
        "Found const declaration {} in line {}",
        name,
        line_number + 1
    );

    register_const(
        table,
        line_number,
        stack_id,
        name,
        Some(String::from(value)),
        _type,
        None,
    );
}

pub fn process_var_declaration(
    line: &str,
    table: &mut SymbolTable,
    line_number: usize,
    stack_id: u32,
    _type: &str,
) {
    let mut tokens = line.split('=');

    let Some(mut name) = tokens.next() else {
        return;
    };

    if let Some(pos) = name.find('[') {
        name = &name[..pos];
    }

    let name = name.trim();
    // let name_lower = name.to_ascii_lowercase();
    // if let Some(symbols) = table.symbols.get(&name_lower) {
    //     for symbol in symbols {
    //         if symbol.stack_id == stack_id {
    //             log::debug!(
    //                 "Found duplicate var declaration {} in line {}",
    //                 name,
    //                 line_number + 1
    //             );
    //             return;
    //         }
    //     }
    // }

    register_var(
        table,
        line_number,
        stack_id,
        name,
        Some(String::from(_type)),
        None,
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

fn function_params_and_return_types(line: &str, signature: &FunctionSignature) -> String {
    let params = signature
        .parameters
        .iter()
        .map(|param| {
            let type_token = token_str(line, &param._type);
            let name_token = param.name.as_ref().map(|name| token_str(line, name));
            let size_token = param.size.as_ref().map(|size| token_str(line, size));

            let type_token = if let Some(size) = size_token {
                format!("{}[{}]", type_token, size)
            } else {
                type_token.to_string()
            };
            match name_token {
                Some(name) => format!("{}: {}", name, type_token),
                None => format!("{}", type_token),
            }
        })
        .collect::<Vec<String>>()
        .join(", ");

    let return_types = signature
        .return_types
        .iter()
        .map(|_type| token_str(line, &_type.token).to_string())
        .collect::<Vec<String>>()
        .join(", ");

    if return_types.is_empty() {
        format!("({params})")
    } else {
        format!("({params}): {return_types}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let p = resolve_path("2.txt", &Some(String::from("C:/dev/1.txt"))).unwrap();
        assert_eq!(p, String::from("C:/dev\\2.txt"));
    }

    #[test]
    fn test2() {
        let s = "test line";
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "line".to_string()));

        let s = "test line // comment";
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "line".to_string()));

        let s = "test line /* comment */";
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "line".to_string()));

        let s = "test line /* comment */ test line";
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "line  test line".to_string()));

        let s = "test line /* comment */ test line /* comment */";
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "line  test line".to_string()));

        let s = "test line";
        let mut inside_comment = true;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("".to_string(), "".to_string()));

        let s = "test line */ after comment";
        let mut inside_comment = true;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("after".to_string(), "comment".to_string()));

        let s = "    leading whitespace";
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("leading".to_string(), "whitespace".to_string()));

        let s = " {comment} test {comment} line";
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "line".to_string()));

        let s = " comment} test {comment} line {comment} ";
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("comment}".to_string(), "test  line".to_string()));

        let s = " comment} test {comment} line {comment} ";
        let mut inside_comment = false;
        let mut inside_comment2 = true;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "line".to_string()));

        let s = "test{ /*  */ } line";
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "line".to_string()));

        let s = "test/* {} */ line";
        let mut inside_comment = false;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "line".to_string()));

        let s = "*/test";
        let mut inside_comment = true;
        let mut inside_comment2 = false;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "".to_string()));

        let s = "}test";
        let mut inside_comment = false;
        let mut inside_comment2 = true;
        let s = strip_comments(s, &mut inside_comment, &mut inside_comment2);
        assert_eq!(s, ("test".to_string(), "".to_string()));
    }
}
