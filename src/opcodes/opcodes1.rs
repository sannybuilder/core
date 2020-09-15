extern crate combine;
use combine::many;
use combine::parser::char::*;
use combine::parser::range::take;
use combine::*;
use parser::repeat::take_until;

use combine::error::ParseError;
use combine::parser::range::{range, take_while1};
use combine::parser::repeat::sep_by;
use combine::parser::Parser;
use combine::stream::RangeStream;
/*

load from file
parse a line

date, publisher, name (file_name)

current opcode setter
current opcode num params
current opcode Nth param pos
current opcode Nth param type
current opcode Nth word
current opcode has Nth word?

Nth opcode num params


Two Types of Input lines:

a - DATE|PUBLISHER=string
b-  char[4]=i8,string


char[4] represents opcode value in hexadecimal notation
i8 is a number from range (-1, ~20)
string is a line in format:

[[string ]%NT%[ string]]*

where N is a number in range 1 - i8
T is a character ('d','g','z','p','o','t','x','s','m')

should strip `b:string` from T

New features:

 - custom types:
    T can be a string mapped onto enum name
*/

struct OpcodeDefinition {
    id: u16,
    num_params: i8,
    words: Vec<String>,
    params: Vec<Param>,
}

struct Param {
    r#type: u8,
    position: u8,
}

struct Opcodes {
    definitions: std::collections::HashMap<u16, OpcodeDefinition>,
    date: String,
    publisher: String,
    currentOpcode: u16,
}

impl Opcodes {
    fn new() -> Self {
        Opcodes {
            definitions: std::collections::HashMap::new(),
            date: "".to_string(),
            publisher: "".to_string(),
            currentOpcode: 0,
        }
    }

    fn add(&mut self, def: OpcodeDefinition) -> () {
        self.definitions.insert(def.id, def);
    }

    fn has_word(&self, index: u8) -> bool {
        if let Some(def) = self.definitions.get(&self.currentOpcode) {
            return def.words.len() > index.into();
        }
        return false;
    }
}

fn parse(line: String) -> Option<OpcodeDefinition> {
    unimplemented!()
}

fn parse_file(file_name: String) -> Option<Opcodes> {
    unimplemented!()
}

// `parse` returns `Result` where `Ok` contains a tuple of the parsers output and any remaining input.

#[cfg(test)]
mod tests {

    use super::*;
    use parser::repeat::repeat_until;
    use std::collections::HashMap;

    #[test]
    fn test_1() {
        #[derive(Default, Debug)]
        struct Tools<'a>(HashMap<&'a str, u32>);

        impl<'a> std::iter::Extend<&'a str> for Tools<'a> {
            fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
                for tool in iter.into_iter() {
                    let counter = self.0.entry(tool).or_insert(0);
                    *counter += 1;
                }
            }
        }

        let input = "Hammer, Saw, Drill, Hammer";

        let tool = take_while1(|c: char| c.is_alphabetic());
        let mut tools = sep_by(tool, range(", ")).map(|m| m);

        let output = tools.easy_parse(input).unwrap().0;
        // Tools({"Saw": 1, "Hammer": 2, "Drill": 1})
        // Construct a parser that parses *many* (and at least *1) *letter*s
        // let word = many1(letter());

        // let param = between(char('%'), char('%'), digit());

        // let collection = many(hex_digit());

        // let op = take_until(char('='));
        // let p = (spaces(), op);
        // let opcode = (
        //     // skip_many(spaces()),
        //     spaces(),
        //     take_until(char('='), hex_digit()).map(|_| 1),
        //     // hex_digit(),
        //     // hex_digit(),
        //     // hex_digit(),
        //     spaces(),
        //     // skip_many(spaces()),
        //     char('='),
        //     param, // skip_many(spaces()),
        // );

        // Construct a parser that parses many *word*s where each word is *separated by* a (white)*space*
        // let mut parser = repeat_until(param, newline()); //sep_by(word, space()).map(|mut words: Vec<String>| words.pop());
        // let result = collection.parse("0051");

        // assert_eq!(result, Ok((((), "0051"), "")));
        // assert_eq!(result, Ok((((), 1, '0', '5', '1', (), '=', '1'), "")));

        // fn decode(input: &str) -> Result<Vec<&str>, String> {
        //     let tool = take_while1(|c: char| c.is_ascii_hexdigit());
        //     let mut p = sep_by(tool, char('='));

        //     match p.easy_parse(input) {
        //         Ok((output, _remaining_input)) => Ok(output),
        //         Err(err) => Err(format!("{} in `{}`", err, input)),
        //     }
        // }

        // let op = count_min_max(1, 4, hex_digit());

        // let input = "0000=";
        // let output = decode(input).unwrap();
        println!(format!("{:#?}", output));
        assert_eq!(1, 1)
    }
}
