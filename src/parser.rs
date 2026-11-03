use chumsky::prelude::*;
use chumsky::text;

use crate::{Entry, Shell, Target};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Entry>, extra::Err<Rich<'a, char>>> {
    let ws = text::whitespace().repeated();

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
        .ignore_then(recursive(|code_block| {
            let string_literal = just('"')
                .ignore_then(none_of('"').repeated())
                .then_ignore(just('"'))
                .ignored();

            let single_quote_literal = just('\'')
                .ignore_then(none_of('\'').repeated())
                .then_ignore(just('\''))
                .ignored();

            // Allow nested braces by recursively parsing them
            let nested_braces = just('{')
                .ignore_then(code_block.clone())
                .then_ignore(just('}'));

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
        .map(str::to_string);

    let alias_target = text::keyword("is")
        .padded()
        .ignore_then(ident.clone())
        .map(Target::Alias);

    let eval_target = text::keyword("eval")
        .padded()
        .ignore_then(code_block.clone())
        .map(|template| Target::EvalOutput {
            template,
            cache: false, // TODO
        });

    let code_target = text::keyword("via")
        .padded()
        .ignore_then(ident.clone()) // shell name
        .padded()
        .then(code_block)
        .try_map(|(shell_str, raw), span| match shell_str.parse::<Shell>() {
            Ok(shell) => Ok(Target::Code { shell, raw }),
            Err(_) => Err(Rich::custom(
                span,
                format!("Unsupported shell: {shell_str}"),
            )),
        });

    let target = choice((alias_target, eval_target, code_target));

    let entry = just('@')
        .padded()
        .ignore_then(ident)
        .padded()
        .then(target)
        .map(|(executable, target)| Entry { executable, target });

    ws.ignore_then(entry.padded().repeated().collect())
        .then_ignore(end())
}
