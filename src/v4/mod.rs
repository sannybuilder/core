pub mod ffi;
pub mod game;
pub mod helpers;
pub mod transform;

use game::Game;

pub fn transform(expr: &str, game: Game) -> Option<String> {
    let body = crate::parser::parse(expr).ok()?.1;
    transform::try_tranform(&body, expr, game)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! t {
        ( for $game:expr => ($( $i:expr => $o:expr ),*) ) => {
            $(
                assert_eq!(transform($i, $game), Some(String::from($o)));
            )*

        };
    }

    macro_rules! tall {
        ($( $i:expr => $o:expr ),*) => {
            for game in [
                Game::gta3,
                Game::vc,
                Game::sa,
                Game::lcs,
                Game::vcs,
                Game::sa_mobile,
            ] {
                t! {
                    for game => (
                        $( $i => $o ),*
                    )
                }
            }
        };
    }

    #[test]
    fn test_unary_bitwise() {
        tall! {
            "~0@" => "BIT_NOT_COMPOUND 0@",
            "~$var" => "BIT_NOT_COMPOUND $var",
            "~&10" => "BIT_NOT_COMPOUND &10",
            "~10@($_,1i)" => "BIT_NOT_COMPOUND 10@($_,1i)",
            "~$0101(1000@,12f)" => "BIT_NOT_COMPOUND $0101(1000@,12f)"
        }
    }

    #[test]
    fn test_binary_bitwise_compound() {
        tall! {
            "0@ &= 1@" => "BIT_AND_COMPOUND 0@ 1@",
            "0@ &= 100" => "BIT_AND_COMPOUND 0@ 100",
            "0@ &= 42.01" => "BIT_AND_COMPOUND 0@ 42.01",
            "0@ &= -1" => "BIT_AND_COMPOUND 0@ -1",
            "0@ |= 1@" => "BIT_OR_COMPOUND 0@ 1@",
            "0@ ^= 1@" => "BIT_XOR_COMPOUND 0@ 1@",
            "0@ %= 1@" => "MOD_COMPOUND 0@ 1@",
            "0@ >>= 1@" => "BIT_SHR_COMPOUND 0@ 1@",
            "0@ <<= 1@" => "BIT_SHL_COMPOUND 0@ 1@",
            "&101 <<= &123" => "BIT_SHL_COMPOUND &101 &123"
        }
    }

    #[test]
    fn test_binary_assignment() {
        for game in [Game::gta3, Game::vc, Game::sa, Game::lcs, Game::sa_mobile] {
            t!(
                for game => (
                    "$var = 5" => "0004: $var 5",
                    "&100 = 5" => "0004: &100 5",
                    "$var[10] = 5" => "0004: $var[10] 5",
                    "0@ = 0" => "0006: 0@ 0",
                    "0@(1@,1i) = 0" => "0006: 0@(1@,1i) 0",
                    "$var = 5.0" => "0005: $var 5.0",
                    "&100 = 5.0" => "0005: &100 5.0",
                    "$var[10] = 5.0" => "0005: $var[10] 5.0",
                    "0@ = 0.0" => "0007: 0@ 0.0",
                    "0@(1@,1i) = 0.0" => "0007: 0@(1@,1i) 0.0",
                    "$89 += 1" => "0008: $89 1",
                    "$TEMPVAR_FLOAT_1 += 1.741" => "0009: $TEMPVAR_FLOAT_1 1.741",
                    "3@ += 3000" => "000A: 3@ 3000",
                    "6@ += 0.1" => "000B: 6@ 0.1"

                )
            );
        }

        t! {
            for Game::vcs => (
                "$var = 5" => "0004: $var 5",
                "&100 = 5" => "0004: &100 5",
                "0@ = 0" => "0004: 0@ 0",
                "$var[10] = 5" => "0004: $var[10] 5",
                "0@(1@,1i) = 0" => "0004: 0@(1@,1i) 0",
                "$var = 5.0" => "0005: $var 5.0",
                "&100 = 5.0" => "0005: &100 5.0",
                "0@ = 0.0" => "0005: 0@ 0.0",
                "$var[10] = 5.0" => "0005: $var[10] 5.0",
                "0@(1@,1i) = 0.0" => "0005: 0@(1@,1i) 0.0",
                "$2 = 1 /* int */" => "0004: $2 1",
                "$1440 = 100.0 // (float)" => "0005: $1440 100.0",
                // "$769 = 'CH_JERR' // (string)  // Jerry Martinez" => "0006: $769 'CH_JERR'",
                "3@ += 10 // (int)" => "0007: 3@ 10",
                "9@ += 0.5 // (float)" => "0008: 9@ 0.5"
            )
        };
    }

    #[test]
    fn test_ternary_bitwise() {
        tall! {
            "0@ = -1 & 1@" => "BIT_AND 0@ -1 1@",
            "0@ = 1 | 1@" => "BIT_OR 0@ 1 1@",
            "0@ = 1 ^ 1@" => "BIT_XOR 0@ 1 1@",
            "0@ = 1 % 1@" => "MOD 0@ 1 1@",
            "0@ = 1 >> 1@" => "BIT_SHR 0@ 1 1@",
            "0@ = 1 << 1@" => "BIT_SHL 0@ 1 1@"
        }
    }

    #[test]
    fn test_ternary_cleo_int() {
        tall! {
            "0@ = 1 + 2" => "INT_ADD 0@ 1 2",
            "0@ = 1 - 2" => "INT_SUB 0@ 1 2",
            "0@ = 1 * 2" => "INT_MUL 0@ 1 2",
            "0@ = 1 / 2" => "INT_DIV 0@ 1 2"
        }
    }

    #[test]
    fn test_not() {
        tall! {
            "0@ = ~1@" => "BIT_NOT 0@ 1@"
        }
    }

    // #[test]
    // fn test_string() {
    //     let t = Transformer::default();
    //     assert_eq!(
    //         t.transform("0@ = \"test\""),
    //         Some(String::from("SET_LVAR_STRING 0@ \"test\""))
    //     );
    // }
}
