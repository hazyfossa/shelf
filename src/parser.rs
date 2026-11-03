use chumsky::prelude::*;
use chumsky::text;

use crate::{Entry, Shell, Target};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Entry>, extra::Err<Rich<'a, char>>> {
    let ident = any()
        .filter(|c: &char| c.is_alphanumeric() || matches!(*c, '_' | '-'))
        .repeated()
        .at_least(1)
        .to_slice()
        .try_map(|s: &str, span| {
            if matches!(s, "is" | "eval" | "via") {
                Err(Rich::custom(
                    span,
                    format!("Unexpected keyword `{s}` used as identifier"),
                ))
            } else {
                Ok(s.to_string())
            }
        });

    let code_block = just('{')
        .ignore_then(recursive(|recurse| {
            let string_literal = none_of('"')
                .repeated()
                .delimited_by(just('"'), just('"'))
                .ignored();

            let single_quote_literal = none_of('\'')
                .repeated()
                .delimited_by(just('\''), just('\''))
                .ignored();

            let nested_braces = recurse.delimited_by(just('{'), just('}')).ignored();

            let other_chars = none_of("{}\"'").ignored();

            choice((
                string_literal,
                single_quote_literal,
                nested_braces,
                other_chars,
            ))
            .repeated()
            .to_slice()
        }))
        .then_ignore(just('}'))
        .map(|s: &str| s.trim().to_string());

    let args_list = just('(')
        .ignore_then(
            none_of(')')
                .repeated()
                .to_slice()
                .map(|s: &str| s.trim().to_string()),
        )
        .then_ignore(just(')'))
        .or_not();

    let alias_target = text::ascii::keyword("is")
        .padded()
        .ignore_then(ident.clone())
        .map(|name| Target::Alias { name, args: None });

    let eval_target = text::ascii::keyword("eval")
        .padded()
        .ignore_then(code_block.clone())
        .map(|template| Target::EvalOutput {
            arg_template: template,
            cache: false,
        });

    let code_target = text::ascii::keyword("via")
        .padded()
        .ignore_then(ident.clone()) // shell name
        .padded()
        .then(code_block)
        .try_map(|(shell_str, raw), span| match shell_str.parse::<Shell>() {
            Ok(shell) => Ok(Target::Code { shell, raw }),
            Err(_) => Err(Rich::custom(span, format!("Invalid shell: {shell_str}"))),
        });

    let target = choice((alias_target, eval_target, code_target));

    let entry = just('@')
        .padded()
        .ignore_then(ident.clone())
        .padded()
        .then(args_list)
        .padded()
        .then(target)
        // TODO: rethink syntax so this is not needed
        .try_map(|((executable, args), mut target), span| {
            // Inject parsed arguments into the Alias target if they exist
            if let Some(a) = args {
                if let Target::Alias {
                    args: target_args, ..
                } = &mut target
                {
                    *target_args = Some(a);
                } else {
                    return Err(Rich::custom(
                        span,
                        "Arguments in parentheses are only allowed for 'is' (alias) targets",
                    ));
                }
            }
            Ok(Entry { executable, target })
        });

    text::whitespace()
        .ignore_then(entry.padded().repeated().collect())
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::Parser;

    fn parse(input: &str) -> Vec<Entry> {
        parser().parse(input).unwrap()
    }

    #[test]
    fn test_empty_input() {
        let result = parse("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let result = parse("   \n\t  \r\n  ");
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_alias_entry() {
        let input = "@ eza is ls";
        let result = parse(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].executable, "eza");
        assert_eq!(
            result[0].target,
            Target::Alias {
                name: "ls".to_string(),
                args: None
            }
        );
    }

    #[test]
    fn test_single_eval_entry() {
        let input = "@ mycmd eval { echo \"hello\" }";
        let result = parse(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].executable, "mycmd");
        assert_eq!(
            result[0].target,
            Target::EvalOutput {
                arg_template: "echo \"hello\"".to_string(),
                cache: false,
            }
        );
    }

    #[test]
    fn test_single_code_entry() {
        let input = "@ runner via bash { ls -la }";
        let result = parse(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].executable, "runner");
        assert_eq!(
            result[0].target,
            Target::Code {
                shell: Shell::Bash,
                raw: "ls -la".to_string(),
            }
        );
    }

    #[test]
    fn test_multiple_entries() {
        let input = r#"
            @ cmd1 is alias1
            @ cmd2 eval{echo "test"}
            @ cmd3 via bash { pwd }
        "#;
        let result = parse(input);
        assert_eq!(result.len(), 3);

        assert_eq!(result[0].executable, "cmd1");
        assert_eq!(
            result[0].target,
            Target::Alias {
                name: "alias1".to_string(),
                args: None,
            }
        );

        assert_eq!(result[1].executable, "cmd2");
        assert_eq!(
            result[1].target,
            Target::EvalOutput {
                arg_template: "echo \"test\"".to_string(),
                cache: false
            }
        );

        assert_eq!(result[2].executable, "cmd3");
        assert_eq!(
            result[2].target,
            Target::Code {
                shell: Shell::Bash,
                raw: "pwd".to_string()
            }
        );
    }

    #[test]
    fn test_invalid_unclosed_braces() {
        let input_eval = "@ cmd eval{echo \"hello\""; // Missing '}'
        assert!(parser().parse(input_eval).has_errors());

        let input_code = "@ cmd via bash { echo \"hello\""; // Missing '}'
        assert!(parser().parse(input_code).has_errors());
    }
}
