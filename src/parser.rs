use std::str::FromStr;

use chumsky::prelude::*;
use chumsky::text;

use crate::{Entry, Shell, Target};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Entry>> {
    let ws = text::whitespace().repeated();

    let ident = any()
        .filter(|c: &char| c.is_alphanumeric())
        .repeated()
        .at_least(1)
        .to_slice()
        .map(str::to_string);

    let code_block = any()
        .filter(|c: &char| *c != '}')
        .repeated()
        .to_slice()
        .map(str::to_string);

    let alias_target = text::keyword("is")
        .ignore_then(ws.clone())
        .ignore_then(ident.clone())
        .map(|ident| Target::Alias(ident));

    let eval_target = text::keyword("eval")
        .ignore_then(just('{'))
        .ignore_then(code_block)
        .then_ignore(just('}'))
        .map(|template| Target::EvalOutput {
            template,
            cache: false, // TODO
        });

    let code_target = text::keyword("via")
        .ignore_then(ws.clone())
        .ignore_then(ident.clone()) // shell
        .then_ignore(ws.clone())
        .then_ignore(just('{'))
        .then(code_block)
        .then_ignore(just('}'))
        .map(|(shell, raw)| {
            let shell = shell.parse()?;
            let ret = Target::Code { shell, raw };
            Ok::<_, <Shell as FromStr>::Err>(ret)
        })
        .unwrapped();

    // Target: `is <alias>`  OR `eval { <command> }` OR  `via <shell> { <code> }`
    let target = alias_target
        .rewind()
        .or(eval_target)
        .rewind()
        .or(code_target);

    // Entry: `@ <executable> <target>`
    let entry = just('@')
        .ignore_then(ws.clone())
        .ignore_then(ident)
        .then_ignore(ws.clone())
        .then(target)
        .map(|(executable, target)| Entry { executable, target });

    ws.clone()
        .ignore_then(entry.then_ignore(ws).repeated().collect())
        .then_ignore(end())
}
