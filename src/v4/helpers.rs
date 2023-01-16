pub fn format_unary(command_name: &str, operand: &str) -> Option<String> {
    [command_name, operand].join(" ").into()
}

pub fn format_binary(command_name: &str, operand1: &str, operand2: &str) -> Option<String> {
    [command_name, operand1, operand2].join(" ").into()
}

pub fn format_ternary(
    command_name: &str,
    operand1: &str,
    operand2: &str,
    operand3: &str,
) -> Option<String> {
    [command_name, operand1, operand2, operand3]
        .join(" ")
        .into()
}
