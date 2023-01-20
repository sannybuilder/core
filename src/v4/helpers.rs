use std::fmt::Display;

pub fn format_unary(command_name: impl Display, operand: impl Display) -> Option<String> {
    format!("{} {}", command_name, operand).into()
}

pub fn format_binary(
    command_name: impl Display,
    operand1: impl Display,
    operand2: impl Display,
) -> Option<String> {
    format!("{} {} {}", command_name, operand1, operand2).into()
}

pub fn format_ternary(
    command_name: impl Display,
    operand1: impl Display,
    operand2: impl Display,
    operand3: impl Display,
) -> Option<String> {
    format!("{} {} {} {}", command_name, operand1, operand2, operand3).into()
}
