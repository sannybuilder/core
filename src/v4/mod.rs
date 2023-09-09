use crate::{
    dictionary::{dictionary_num_by_str::DictNumByStr, dictionary_str_by_str::DictStrByStr},
    legacy_ini::OpcodeTable,
    namespaces::namespaces::Namespaces,
};

pub mod ffi;
pub mod helpers;
pub mod transform;

pub fn transform(
    expr: &str,
    ns: &Namespaces,
    legacy_ini: &OpcodeTable,
    var_types: &DictNumByStr,
    const_lookup: &DictStrByStr,
) -> Option<String> {
    let body = crate::parser::parse(expr).ok()?.1;
    transform::try_tranform(&body, expr, ns, legacy_ini, var_types, const_lookup)
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use crate::legacy_ini::Game;

    use super::*;
    #[test]
    fn test_command_unary() {
        let mut table = OpcodeTable::new(Game::SA);
        table.load_from_file("SASCM.ini");
        let mut ns = Namespaces::new();
        ns.load_library("sa.json");
        let dict = DictNumByStr::default();
        let mut const_lookup = DictStrByStr::default();
        const_lookup.add(CString::new("x").unwrap(), CString::new("3@").unwrap());

        let t = |input: &str| -> String {
            transform(input, &ns, &table, &dict, &const_lookup).unwrap_or_default()
        };

        assert_eq!(t("~0@"), "0B1A: 0@");
        assert_eq!(t("~$var"), "0B1A: $var");
        assert_eq!(t("~&10"), "0B1A: &10");
        assert_eq!(t("~10@($_,1i)"), "0B1A: 10@($_,1i)");
        assert_eq!(t("~$0101(1000@,12f)"), "0B1A: $0101(1000@,12f)");
        assert_eq!(t("~x"), "0B1A: 3@");
    }

    #[test]
    fn test_command_binary() {
        let mut table = OpcodeTable::new(Game::SA);
        table.load_from_file("SASCM.ini");
        let mut ns = Namespaces::new();
        ns.load_library("sa.json");
        let dict = DictNumByStr::default();
        let mut const_lookup = DictStrByStr::default();
        const_lookup.add(CString::new("x").unwrap(), CString::new("3@").unwrap());
        const_lookup.add(CString::new("y").unwrap(), CString::new("4@").unwrap());
        const_lookup.add(CString::new("n").unwrap(), CString::new("100").unwrap());
        const_lookup.add(CString::new("f").unwrap(), CString::new("100.0").unwrap());
        const_lookup.add(
            CString::new("fminus").unwrap(),
            CString::new("-100.0").unwrap(),
        );

        let t = |input: &str| -> String {
            transform(input, &ns, &table, &dict, &const_lookup).unwrap_or_default()
        };
        assert_eq!(t("0@ &= 1@"), "0B17: 0@ 1@");
        assert_eq!(t("0@ &= 100"), "0B17: 0@ 100");
        assert_eq!(t("0@ &= 42.01"), "0B17: 0@ 42.01");
        assert_eq!(t("0@ &= -1"), "0B17: 0@ -1");
        assert_eq!(t("0@ |= 1@"), "0B18: 0@ 1@");
        assert_eq!(t("0@ ^= 1@"), "0B19: 0@ 1@");
        assert_eq!(t("0@ %= 1@"), "0B1B: 0@ 1@");
        assert_eq!(t("0@ >>= 1@"), "0B1C: 0@ 1@");
        assert_eq!(t("0@ <<= 1@"), "0B1D: 0@ 1@");
        assert_eq!(t("&101 <<= &123"), "0B1D: &101 &123");

        assert_eq!(t("$var = 5"), "0004: $var 5");
        assert_eq!(t("$var = -5"), "0004: $var -5");
        assert_eq!(t("&100 = 5"), "0004: &100 5");
        assert_eq!(t("$var = -n"), "0004: $var -100");

        assert_eq!(t("$var = 5.0"), "0005: $var 5.0");
        assert_eq!(t("$var = -5.0"), "0005: $var -5.0");
        assert_eq!(t("&100 = 5.0"), "0005: &100 5.0");
        assert_eq!(t("$var = -f"), "0005: $var -100.0");

        assert_eq!(t("0@ = 0"), "0006: 0@ 0");
        assert_eq!(t("0@ = -1"), "0006: 0@ -1");
        assert_eq!(t("0@ = n"), "0006: 0@ 100");
        assert_eq!(t("0@ = -n"), "0006: 0@ -100");

        assert_eq!(t("0@ = 0.5"), "0007: 0@ 0.5");
        assert_eq!(t("0@ = -0.5"), "0007: 0@ -0.5");
        assert_eq!(t("0@ = f"), "0007: 0@ 100.0");
        assert_eq!(t("0@ = -f"), "0007: 0@ -100.0");
        assert_eq!(t("0@ = fminus"), "0007: 0@ -100.0");
        assert_eq!(t("0@ = -fminus"), "0007: 0@ 100.0");

        assert_eq!(t("$var[10] = 5.0"), "0005: $var[10] 5.0");
        assert_eq!(t("0@(1@,1i) = 0.0"), "0007: 0@(1@,1i) 0.0");
        assert_eq!(t("x &= y"), "0B17: 3@ 4@");
        assert_eq!(t("x &= -n"), "0B17: 3@ -100");
        assert_eq!(t("x |= y"), "0B18: 3@ 4@");
        assert_eq!(t("x ^= y"), "0B19: 3@ 4@");
        assert_eq!(t("x %= y"), "0B1B: 3@ 4@");
        assert_eq!(t("x >>= y"), "0B1C: 3@ 4@");
        assert_eq!(t("x <<= y"), "0B1D: 3@ 4@");
        assert_eq!(t("x = n"), "0006: 3@ 100");
        assert_eq!(t("x = 5"), "0006: 3@ 5");
        assert_eq!(t("x = 5.0"), "0007: 3@ 5.0");

        assert_ne!(t("0@ ^= ~1@"), "0B19: 0@ ~1@");
    }

    #[test]
    fn test_ternary() {
        let mut table = OpcodeTable::new(Game::SA);
        table.load_from_file("SASCM.ini");
        let mut ns = Namespaces::new();
        ns.load_library("sa.json");
        let dict = DictNumByStr::default();
        let mut const_lookup = DictStrByStr::default();
        const_lookup.add(CString::new("x").unwrap(), CString::new("3@").unwrap());
        const_lookup.add(CString::new("y").unwrap(), CString::new("4@").unwrap());
        const_lookup.add(CString::new("z").unwrap(), CString::new("5@").unwrap());

        let t = |input: &str| -> String {
            transform(input, &ns, &table, &dict, &const_lookup).unwrap_or_default()
        };
        assert_eq!(t("0@ = -1 & 1@"), "0B10: 0@ -1 1@");
        assert_eq!(t("0@ = 1 | 1@"), "0B11: 0@ 1 1@");
        assert_eq!(t("0@ = 1 ^ 1@"), "0B12: 0@ 1 1@");
        assert_eq!(t("0@ = 1 % 1@"), "0B14: 0@ 1 1@");
        assert_eq!(t("0@ = 1 >> 1@"), "0B15: 0@ 1 1@");
        assert_eq!(t("0@ = 1 << 1@"), "0B16: 0@ 1 1@");
        assert_eq!(t("0@ = 1 + 2"), "0A8E: 0@ 1 2");
        assert_eq!(t("0@ = 1 - 2"), "0A8F: 0@ 1 2");
        assert_eq!(t("0@ = -1 - -2"), "0A8F: 0@ -1 -2");
        assert_eq!(t("0@ = 1 * 2"), "0A90: 0@ 1 2");
        assert_eq!(t("0@ = 1 / 2"), "0A91: 0@ 1 2");
        assert_eq!(t("x = y & z"), "0B10: 3@ 4@ 5@");
        assert_eq!(t("x = y | z"), "0B11: 3@ 4@ 5@");
        assert_eq!(t("x = y ^ z"), "0B12: 3@ 4@ 5@");
        assert_eq!(t("x = y % z"), "0B14: 3@ 4@ 5@");
        assert_eq!(t("x = y >> z"), "0B15: 3@ 4@ 5@");
        assert_eq!(t("x = y << z"), "0B16: 3@ 4@ 5@");
        assert_eq!(t("x = y + z"), "0A8E: 3@ 4@ 5@");
        assert_eq!(t("x = y - z"), "0A8F: 3@ 4@ 5@");
        assert_eq!(t("x = y * z"), "0A90: 3@ 4@ 5@");
        assert_eq!(t("x = y / z"), "0A91: 3@ 4@ 5@");

        assert_ne!(t("0@ += 1 | 1@"), "0B11: 0@ 1 1@");
    }

    #[test]
    fn test_not() {
        let mut table = OpcodeTable::new(Game::SA);
        table.load_from_file("SASCM.ini");
        let mut ns = Namespaces::new();
        ns.load_library("sa.json");
        let dict = DictNumByStr::default();
        let mut const_lookup = DictStrByStr::default();
        const_lookup.add(CString::new("x").unwrap(), CString::new("3@").unwrap());

        let t = |input: &str| -> String {
            transform(input, &ns, &table, &dict, &const_lookup).unwrap_or_default()
        };
        assert_eq!(t("0@ = ~1@"), "0B13: 0@ 1@");
        assert_eq!(t("~0@"), "0B1A: 0@");
        assert_eq!(t("~x"), "0B1A: 3@");
    }

    #[test]
    fn test_timed_addition_assignment() {
        let mut table = OpcodeTable::new(Game::SA);
        table.load_from_file("SASCM.ini");
        let mut ns = Namespaces::new();
        ns.load_library("sa.json");
        let dict = DictNumByStr::default();
        let mut const_lookup = DictStrByStr::default();
        const_lookup.add(CString::new("x").unwrap(), CString::new("3@").unwrap());
        const_lookup.add(CString::new("y").unwrap(), CString::new("4@").unwrap());
        const_lookup.add(CString::new("n").unwrap(), CString::new("100").unwrap());
        const_lookup.add(CString::new("f").unwrap(), CString::new("100.0").unwrap());
        let t = |input: &str| -> String {
            transform(input, &ns, &table, &dict, &const_lookup).unwrap_or_default()
        };
        // +=@
        assert_eq!(t("$var +=@ 5.0"), "0078: $var 5.0");
        assert_eq!(t("$var +=@ f"), "0078: $var 100.0");
        assert_eq!(t("$var +=@ -f"), "0078: $var -100.0");
        assert_eq!(t("$var +=@ -10.23"), "0078: $var -10.23");
        assert_eq!(t("0@ +=@ 5.0"), "0079: 0@ 5.0");
        assert_eq!(t("$var1 +=@ $var2"), "007A: $var1 $var2");
        assert_eq!(t("0@ +=@ 1@"), "007B: 0@ 1@");
        assert_eq!(t("0@ +=@ $var"), "007C: 0@ $var");
        assert_eq!(t("$var +=@ 1@"), "007D: $var 1@");
        assert_eq!(t("x +=@ y"), "007B: 3@ 4@");

        //-=@
        assert_eq!(t("$var -=@ 5.0"), "007E: $var 5.0");
        assert_eq!(t("$var -=@ -5.0"), "007E: $var -5.0");
        assert_eq!(t("$var -=@ -f"), "007E: $var -100.0");
        assert_eq!(t("0@ -=@ 5.0"), "007F: 0@ 5.0");
        assert_eq!(t("$var1 -=@ $var2"), "0080: $var1 $var2");
        assert_eq!(t("0@ -=@ 1@"), "0081: 0@ 1@");
        assert_eq!(t("0@ -=@ $var"), "0082: 0@ $var");
        assert_eq!(t("$var -=@ 1@"), "0083: $var 1@");
        assert_eq!(t("x -=@ y"), "0081: 3@ 4@");
    }

    #[test]
    fn test_cast_assignment() {
        use crate::utils::compiler_const::{TOKEN_FLOAT, TOKEN_INT};

        let mut table = OpcodeTable::new(Game::SA);
        table.load_from_file("SASCM.ini");
        let mut ns = Namespaces::new();
        ns.load_library("sa.json");
        let mut dict = DictNumByStr::default();
        let mut const_lookup = DictStrByStr::default();
        const_lookup.add(CString::new("x").unwrap(), CString::new("3@").unwrap());
        const_lookup.add(CString::new("y").unwrap(), CString::new("4@").unwrap());

        dict.add("$i".to_string(), TOKEN_INT);
        dict.add("0@".to_string(), TOKEN_INT);
        dict.add("3@".to_string(), TOKEN_INT);
        dict.add("$f".to_string(), TOKEN_FLOAT);
        dict.add("1@".to_string(), TOKEN_FLOAT);
        dict.add("4@".to_string(), TOKEN_FLOAT);
        let t = |input: &str| -> String {
            transform(input, &ns, &table, &dict, &const_lookup).unwrap_or_default()
        };
        assert_eq!(t("$i =# $f"), "008C: $i $f");
        assert_eq!(t("$f =# $i"), "008D: $f $i");
        assert_eq!(t("0@ =# $f"), "008E: 0@ $f");
        assert_eq!(t("1@ =# $i"), "008F: 1@ $i");
        assert_eq!(t("$i =# 1@"), "0090: $i 1@");
        assert_eq!(t("$f =# 0@"), "0091: $f 0@");
        assert_eq!(t("0@ =# 1@"), "0092: 0@ 1@");
        assert_eq!(t("1@ =# 0@"), "0093: 1@ 0@");
        assert_eq!(t("x =# y"), "0092: 3@ 4@");
    }
}
