#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{
    collections::HashMap,
    fs::File,
    io,
    process::{self, Child},
};
pub fn launch_child(
    command: &str,
    current_dir: &Option<String>,
    log_file: Option<File>,
    env: &Option<Vec<(String, String)>>,
) -> io::Result<Child> {
    let args = shell_words::split(command).unwrap();

    let mut output = process::Command::new(&*args[0]);
    if args.len() > 1 {
        output.args(&args[1..]);
    }
    if let Some(cur_dir) = current_dir {
        output.current_dir(cur_dir);
    };

    match log_file {
        Some(log_file) => output.stderr(log_file.try_clone()?).stdout(log_file),
        None => output
            .stderr(process::Stdio::null())
            .stdout(process::Stdio::null()),
    };

    if let Some(env) = env {
        for e in env {
            output.env(&e.0, &e.1);
        }
    }
    #[cfg(unix)]
    unsafe {
        output.pre_exec(|| {
            libc::setsid();
            Ok(())
        })
    };
    #[cfg(windows)]
    {}

    output.spawn()
}

#[derive(Debug)]
pub struct LaunchConfig {
    procs: Vec<LaunchProc>,
}

#[derive(Debug)]
struct LaunchProc {
    command: String,
    current_dir: Option<String>,
    log_file: Option<String>,
    env: Option<Vec<(String, String)>>,
}

impl LaunchConfig {
    pub fn new(processes: &Vec<&toml_edit::Table>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut res = vec![];
        for p in processes {
            let command: String;
            if let Some(com) = p["command"].as_str() {
                command = com.to_owned();
            } else {
                // Handle the None case, e.g., log an error or skip
                continue;
            }

            let current_dir = match p.get("current_dir") {
                Some(v) => match v.as_str() {
                    Some(s) => Some(s.to_owned()),
                    None => None,
                },
                None => None,
            };

            let log_file = match p.get("log_file") {
                Some(v) => match v.as_str() {
                    Some(s) => Some(s.to_owned()),
                    None => None,
                },
                None => None,
            };

            let env: Option<Vec<(String, String)>> =
                if let Some(env_table) = p.get("env").and_then(|e| e.as_inline_table()) {
                    Some(
                        env_table
                            .iter()
                            .map(|(key, val)| (key.to_owned(), val.as_str().unwrap().to_owned()))
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                };

            res.push(LaunchProc {
                command,
                current_dir,
                log_file,
                env,
            });
        }
        Ok(LaunchConfig { procs: res })
    }

    pub fn start(&self) -> HashMap<&str, u32> {
        let mut res = HashMap::new();
        for conf in &self.procs {
            let logfile = match &conf.log_file {
                Some(f_path) => match std::fs::File::create(f_path) {
                    Ok(f) => Some(f),
                    Err(_) => None,
                },
                None => None,
            };
            let child = launch_child(&conf.command, &conf.current_dir, logfile, &conf.env);

            let child = match child {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{e:?}");
                    continue;
                }
            };

            let id = child.id();

            res.insert(conf.command.as_str(), id);
        }

        res
    }
}
