use std::{collections::HashSet, env, fs, path::PathBuf};

use argh::FromArgs;
use chumsky::Parser;
use eyre::{Context, Result};
use strum::EnumString;
use wincode::{SchemaRead, SchemaWrite};

use crate::utils::{WarnExt, xdg_dir};

hazymacros::setup!();

mod parser;
mod utils;

type ShellCode = String;
type Executable = String;

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

impl Shell {
    fn alias(from: &str, to: &str) -> ShellCode {
        todo!()
    }

    fn eval_output(cmd: &str) -> ShellCode {
        todo!()
    }
}

#[derive(EnumString, SchemaRead, SchemaWrite)]
#[allow(non_camel_case_types)]
enum Interface {
    cd,
    ls,
    cat,
    grep,
    sudo,
    less,
}

enum Target {
    Interface(Interface),
    Alias { target: String },
    EvalOutput { template: String, memoize: bool },
    Code { shell: Shell, raw: String },
}

struct Entry {
    executable: Executable,
    target: Target,
}

#[derive(SchemaRead, SchemaWrite)]
struct SelectedProvider {
    name: Executable,
    selected_from: HashSet<Executable>,
}

#[derive(SchemaRead, SchemaWrite)]
struct State {
    selected_providers: Vec<(Interface, SelectedProvider)>,
    cached_code: ShellCode,
}

#[derive(FromArgs)]
#[rustfmt::skip]
/// TODO
struct InitArgs {
    /// disables interactivity guards, even if the shell supports them
    #[argh(switch)] even_inside_scripts: bool,

    /// disables internal caching
    #[argh(switch)] no_cache: bool,
}

#[derive(FromArgs)]
#[rustfmt::skip]
/// Shelf - Useful for keeping shells in order
struct Args {
    /// path to config
    #[argh(option)] config: Option<PathBuf>,

    /// path to cached state
    #[argh(option)] cache: Option<PathBuf>,
}

fn load_config_dir(path: Option<PathBuf>) -> Result<Vec<Entry>> {
    let path = path.unwrap_or_else(|| xdg_dir("XDG_CONFIG", ".config").join("shelf.d"));

    let entries = fs::read_dir(path)?
        .filter_map(|file_entry| {
            let path = file_entry.warn()?.path();
            let content = fs::read_to_string(path).warn()?;
            let new_entries = parser::parser().parse(&content).unwrap(); // TODO
            Some(new_entries)
        })
        .flatten()
        .collect();

    Ok(entries)
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let apps = load_config_dir(args.config)?;

    Ok(())
}
