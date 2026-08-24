mod parser;

use argh::FromArgs;
use eyre::Result;
use strum::EnumString;

hazymacros::setup!();

#[derive(EnumString)]
#[strum(serialize_all = "lowercase")]
enum Shell {
    Bash,
    Elvish,
    Fish,
    // Nushell,
    Xonsh,
    Zsh,
}

// impl Shell {
//     fn alias(from: &str, to: &str) -> String {}
//     fn eval(cmd: &str) -> String {}
// }

type App = String;

#[derive(EnumString)]
#[allow(non_camel_case_types)]
enum Interface {
    cd,
    ls,
    cat,
    grep,
    sudo,
    less,
}

#[derive(FromArgs)]
#[rustfmt::skip]
/// Useful for keeping shells in order
struct InitArgs {
    /// disables interactivity guards, even if the shell supports them
    #[argh(switch)] even_inside_scripts: bool,

    /// completely disables internal caching
    #[argh(switch)] no_cache: bool,
}

fn main() -> Result<()> {
    Ok(())
}
