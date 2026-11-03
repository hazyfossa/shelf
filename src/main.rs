use std::{fs, path::PathBuf};

use argh::FromArgs;
use chumsky::Parser;
use eyre::Result;
use strum::{Display, EnumString};

use crate::utils::{WarnExt, xdg_dir};

mod parser;
mod utils;

type ShellCode = String;
type Executable = String;

#[derive(EnumString, Display, PartialEq)]
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
    fn alias(&self, from: &str, to: &str) -> ShellCode {
        match self {
            Self::Fish => format!("alias {from}={to}"),
            _ => panic!(),
        }
    }

    fn eval_output(&self, cmd: &str) -> ShellCode {
        match self {
            Self::Fish => format!("{cmd} | source"),
            _ => panic!(),
        }
    }
}

enum Target {
    Alias(String),
    EvalOutput { template: String, cache: bool },
    Code { shell: Shell, raw: String },
}

struct Entry {
    executable: Executable,
    target: Target,
}

impl Entry {
    fn codegen(&self, shell: &Shell) -> Option<ShellCode> {
        match &self.target {
            Target::Alias(alias_target) => Some(shell.alias(&self.executable, &alias_target)),
            Target::EvalOutput { template, cache } => {
                let cmd = template.replace("{shell}", &shell.to_string());
                Some(shell.eval_output(&cmd))
            }
            Target::Code {
                shell: target_shell,
                raw,
            } => (target_shell == shell).then_some(raw.to_string()),
        }
    }
}

#[derive(FromArgs)]
#[rustfmt::skip]
/// Shelf - Useful for keeping shells in order
struct Args {
    /// path to config
    #[argh(option)] config: Option<PathBuf>,

    /// disables interactivity guards, even if the shell supports them
    #[argh(switch)] even_inside_scripts: bool,
}

fn load_config_dir(path: Option<PathBuf>) -> Result<impl Iterator<Item = Entry>> {
    let path = path.unwrap_or_else(|| xdg_dir("XDG_CONFIG", ".config").join("shelf.d"));

    let entries = fs::read_dir(path)?
        .filter_map(|file_entry| {
            let path = file_entry.warn()?.path();
            let content = fs::read_to_string(path).warn()?;
            let new_entries = parser::parser().parse(&content).unwrap(); // TODO
            Some(new_entries)
        })
        .flatten();

    Ok(entries)
}

fn current_shell() -> Result<Shell> {
    todo!()
}

fn executable_exists(name: &str) -> bool {
    which::which(name).ok().is_some()
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let shell = current_shell()?;

    let apps = load_config_dir(args.config)?;
    let existing_apps = apps.filter(|entry| executable_exists(&entry.executable));

    for app in existing_apps {
        if let Some(app_code) = app.codegen(&shell) {
            println!("{app_code}");
        }
    }

    Ok(())
}
