use std::{env, fmt::Debug, path::PathBuf};

pub trait WarnExt<T> {
    fn warn(self) -> Option<T>;
}

impl<T, E: Debug> WarnExt<T> for std::result::Result<T, E> {
    fn warn(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(e) => {
                eprintln!("WARN: {e:?}");
                None
            }
        }
    }
}

pub fn xdg_dir(var: &str, default: &str) -> PathBuf {
    let home = env::home_dir().expect("Cannot locate HOME");

    env::var_os(var)
        .map(PathBuf::from)
        .unwrap_or(home.join(default))
}
