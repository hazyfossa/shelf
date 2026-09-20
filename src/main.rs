use std::{collections::HashSet, env, fs, path::PathBuf};

use argh::FromArgs;
use chumsky::Parser;
use eyre::{Context, Result, bail};
use strum::EnumString;
use wincode::{SchemaRead, SchemaWrite};

hazymacros::setup!();

mod parser;

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
/// Shelf - Useful for keeping shells in order
struct InitArgs {
    /// disables interactivity guards, even if the shell supports them
    #[argh(switch)] even_inside_scripts: bool,

    /// disables internal caching
    #[argh(switch)] no_cache: bool,

    /// path to config
    #[argh(option)] config: Option<PathBuf>,
}

fn default_config_path() -> Result<PathBuf> {
    let home = env::home_dir().expect("Cannot locate HOME");
    let dot_config = match env::var("XDG_CONFIG") {
        Ok(x) => PathBuf::from(x),
        Err(env::VarError::NotPresent) => home.join(".config"),
        Err(other) => bail!(other),
    };

    Ok(dot_config.join("config.shelf"))
}

fn main() -> Result<()> {
    let args: InitArgs = argh::from_env();
    let config_path = args.config.unwrap_or(default_config_path()?);

    let config = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config from {config_path:?}"))?;

    // TODO: proper parsing errors
    let entries = parser::parser().parse(&config).unwrap();

    Ok(())
}
