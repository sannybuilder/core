use super::{classes_parser::Param, Command, CommandParam, CommandParamSource, Operator};

pub fn command_to_snippet_line(command: &Command) -> String {
    let mut line = format!("{{{:04X}:}} ", command.id);
    if command.attrs.is_condition {
        line += "  ";
    }

    if let Some(operator) = &command.operator {
        let cb = |param: &CommandParam| braceify(stringify_type_and_source(param), "[]");
        return line + &stringify_command_with_operator(command, operator, &cb, &cb);
    }

    let output = output_params(command);
    let input = input_params(command);

    if !output.is_empty() {
        line += &stringify(output_params(command), ", ", |p| {
            braceify(
                if !p.name.is_empty() {
                    stringify_with_colon(p)
                } else {
                    stringify_type_and_source(p)
                },
                "[]",
            )
        });
        line += " = ";
    }

    line += &[
        command.name.to_lowercase(),
        stringify(input, " ", |param| {
            let t = braceify(stringify_type_and_source(param), "[]");
            if !param.name.is_empty() && !param.name.eq("self") {
                return format!("{} {}", get_param_name(param), t);
            }
            t
        }),
    ]
    .iter()
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string())
    .collect::<Vec<String>>()
    .join(" ");

    line
}

fn input_params(command: &Command) -> Vec<CommandParam> {
    command.input.clone()
}

fn output_params(command: &Command) -> Vec<CommandParam> {
    command.output.clone()
}

fn stringify<F>(params: Vec<CommandParam>, sep: &str, map_fn: F) -> String
where
    F: Fn(&CommandParam) -> String,
{
    params.iter().map(map_fn).collect::<Vec<String>>().join(sep)
}

fn get_param_name(param: &CommandParam) -> String {
    match &param.source {
        CommandParamSource::AnyVar
        | CommandParamSource::AnyVarGlobal
        | CommandParamSource::AnyVarLocal => format!("{{var_{}}}", param.name),
        _ => format!("{{{}}}", param.name),
    }
}

fn stringify_with_colon(p: &CommandParam) -> String {
    [
        [stringify_source(&p.source), p.name.clone()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
            .join(" "),
        p.r#type.clone(),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect::<Vec<String>>()
    .join(": ")
}

fn stringify_source(source: &CommandParamSource) -> String {
    match source {
        CommandParamSource::AnyVar => "var".to_string(),
        CommandParamSource::AnyVarGlobal => "global var".to_string(),
        CommandParamSource::AnyVarLocal => "local var".to_string(),
        CommandParamSource::Literal => "literal".to_string(),
        CommandParamSource::Pointer => "pointer".to_string(),
        _ => "".to_string(),
    }
}

fn stringify_type_and_source(p: &CommandParam) -> String {
    [
        [stringify_source(&p.source)]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
            .join(" "),
        p.r#type.clone(),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect::<Vec<String>>()
    .join(" ")
}

fn braceify(value: String, braces: &str) -> String {
    format!(
        "{}{}{}",
        braces.chars().nth(0).unwrap(),
        value,
        braces.chars().nth(1).unwrap()
    )
}

fn stringify_command_with_operator<Cb>(
    command: &Command,
    operator: &Operator,
    on_input: Cb,
    on_output: Cb,
) -> String
where
    Cb: Fn(&CommandParam) -> String,
{
    let output = output_params(command);
    let input = input_params(command);

    let operator = match operator {
        Operator::Assignment => "=",
        Operator::Addition => "+",
        Operator::Subtraction => "-",
        Operator::Multiplication => "*",
        Operator::Division => "/",
        Operator::TimedAddition => "+=@",
        Operator::TimedSubtraction => "-=@",
        Operator::CastAssignment => "=#",
        Operator::IsEqualTo => "==",
        Operator::IsGreaterThan => ">",
        Operator::IsGreaterOrEqualTo => ">=",
        Operator::And => "&",
        Operator::Or => "|",
        Operator::Xor => "^",
        Operator::Not => "~",
        Operator::Mod => "%",
        Operator::ShiftLeft => "<<",
        Operator::ShiftRight => ">>",
    };

    if input.len() == 1 && output.is_empty() {
        // unary
        return format!("{}{}", operator, on_input(&input[0]));
    }
    if !output.is_empty() {
        // binary not
        if operator.eq("~") {
            return format!("{} = ~{}", on_output(&output[0]), on_input(&input[0]));
        }

        // ternary
        return format!(
            "{} = {} {} {}",
            on_output(&output[0]),
            on_input(&input[0]),
            operator,
            on_input(&input[1])
        );
    }

    match &operator {
        // assignment or comparison
        op if ["=", "+=@", "-=@", "=#", "==", ">", ">="].contains(&op) => {
            return format!("{} {} {}", on_input(&input[0]), op, on_input(&input[1]));
        }

        // compound assignment
        _ => {
            return format!(
                "{} {}= {}",
                on_input(&input[0]),
                operator,
                on_input(&input[1])
            );
        }
    }
}
