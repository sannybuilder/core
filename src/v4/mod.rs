pub mod ffi;
pub mod helpers;
pub mod transform;

pub fn transform(expr: &str) -> Option<String> {
    let body = crate::parser::parse(expr).ok()?.1;
    transform::try_tranform(&body, expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_command_unary() {
        assert_eq!(transform("~0@"), Some(String::from("BIT_NOT_COMPOUND 0@")));
        assert_eq!(
            transform("~$var"),
            Some(String::from("BIT_NOT_COMPOUND $var"))
        );
        assert_eq!(
            transform("~10@($_,1i)"),
            Some(String::from("BIT_NOT_COMPOUND 10@($_,1i)"))
        );
        assert_eq!(
            transform("~$0101(1000@,12f)"),
            Some(String::from("BIT_NOT_COMPOUND $0101(1000@,12f)"))
        );
    }

    #[test]
    fn test_command_binary() {
        assert_eq!(
            transform("0@ &= 1@"),
            Some(String::from("BIT_AND_COMPOUND 0@ 1@"))
        );
        assert_eq!(
            transform("0@ &= 100"),
            Some(String::from("BIT_AND_COMPOUND 0@ 100"))
        );
        assert_eq!(
            transform("0@ &= 42.01"),
            Some(String::from("BIT_AND_COMPOUND 0@ 42.01"))
        );
        assert_eq!(
            transform("0@ &= -1"),
            Some(String::from("BIT_AND_COMPOUND 0@ -1"))
        );
        assert_eq!(
            transform("0@ |= 1@"),
            Some(String::from("BIT_OR_COMPOUND 0@ 1@"))
        );
        assert_eq!(
            transform("0@ ^= 1@"),
            Some(String::from("BIT_XOR_COMPOUND 0@ 1@"))
        );
        assert_eq!(
            transform("0@ %= 1@"),
            Some(String::from("MOD_COMPOUND 0@ 1@"))
        );
        assert_eq!(
            transform("0@ >>= 1@"),
            Some(String::from("BIT_SHR_COMPOUND 0@ 1@"))
        );
        assert_eq!(
            transform("0@ <<= 1@"),
            Some(String::from("BIT_SHL_COMPOUND 0@ 1@"))
        );
        assert_eq!(
            transform("$var = 5"),
            Some(String::from("SET_VAR_INT $var 5"))
        );
        assert_eq!(transform("0@ = 0"), Some(String::from("SET_LVAR_INT 0@ 0")));
        assert_eq!(
            transform("$var[10] = 5.0"),
            Some(String::from("SET_VAR_FLOAT $var[10] 5.0"))
        );
        assert_eq!(
            transform("0@(1@,1i) = 0.0"),
            Some(String::from("SET_LVAR_FLOAT 0@(1@,1i) 0.0"))
        );
    }

    #[test]
    fn test_ternary() {
        assert_eq!(
            transform("0@ = -1 & 1@"),
            Some(String::from("BIT_AND 0@ -1 1@"))
        );
        assert_eq!(
            transform("0@ = 1 | 1@"),
            Some(String::from("BIT_OR 0@ 1 1@"))
        );
        assert_eq!(
            transform("0@ = 1 ^ 1@"),
            Some(String::from("BIT_XOR 0@ 1 1@"))
        );
        assert_eq!(transform("0@ = 1 % 1@"), Some(String::from("MOD 0@ 1 1@")));
        assert_eq!(
            transform("0@ = 1 >> 1@"),
            Some(String::from("BIT_SHR 0@ 1 1@"))
        );
        assert_eq!(
            transform("0@ = 1 << 1@"),
            Some(String::from("BIT_SHL 0@ 1 1@"))
        );
        assert_eq!(
            transform("0@ = 1 + 2"),
            Some(String::from("INT_ADD 0@ 1 2"))
        );
        assert_eq!(
            transform("0@ = 1 - 2"),
            Some(String::from("INT_SUB 0@ 1 2"))
        );
        assert_eq!(
            transform("0@ = 1 * 2"),
            Some(String::from("INT_MUL 0@ 1 2"))
        );
        assert_eq!(
            transform("0@ = 1 / 2"),
            Some(String::from("INT_DIV 0@ 1 2"))
        );
    }

    #[test]
    fn test_not() {
        assert_eq!(transform("0@ = ~1@"), Some(String::from("BIT_NOT 0@ 1@")));
        assert_eq!(transform("~0@"), Some(String::from("BIT_NOT_COMPOUND 0@")));
    }
}
