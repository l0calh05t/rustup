use std::collections::VecDeque;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::process;

pub const RUST_RECURSION_COUNT_MAX: u32 = 20;

pub(crate) fn prepend_path(name: &str, prepend: Vec<PathBuf>, cmd: &mut Command) {
    let old_value = process().var_os(name);
    let parts = if let Some(ref v) = old_value {
        let mut tail = env::split_paths(v).collect::<VecDeque<_>>();
        for path in prepend.into_iter().rev() {
            if !tail.contains(&path) {
                tail.push_front(path);
            }
        }
        tail
    } else {
        prepend.into()
    };

    if let Ok(new_value) = env::join_paths(parts) {
        cmd.env(name, new_value);
    }
}

pub(crate) fn inc(name: &str, cmd: &mut Command) {
    let old_value = process()
        .var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    cmd.env(name, (old_value + 1).to_string());
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ffi::{OsStr, OsString};

    use rustup_macros::unit_test as test;

    use super::*;
    use crate::currentprocess;
    use crate::test::{with_saved_path, Env};

    #[test]
    fn prepend_unique_path() {
        let mut vars = HashMap::new();
        vars.env(
            "PATH",
            env::join_paths(vec!["/home/a/.cargo/bin", "/home/b/.cargo/bin"].iter()).unwrap(),
        );
        let tp = Box::new(currentprocess::TestProcess {
            vars,
            ..Default::default()
        });
        with_saved_path(&mut || {
            currentprocess::with(tp.clone(), || {
                let mut path_entries = vec![];
                let mut cmd = Command::new("test");

                let a = OsString::from("/home/a/.cargo/bin");
                let path_a = PathBuf::from(a);
                path_entries.push(path_a);

                let _a = OsString::from("/home/a/.cargo/bin");
                let _path_a = PathBuf::from(_a);
                path_entries.push(_path_a);

                let z = OsString::from("/home/z/.cargo/bin");
                let path_z = PathBuf::from(z);
                path_entries.push(path_z);

                prepend_path("PATH", path_entries, &mut cmd);
                let envs: Vec<_> = cmd.get_envs().collect();

                assert_eq!(
                    envs,
                    &[(
                        OsStr::new("PATH"),
                        Some(
                            env::join_paths(
                                vec![
                                    "/home/z/.cargo/bin",
                                    "/home/a/.cargo/bin",
                                    "/home/b/.cargo/bin"
                                ]
                                .iter()
                            )
                            .unwrap()
                            .as_os_str()
                        )
                    ),]
                );
            });
        });
    }
}
