use chumsky::prelude::*;
use chumsky::text;

use crate::{Entry, Shell, Target};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Entry>, extra::Err<Rich<'a, char>>> {
    let ident = any()
        .filter(|c: &char| c.is_alphanumeric() || *c == '_' || *c == '-')
        .repeated()
        .at_least(1)
        .to_slice()
        .map(str::to_string)
        .try_map(|s: String, span| {
            if s == "is" || s == "eval" || s == "via" {
                Err(Rich::custom(
                    span,
                    format!("Unexpected keyword `{}` used as identifier", s),
                ))
            } else {
                Ok(s)
            }
        });

    let code_block = just('{')
        .ignore_then(recursive(|recurse| {
            let string_literal = just('"')
                .ignore_then(none_of('"').repeated())
                .then_ignore(just('"'))
                .ignored();

            let single_quote_literal = just('\'')
                .ignore_then(none_of('\'').repeated())
                .then_ignore(just('\''))
                .ignored();

            // Allow nested braces by recursively parsing them
            // TODO: the following repeats the logic around code_block
            // extract is as bracket_pair helper
            let nested_braces = just('{')
                .ignore_then(recurse)
                .then_ignore(just('}'))
                .ignored();

            // Anything else that isn't a brace or quote
            let other_chars = none_of("{}\"'").ignored();

            choice((
                string_literal,
                single_quote_literal,
                nested_braces,
                other_chars,
            ))
            .repeated()
        }))
        .then_ignore(just('}'))
        .to_slice()
        .map(|x: &str| {
            x.strip_prefix('{')
                .unwrap()
                .strip_suffix('}')
                .unwrap()
                .trim()
                .to_string()
        });

    let alias_target = text::keyword("is")
        .padded()
        .ignore_then(ident.clone())
        .map(|name| Target::Alias { name, args: None });

    let eval_target = text::keyword("eval")
        .padded()
        .ignore_then(code_block.clone())
        .map(|template| Target::EvalOutput {
            arg_template: template,
            cache: false,
        });

    let code_target = text::keyword("via")
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
        .ignore_then(ident)
        .padded()
        .then(target)
        .map(|(executable, target)| Entry { executable, target });

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
        assert_eq!(result[0].executable, "ls");
        assert_eq!(
            result[0].target,
            Target::Alias {
                name: "list".to_string(),
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
