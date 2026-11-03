use std::{
    fs::{self, File},
    io::Read,
    os::unix,
    path::{Path, PathBuf},
};

use argh::FromArgs;
use chumsky::Parser;
use eyre::{Context, Result, eyre};
use strum::{Display, EnumString};

use crate::utils::{WarnExt, xdg_dir};

mod parser;
mod utils;

type Executable = String;

#[derive(Debug, EnumString, Display, PartialEq)]
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
    fn alias(&self, from: &str, to: &str) {
        match self {
            Self::Fish => println!("alias {from}=\"{to}\""),
            _ => todo!(),
        }
    }

    fn eval_output(&self, cmd: &str) {
        match self {
            Self::Fish => println!("{cmd} | source"),
            _ => todo!(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Target {
    Alias { name: String, args: Option<String> },
    EvalOutput { arg_template: String, cache: bool },
    Code { shell: Shell, raw: String },
}

struct Entry {
    executable: Executable,
    target: Target,
}

impl Entry {
    fn codegen(&self, shell: &Shell) {
        let exe = &self.executable;
        match &self.target {
            Target::Alias { name, args } => {
                let mut alias_target = exe.clone();

                if let Some(args) = args {
                    alias_target.push(' ');
                    alias_target.push_str(args);
                }

                shell.alias(name, &alias_target);
            }
            Target::EvalOutput {
                arg_template,
                cache,
            } => {
                let args = arg_template.replace("{shell}", &shell.to_string());
                shell.eval_output(&format!("{exe} {args}"))
            }
            Target::Code {
                shell: target_shell,
                raw,
            } => {
                // TODO: if an app was found, but no entry for it exists for a current shell
                // it will be skipped silently, which might be confusing
                if target_shell == shell {
                    println!("{raw}");
                }
            }
        }
    }
}

#[derive(FromArgs)]
#[rustfmt::skip]
/// Shelf - Useful for keeping shells in order
struct Args {
    /// path to config
    #[argh(option)] config: Option<PathBuf>,
}

fn load_config_dir(path: Option<PathBuf>) -> Result<impl Iterator<Item = Entry>> {
    let path = path.unwrap_or_else(|| xdg_dir("XDG_CONFIG", ".config").join("shelf.d"));

    let entries = fs::read_dir(&path)
        .with_context(|| format!("Failed to read config dir at {path:?}"))?
        .filter_map(|file_entry| {
            let path = file_entry.warn()?.path();

            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read: {path:?}"))
                .warn()?;

            let new_entries = parser::parser().parse(&content).unwrap(); // TODO
            Some(new_entries)
        })
        .flatten();

    Ok(entries)
}

fn read_file(path: impl AsRef<Path>) -> Result<String> {
    let mut buf = String::new();
    File::open(path)?.read_to_string(&mut buf)?;
    Ok(buf)
}

fn current_shell() -> Result<Shell> {
    let parent_pid = unix::process::parent_id();

    let parent_name = read_file(format!("/proc/{parent_pid}/comm"))
        .context("Failed to read /proc entry for parent")
        .context("Shell detection failed")?;

    parent_name
        .trim()
        .parse()
        .map_err(|_| eyre!("Current shell ({parent_name}) is not supported"))
}

fn executable_exists(name: &Executable) -> bool {
    which::which(name).ok().is_some()
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let shell = current_shell()?;

    let apps = load_config_dir(args.config)?;
    let existing_apps = apps.filter(|entry| executable_exists(&entry.executable));

    for app in existing_apps {
        app.codegen(&shell);
    }

    Ok(())
}
