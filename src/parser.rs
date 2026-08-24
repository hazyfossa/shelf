use crate::Shell;

#[rustfmt::skip]
enum Expr {
    Alias { from: String, to: String },
    Eval { from_cmd: String },
    Arbitrary { shell: Option<Shell>, literal: String },
}

struct Item<'a> {
    name: &'a str,
    shell: Option<Shell>,
    code: &'a str,
}
