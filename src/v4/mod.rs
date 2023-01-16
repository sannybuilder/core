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
            // var = literal
            t!(
                for game => (
                    "$var = -5" => "0004: $var -5",
                    "&100 = 5" => "0004: &100 5",
                    "$var[10] = 5" => "0004: $var[10] 5",
                    "0@ = 0" => "0006: 0@ 0",
                    "0@(1@,1i) = 0" => "0006: 0@(1@,1i) 0",
                    "$var = 5.0" => "0005: $var 5.0",
                    "&100 = 5.0" => "0005: &100 5.0",
                    "$var[10] = 5.0" => "0005: $var[10] 5.0",
                    "0@ = 0.0" => "0007: 0@ 0.0",
                    "0@(1@,1i) = 0.0" => "0007: 0@(1@,1i) 0.0"
                )
            );

            // var = var
            t!(
                for game => (
                    // [global var: int] = [global var: int]
                    "$2($3,1i) = $4($5,1i)" => "0084: $2($3,1i) $4($5,1i)",
                    // [local var: int] = [local var: int]
                    "0@(1@,1i) = 0@(1@,1i)" => "0085: 0@(1@,1i) 0@(1@,1i)",
                    // [global var: float] = [global var: float]
                    "$2($3,1f) = $4($5,1f)" => "0086: $2($3,1f) $4($5,1f)",
                    // [local var: float] = [local var: float]
                    "0@(1@,1f) = 0@(1@,1f)" => "0087: 0@(1@,1f) 0@(1@,1f)",
                    // [global var: float] = [local var: float]
                    "$2($3,1f) = 0@(1@,1f)" => "0088: $2($3,1f) 0@(1@,1f)",
                    // [local var: float] = [global var: float]
                    "0@(1@,1f) = $2($3,1f)" => "0089: 0@(1@,1f) $2($3,1f)",
                    // [global var: int] = [local var: int]
                    "$2($3,1i) = 0@(1@,1i)" => "008A: $2($3,1i) 0@(1@,1i)",
                    // [local var: int] = [global var: int]
                    "0@(1@,1i) = $2($3,1i)" => "008B: 0@(1@,1i) $2($3,1i)"

                )
            );
        }

        // var = literal
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
                "$1440 = 100.0 // (float)" => "0005: $1440 100.0"
            )
        };

        // var = var
        t! {
            for Game::vcs => (
                "$2($3,1i) = $4($5,1i)" => "0035: $2($3,1i) $4($5,1i)",
                "0@(1@,1i) = 0@(1@,1i)" => "0035: 0@(1@,1i) 0@(1@,1i)",
                "$2($3,1f) = $4($5,1f)" => "0036: $2($3,1f) $4($5,1f)",
                "0@(1@,1f) = 0@(1@,1f)" => "0036: 0@(1@,1f) 0@(1@,1f)",
                "$2($3,1f) = 0@(1@,1f)" => "0036: $2($3,1f) 0@(1@,1f)",
                "0@(1@,1f) = $2($3,1f)" => "0036: 0@(1@,1f) $2($3,1f)",
                "$2($3,1i) = 0@(1@,1i)" => "0035: $2($3,1i) 0@(1@,1i)",
                "0@(1@,1i) = $2($3,1i)" => "0035: 0@(1@,1i) $2($3,1i)"
            )
        }
    }

    #[test]
    fn test_binary_add_compound() {
        for game in [Game::gta3, Game::vc, Game::sa, Game::lcs, Game::sa_mobile] {
            // var += literal
            t!(
                for game => (
                    // [global var: int] += [literal: int]
                    "$89 += 1" => "0008: $89 1",
                    // [global var: float] += [literal: float]
                    "$TEMPVAR_FLOAT_1 += 1.741" => "0009: $TEMPVAR_FLOAT_1 1.741",
                    // [local var: int] += [literal: int]
                    "0@ += 1" => "000A: 0@ 1",
                    // [local var: float] += [literal: float]
                    "0@ += 1.741" => "000B: 0@ 1.741"
                )
            );

            // var += var
            t!(
                for game => (
                    // [global var: int] += [global var: int]
                    "$2($3,1i) += $2($3,1i)" => "0058: $2($3,1i) $2($3,1i)",
                    // [global var: float] += [global var: float]
                    "$2($3,1f) += $2($3,1f)" => "0059: $2($3,1f) $2($3,1f)",
                    // [local var: int] += [local var: int]
                    "0@(1@,1i) += 0@(1@,1i)" => "005A: 0@(1@,1i) 0@(1@,1i)",
                    // [local var: float] += [local var: float]
                    "0@(1@,1f) += 0@(1@,1f)" => "005B: 0@(1@,1f) 0@(1@,1f)",
                    // [local var: int] += [global var: int]
                    "0@(1@,1i) += $2($3,1i)" => "005C: 0@(1@,1i) $2($3,1i)",
                    // [local var: float] += [global var: float]
                    "0@(1@,1f) += $2($3,1f)" => "005D: 0@(1@,1f) $2($3,1f)",
                    // [global var: int] += [local var: int]
                    "$2($3,1i) += 0@(1@,1i)" => "005E: $2($3,1i) 0@(1@,1i)",
                    // [global var: float] += [local var: float]
                    "$2($3,1f) += 0@(1@,1f)" => "005F: $2($3,1f) 0@(1@,1f)"
                )
            );
        }

        // var += literal
        t! {
            for Game::vcs => (
                "$2 += 1" => "0007: $2 1",
                "0@ += 0" => "0007: 0@ 0",
                "$2($3,1i) += 1" => "0007: $2($3,1i) 1",
                "0@(1@,1i) += 0" => "0007: 0@(1@,1i) 0",
                "$2 += 1.0" => "0008: $2 1.0",
                "0@ += 0.0" => "0008: 0@ 0.0",
                "$2($3,1f) += 1.0" => "0008: $2($3,1f) 1.0",
                "0@(1@,1f) += 0.0" => "0008: 0@(1@,1f) 0.0"
            )
        }

        t! {
            for Game::vcs => (
                // var += var
                "$2($3,1i) += $2($3,1i)" => "0029: $2($3,1i) $2($3,1i)",
                "0@(1@,1i) += 0@(1@,1i)" => "0029: 0@(1@,1i) 0@(1@,1i)",
                "$2($3,1f) += $2($3,1f)" => "002A: $2($3,1f) $2($3,1f)",
                "0@(1@,1f) += 0@(1@,1f)" => "002A: 0@(1@,1f) 0@(1@,1f)",
                "0@(1@,1i) += $2($3,1i)" => "0029: 0@(1@,1i) $2($3,1i)",
                "0@(1@,1f) += $2($3,1f)" => "002A: 0@(1@,1f) $2($3,1f)",
                "$2($3,1i) += 0@(1@,1i)" => "0029: $2($3,1i) 0@(1@,1i)",
                "$2($3,1f) += 0@(1@,1f)" => "002A: $2($3,1f) 0@(1@,1f)"
            )
        }
    }

    #[test]
    fn test_binary_sub_compound() {
        for game in [Game::gta3, Game::vc, Game::sa, Game::lcs, Game::sa_mobile] {
            // var -= literal
            t!(
                for game => (
                    // [global var: int] -= [literal: int]
                    "$89 -= 1" => "000C: $89 1",
                    // [global var: float] -= [literal: float]
                    "$TEMPVAR_FLOAT_1 -= 1.741" => "000D: $TEMPVAR_FLOAT_1 1.741",
                    // [local var: int] -= [literal: int]
                    "0@ -= 1" => "000E: 0@ 1",
                    // [local var: float] -= [literal: float]
                    "0@ -= 1.741" => "000F: 0@ 1.741"
                )
            );

            // var -= var
            t!(
                for game => (
                    // [global var: int] -= [global var: int]
                    "$2($3,1i) -= $2($3,1i)" => "0060: $2($3,1i) $2($3,1i)",
                    // [global var: float] -= [global var: float]
                    "$2($3,1f) -= $2($3,1f)" => "0061: $2($3,1f) $2($3,1f)",
                    // [local var: int] -= [local var: int]
                    "0@(1@,1i) -= 0@(1@,1i)" => "0062: 0@(1@,1i) 0@(1@,1i)",
                    // [local var: float] -= [local var: float]
                    "0@(1@,1f) -= 0@(1@,1f)" => "0063: 0@(1@,1f) 0@(1@,1f)",
                    // [local var: int] -= [global var: int]
                    "0@(1@,1i) -= $2($3,1i)" => "0064: 0@(1@,1i) $2($3,1i)",
                    // [local var: float] -= [global var: float]
                    "0@(1@,1f) -= $2($3,1f)" => "0065: 0@(1@,1f) $2($3,1f)",
                    // [global var: int] -= [local var: int]
                    "$2($3,1i) -= 0@(1@,1i)" => "0066: $2($3,1i) 0@(1@,1i)",
                    // [global var: float] -= [local var: float]
                    "$2($3,1f) -= 0@(1@,1f)" => "0067: $2($3,1f) 0@(1@,1f)"
                )

            );
        }

        // var -= literal
        t! {
            for Game::vcs => (
                "$89 -= 1" => "0009: $89 1",
                "$TEMPVAR_FLOAT_1 -= 1.741" => "000A: $TEMPVAR_FLOAT_1 1.741",
                "0@ -= 1" => "0009: 0@ 1",
                "0@ -= 1.741" => "000A: 0@ 1.741"
            )
        }

        // var -= var
        t! {
            for Game::vcs => (
                "$2($3,1i) -= $2($3,1i)" => "002B: $2($3,1i) $2($3,1i)",
                "0@(1@,1i) -= 0@(1@,1i)" => "002B: 0@(1@,1i) 0@(1@,1i)",
                "$2($3,1f) -= $2($3,1f)" => "002C: $2($3,1f) $2($3,1f)",
                "0@(1@,1f) -= 0@(1@,1f)" => "002C: 0@(1@,1f) 0@(1@,1f)",
                "0@(1@,1i) -= $2($3,1i)" => "002B: 0@(1@,1i) $2($3,1i)",
                "0@(1@,1f) -= $2($3,1f)" => "002C: 0@(1@,1f) $2($3,1f)",
                "$2($3,1i) -= 0@(1@,1i)" => "002B: $2($3,1i) 0@(1@,1i)",
                "$2($3,1f) -= 0@(1@,1f)" => "002C: $2($3,1f) 0@(1@,1f)"
            )
        }
    }

    #[test]
    fn test_binary_mul_compound() {
        for game in [Game::gta3, Game::vc, Game::sa, Game::lcs, Game::sa_mobile] {
            // var *= literal
            t!(
                for game => (
                    // [global var: int] *= [literal: int]
                    "$89 *= 1" => "0010: $89 1",
                    // [global var: float] *= [literal: float]
                    "$TEMPVAR_FLOAT_1 *= 1.741" => "0011: $TEMPVAR_FLOAT_1 1.741",
                    // [local var: int] *= [literal: int]
                    "0@ *= 1" => "0012: 0@ 1",
                    // [local var: float] *= [literal: float]
                    "0@ *= 1.741" => "0013: 0@ 1.741"
                )
            );

            // var *= var
            t!(
                for game => (
                    // [global var: int] *= [global var: int]
                    "$2($3,1i) *= $2($3,1i)" => "0068: $2($3,1i) $2($3,1i)",
                    // [global var: float] *= [global var: float]
                    "$2($3,1f) *= $2($3,1f)" => "0069: $2($3,1f) $2($3,1f)",
                    // [local var: int] *= [local var: int]
                    "0@(1@,1i) *= 0@(1@,1i)" => "006A: 0@(1@,1i) 0@(1@,1i)",
                    // [local var: float] *= [local var: float]
                    "0@(1@,1f) *= 0@(1@,1f)" => "006B: 0@(1@,1f) 0@(1@,1f)",
                    // [local var: int] *= [global var: int]
                    "0@(1@,1i) *= $2($3,1i)" => "006C: 0@(1@,1i) $2($3,1i)",
                    // [local var: float] *= [global var: float]
                    "0@(1@,1f) *= $2($3,1f)" => "006D: 0@(1@,1f) $2($3,1f)",
                    // [global var: int] *= [local var: int]
                    "$2($3,1i) *= 0@(1@,1i)" => "006E: $2($3,1i) 0@(1@,1i)",
                    // [global var: float] *= [local var: float]
                    "$2($3,1f) *= 0@(1@,1f)" => "006F: $2($3,1f) 0@(1@,1f)"
                )

            );
        }

        // var *= literal
        t! {
            for Game::vcs => (
                "$89 *= 1" => "000B: $89 1",
                "$TEMPVAR_FLOAT_1 *= 1.741" => "000C: $TEMPVAR_FLOAT_1 1.741",
                "0@ *= 1" => "000B: 0@ 1",
                "0@ *= 1.741" => "000C: 0@ 1.741"
            )
        }

        // var *= var
        t! {
            for Game::vcs => (
                "$2($3,1i) *= $2($3,1i)" => "002D: $2($3,1i) $2($3,1i)",
                "$2($3,1f) *= $2($3,1f)" => "002E: $2($3,1f) $2($3,1f)",
                "0@(1@,1i) *= 0@(1@,1i)" => "002D: 0@(1@,1i) 0@(1@,1i)",
                "0@(1@,1f) *= 0@(1@,1f)" => "002E: 0@(1@,1f) 0@(1@,1f)",
                "0@(1@,1i) *= $2($3,1i)" => "002D: 0@(1@,1i) $2($3,1i)",
                "0@(1@,1f) *= $2($3,1f)" => "002E: 0@(1@,1f) $2($3,1f)",
                "$2($3,1i) *= 0@(1@,1i)" => "002D: $2($3,1i) 0@(1@,1i)",
                "$2($3,1f) *= 0@(1@,1f)" => "002E: $2($3,1f) 0@(1@,1f)"
            )
        }
    }

    #[test]
    fn test_binary_div_compound() {
        for game in [Game::gta3, Game::vc, Game::sa, Game::lcs, Game::sa_mobile] {
            // var /= literal
            t!(
                for game => (
                    // [global var: int] /= [literal: int]
                    "$89 /= 1" => "0014: $89 1",
                    // [global var: float] /= [literal: float]
                    "$TEMPVAR_FLOAT_1 /= 1.741" => "0015: $TEMPVAR_FLOAT_1 1.741",
                    // [local var: int] /= [literal: int]
                    "0@ /= 1" => "0016: 0@ 1",
                    // [local var: float] /= [literal: float]
                    "0@ /= 1.741" => "0017: 0@ 1.741"
                )
            );

            // var /= var
            t!(
                for game => (
                    // [global var: int] /= [global var: int]
                    "$2($3,1i) /= $2($3,1i)" => "0070: $2($3,1i) $2($3,1i)",
                    // [global var: float] /= [global var: float]
                    "$2($3,1f) /= $2($3,1f)" => "0071: $2($3,1f) $2($3,1f)",
                    // [local var: int] /= [local var: int]
                    "0@(1@,1i) /= 0@(1@,1i)" => "0072: 0@(1@,1i) 0@(1@,1i)",
                    // [local var: float] /= [local var: float]
                    "0@(1@,1f) /= 0@(1@,1f)" => "0073: 0@(1@,1f) 0@(1@,1f)",
                   // [global var: int] /= [local var: int]
                    "$2($3,1i) /= 0@(1@,1i)" => "0074: $2($3,1i) 0@(1@,1i)",
                    // [global var: float] /= [local var: float]
                    "$2($3,1f) /= 0@(1@,1f)" => "0075: $2($3,1f) 0@(1@,1f)",
                    // [local var: int] /= [global var: int]
                    "0@(1@,1i) /= $2($3,1i)" => "0076: 0@(1@,1i) $2($3,1i)",
                    // [local var: float] /= [global var: float]
                    "0@(1@,1f) /= $2($3,1f)" => "0077: 0@(1@,1f) $2($3,1f)"
                )
            );
        }

        // var /= literal
        t! {
            for Game::vcs => (
                "$89 /= 1" => "000D: $89 1",
                "$TEMPVAR_FLOAT_1 /= 1.741" => "000E: $TEMPVAR_FLOAT_1 1.741",
                "0@ /= 1" => "000D: 0@ 1",
                "0@ /= 1.741" => "000E: 0@ 1.741"
            )
        }

        // var /= var
        t! {
            for Game::vcs => (
                "$2($3,1i) /= $2($3,1i)" => "002F: $2($3,1i) $2($3,1i)",
                "$2($3,1f) /= $2($3,1f)" => "0030: $2($3,1f) $2($3,1f)",
                "0@(1@,1i) /= 0@(1@,1i)" => "002F: 0@(1@,1i) 0@(1@,1i)",
                "0@(1@,1f) /= 0@(1@,1f)" => "0030: 0@(1@,1f) 0@(1@,1f)",
                "0@(1@,1i) /= $2($3,1i)" => "002F: 0@(1@,1i) $2($3,1i)",
                "0@(1@,1f) /= $2($3,1f)" => "0030: 0@(1@,1f) $2($3,1f)",
                "$2($3,1i) /= 0@(1@,1i)" => "002F: $2($3,1i) 0@(1@,1i)",
                "$2($3,1f) /= 0@(1@,1f)" => "0030: $2($3,1f) 0@(1@,1f)"
            )
        }
    }
    #[test]
    fn test_binary_comparison() {
        for game in [Game::gta3, Game::vc, Game::sa, Game::lcs, Game::sa_mobile] {
            t!(
                for game => (
                    "$CATALINA_TOTAL_PASSED_MISSIONS > 2" => "0018: $CATALINA_TOTAL_PASSED_MISSIONS 2",
                    "0@ > 0" => "0019: 0@ 0",
                    "$HJ_TWOWHEELS_DISTANCE_FLOAT > 0.0" => "0020: $HJ_TWOWHEELS_DISTANCE_FLOAT 0.0",
                    "26@ > 64.0" => "0021: 26@ 64.0"
                )
            );
        }

        t! {
            for Game::vcs => (
                "$1448 > 0" => "000F: $1448 0",
                "3@ > 380.0" => "0012: 3@ 380.0"
            )
        }
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
