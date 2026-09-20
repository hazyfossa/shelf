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

    let interface_target = text::keyword("implements")
        .ignore_then(ws.clone())
        .ignore_then(ident.clone())
        .map(|ident| ident.parse().map(Target::Interface))
        .unwrapped();

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

    // Target: `implements <interface>`  OR  `via <shell> { <code> }`
    let target = interface_target.rewind().or(code_target);

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
