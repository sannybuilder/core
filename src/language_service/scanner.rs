use super::ffi::{SymbolInfoMap, SymbolType};
use crate::dictionary::dictionary_num_by_str::DictNumByStr;
use std::fs;
use std::path::Path;

const TOKEN_INCLUDE: i32 = 103;
const TOKEN_CONST: i32 = 65;
const TOKEN_END: i32 = 255;
const TOKEN_INT: i32 = 1;
const TOKEN_FLOAT: i32 = 2;
const TOKEN_STRING: i32 = 3;
const TOKEN_LONGSTRING: i32 = 4;
const TOKEN_HANDLE: i32 = 5;
const TOKEN_BOOL: i32 = 6;

fn document_tree_walk<'a>(file_name: String, dict: &DictNumByStr) -> Option<Vec<String>> {
    let content = fs::read_to_string(&file_name).ok()?;
    let mut files = content
        .lines()
        .filter_map(|x| {
            // todo: use nom parser
            let mut words = x.split_ascii_whitespace();
            let first = words.next()?.to_ascii_lowercase();

            if let Some(token) = dict.map.get(&first) {
                if token == &TOKEN_INCLUDE {
                    let mut include_path = words.collect::<String>();

                    if include_path.ends_with('}') {
                        include_path.pop();
                    }

                    return Some(document_tree_walk(
                        resolve_path(include_path, &file_name)?,
                        dict,
                    )?);
                }
            }
            None
        })
        .flatten()
        .collect::<Vec<_>>();

    files.push(file_name);
    Some(files)
}

fn resolve_path(p: String, parent_file: &String) -> Option<String> {
    let path = Path::new(&p);

    if path.is_absolute() {
        return Some(p);
    }

    let dir_name = Path::new(&parent_file).parent()?;
    let abs_name = dir_name.join(path);

    return Some(String::from(abs_name.to_str()?));
}

pub fn document_tree<'a>(file_name: &'a str, dict: &DictNumByStr) -> Option<Vec<String>> {
    Some(document_tree_walk(file_name.to_string(), dict)?)
}

pub fn find_constants<'a>(
    file_name: String,
    dict: &DictNumByStr,
) -> Option<Vec<(String, SymbolInfoMap)>> {
    let content = fs::read_to_string(&file_name).ok()?;
    let mut lines = content.lines().enumerate();
    let mut res = vec![];
    let mut inside_const = false;
    while let Some((line_number, mut line)) = lines.next() {
        line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let mut words = line.split_ascii_whitespace();
        let first = words.next().unwrap_or("");
        match dict.map.get(first) {
            Some(token) => match *token {
                TOKEN_CONST => inside_const = true,
                TOKEN_END if inside_const => inside_const = false,
                TOKEN_INT | TOKEN_FLOAT | TOKEN_STRING | TOKEN_LONGSTRING | TOKEN_HANDLE
                | TOKEN_BOOL => {
                    // inline variable declaration
                    let name = words.next().unwrap_or("");
                    if !name.is_empty() {
                        res.push((
                            String::from(name),
                            SymbolInfoMap {
                                line_number: line_number as u32,
                                _type: SymbolType::Var,
                                file_name: file_name.clone(),
                            },
                        ))
                    }
                }
                _ => {}
            },
            _ if inside_const => {
                // todo: try nom parser
                let mut tokens = line.split('=');

                if let Some(mut name) = tokens.next() {
                    if let Some(mut value) = tokens.next() {
                        name = name.trim();
                        value = value.trim();
                        let mut _type = SymbolType::Number;
                        if value.starts_with('$') || value.ends_with('@') {
                            _type = SymbolType::Var
                        } else if value.starts_with('"') || value.starts_with('\'') {
                            _type = SymbolType::String
                        } else if let Some(_) = value.parse::<f32>().ok() {
                            _type = SymbolType::Number
                        } else {
                            continue;
                        }
                        res.push((
                            String::from(name),
                            SymbolInfoMap {
                                line_number: line_number as u32,
                                _type,
                                file_name: file_name.clone(),
                            },
                        ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let p = resolve_path(String::from("2.txt"), &String::from("C:/dev/1.txt")).unwrap();
        assert_eq!(p, String::from("C:/dev\\2.txt"));
    }
}
