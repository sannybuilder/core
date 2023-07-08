use crate::{legacy_ini::OpcodeTable, namespaces::namespaces::Namespaces};

pub mod ffi;
pub mod helpers;
pub mod transform;

pub fn transform(expr: &str, ns: &Namespaces, legacy_ini: &OpcodeTable) -> Option<String> {
    let body = crate::parser::parse(expr).ok()?.1;
    transform::try_tranform(&body, expr, ns, legacy_ini)
}

#[cfg(test)]
mod tests {
    use crate::legacy_ini::Game;

    use super::*;
    #[test]
    fn test_command_unary() {
        let mut table = OpcodeTable::new(Game::SA);
        table.load_from_file("SASCM.ini");
        let mut ns = Namespaces::new();
        ns.load_library("sa.json");

        assert_eq!(
            transform("~0@", &ns, &table),
            Some(String::from("0B1A: 0@"))
        );
        assert_eq!(
            transform("~$var", &ns, &table),
            Some(String::from("0B1A: $var"))
        );
        assert_eq!(
            transform("~&10", &ns, &table),
            Some(String::from("0B1A: &10"))
        );
        assert_eq!(
            transform("~10@($_,1i)", &ns, &table),
            Some(String::from("0B1A: 10@($_,1i)"))
        );
        assert_eq!(
            transform("~$0101(1000@,12f)", &ns, &table),
            Some(String::from("0B1A: $0101(1000@,12f)"))
        );
    }

    #[test]
    fn test_command_binary() {
        let mut table = OpcodeTable::new(Game::SA);
        table.load_from_file("SASCM.ini");
        let mut ns = Namespaces::new();
        ns.load_library("sa.json");

        assert_eq!(
            transform("0@ &= 1@", &ns, &table),
            Some(String::from("0B17: 0@ 1@"))
        );
        assert_eq!(
            transform("0@ &= 100", &ns, &table),
            Some(String::from("0B17: 0@ 100"))
        );
        assert_eq!(
            transform("0@ &= 42.01", &ns, &table),
            Some(String::from("0B17: 0@ 42.01"))
        );
        assert_eq!(
            transform("0@ &= -1", &ns, &table),
            Some(String::from("0B17: 0@ -1"))
        );
        assert_eq!(
            transform("0@ |= 1@", &ns, &table),
            Some(String::from("0B18: 0@ 1@"))
        );
        assert_eq!(
            transform("0@ ^= 1@", &ns, &table),
            Some(String::from("0B19: 0@ 1@"))
        );
        assert_eq!(
            transform("0@ %= 1@", &ns, &table),
            Some(String::from("0B1B: 0@ 1@"))
        );
        assert_eq!(
            transform("0@ >>= 1@", &ns, &table),
            Some(String::from("0B1C: 0@ 1@"))
        );
        assert_eq!(
            transform("0@ <<= 1@", &ns, &table),
            Some(String::from("0B1D: 0@ 1@"))
        );
        assert_eq!(
            transform("&101 <<= &123", &ns, &table),
            Some(String::from("0B1D: &101 &123"))
        );
        assert_eq!(
            transform("$var = 5", &ns, &table),
            Some(String::from("0004: $var 5"))
        );
        assert_eq!(
            transform("&100 = 5", &ns, &table),
            Some(String::from("0004: &100 5"))
        );
        assert_eq!(
            transform("0@ = 0", &ns, &table),
            Some(String::from("0006: 0@ 0"))
        );
        assert_eq!(
            transform("$var[10] = 5.0", &ns, &table),
            Some(String::from("0005: $var[10] 5.0"))
        );
        assert_eq!(
            transform("0@(1@,1i) = 0.0", &ns, &table),
            Some(String::from("0007: 0@(1@,1i) 0.0"))
        );
    }

    #[test]
    fn test_ternary() {
        let mut table = OpcodeTable::new(Game::SA);
        table.load_from_file("SASCM.ini");
        let mut ns = Namespaces::new();
        ns.load_library("sa.json");

        assert_eq!(
            transform("0@ = -1 & 1@", &ns, &table),
            Some(String::from("0B10: 0@ -1 1@"))
        );
        assert_eq!(
            transform("0@ = 1 | 1@", &ns, &table),
            Some(String::from("0B11: 0@ 1 1@"))
        );
        assert_eq!(
            transform("0@ = 1 ^ 1@", &ns, &table),
            Some(String::from("0B12: 0@ 1 1@"))
        );
        assert_eq!(
            transform("0@ = 1 % 1@", &ns, &table),
            Some(String::from("0B14: 0@ 1 1@"))
        );
        assert_eq!(
            transform("0@ = 1 >> 1@", &ns, &table),
            Some(String::from("0B15: 0@ 1 1@"))
        );
        assert_eq!(
            transform("0@ = 1 << 1@", &ns, &table),
            Some(String::from("0B16: 0@ 1 1@"))
        );
        assert_eq!(
            transform("0@ = 1 + 2", &ns, &table),
            Some(String::from("0A8E: 0@ 1 2"))
        );
        assert_eq!(
            transform("0@ = 1 - 2", &ns, &table),
            Some(String::from("0A8F: 0@ 1 2"))
        );
        assert_eq!(
            transform("0@ = 1 * 2", &ns, &table),
            Some(String::from("0A90: 0@ 1 2"))
        );
        assert_eq!(
            transform("0@ = 1 / 2", &ns, &table),
            Some(String::from("0A91: 0@ 1 2"))
        );
    }

    #[test]
    fn test_not() {
        let mut table = OpcodeTable::new(Game::SA);
        table.load_from_file("SASCM.ini");
        let mut ns = Namespaces::new();
        ns.load_library("sa.json");

        assert_eq!(transform("0@ = ~1@", &ns, &table), Some(String::from("0B13: 0@ 1@")));
        assert_eq!(transform("~0@", &ns, &table), Some(String::from("0B1A: 0@")));
    }
}
