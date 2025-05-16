#![cfg(any(unix, windows))]

#[cfg(not(any(unix, windows)))]
compile_error!("이 프로젝트는 Unix 계열 OS와 윈도우즈 OS에서만 빌드할 수 있습니다.");

use std::{
    env::{self},
    io::Write,
    path::{Path, PathBuf},
};

use colored::Colorize;
use toml_edit::DocumentMut;

mod start;

fn start(unit: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = std::fs::read_to_string(path)?;
    let mut doc: DocumentMut = config.parse()?;

    let procs = doc[unit]
        .as_array_of_tables_mut()
        .ok_or("dongtomi 항목이 배열이 아님")?;
    // 복사본 만들고 기존 프로세스에서 id 있는 거 제거
    let mut to_launch = vec![];

    for proc in procs.iter() {
        if proc.get("id").is_none() {
            to_launch.push(proc); // id 없는 것만 복사
        } else if let Some(id) = proc.get("id") {
            eprintln!("already exist {}", id);
        }
    }

    //런치정보 객체 생성
    let a = start::LaunchConfig::new(&to_launch)?;
    // 실행
    let pids = a.start();
    for proc in procs.iter_mut() {
        //command 값
        let command = {
            let cmd = match proc["command"].as_str() {
                Some(c) => c,
                None => continue,
            };
            cmd.to_owned()
        };

        //그 객체에 id가 있으면 -> 실행이 잘 됐으면 pid 가져오기
        let pid = match pids.get(command.as_str()) {
            Some(a) => a,
            None => continue,
        };

        proc.insert("id", toml_edit::value(*pid as i64));
    }

    pids.iter()
        .for_each(|(command, pid)| println!("\"{}\" -> PID: {}", command, pid));
    let mut config_file = std::fs::File::create(path)?;
    config_file.write_all(doc.to_string().as_bytes())?;

    Ok(())
}

fn end(unit: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = std::fs::read_to_string(path)?;
    let mut config: DocumentMut = config.parse()?;

    let procs = config[unit].as_array_of_tables_mut().unwrap();

    for proc in procs.iter_mut() {
        let p = proc;
        let command = p["command"].as_str().unwrap().to_owned();
        let id = match p.get("id") {
            Some(i) => i.as_integer().unwrap() as i32,
            None => {
                eprintln!("{} : there is not pid.", command);
                continue;
            }
        };
        let mut remove_id = || match p.remove("id") {
            Some(_) => {}
            None => eprintln!("엥"),
        };
        #[cfg(unix)]
        {
            let result = unsafe { libc::kill(id, libc::SIGTERM) };

            if result == 0 {
                println!("{} [{}] 종료 성공", command, id);
                remove_id();
            } else {
                let err = std::io::Error::last_os_error();
                if !(err.raw_os_error() == Some(libc::EPERM)) {
                    remove_id();
                }
                println!("PID [{}] 종료 실패: {}", id, err);
            }
        }
        #[cfg(windows)]
        {
            let output = std::process::Command::new("taskkill")
                .args(["/T", "/PID"])
                .arg(&*(id.to_string()))
                .output();

            let output = match output {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e:?}");
                    continue;
                }
            };

            if output.status.success() {
                println!("{} [{}] 종료 성공", command, id);
                remove_id();
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);

                if stderr.contains("not found") || stderr.contains("is not running") {
                    println!("❌ 프로세스가 존재하지 않음");
                    remove_id();
                } else if stderr.contains("Access is denied") {
                    println!("❌ 관리자 권한 부족");
                } else {
                    println!("❌ 알 수 없는 에러:\n{}", stderr);
                    remove_id();
                }
            };
        }
    }

    let config = config.to_string();
    let mut file = std::fs::File::create(path)?;
    file.write(config.as_bytes())?;

    Ok(())
}

fn custom(command: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = std::fs::read_to_string(path)?;
    let config: DocumentMut = config.parse()?;

    let command = if let Some(command) = config.get(command) {
        command
    } else {
        eprintln!("{command} is not exist in Launch.toml");
        return Ok(());
    };

    let args = shell_words::split(command["command"].as_str().unwrap()).unwrap();

    let mut cmd = std::process::Command::new(&*args[0]);

    let mut cmd = cmd.stdout(std::io::stdout()).stderr(std::io::stderr());

    if args.len() > 1 {
        cmd = cmd.args(&args[1..])
    }

    if let Some(dir) = command.get("current_dir").and_then(|val| val.as_str()) {
        cmd = cmd.current_dir(dir);
    }

    let mut child = cmd.spawn().unwrap();

    child.wait()?;
    Ok(())
}

fn main() {
    let mut args = std::env::args();
    let _ = args.next(); // program name : 
    let command = if let Some(s) = args.next() {
        s
    } else {
        eprintln!("This process need arguments...");
        eprintln!("launcher < start | restart | end > <unit> [option..]");
        return;
    };
    //arg commands
    const START: &str = "start";
    const END: &str = "end";
    const RESTART: &str = "restart";
    const LIST: &str = "list";

    let unit = match &*command {
        START | END | RESTART => match args.next() {
            Some(unit) => unit,
            None => "process".to_owned(),
        },
        _ => String::new(),
    };
    let args = std::env::args().collect::<Vec<String>>();
    let file_path: PathBuf = get_launch_path(&args);

    match &*command {
        START => {
            start(&unit, &file_path).unwrap();
        }
        END => end(&unit, &file_path).unwrap(),
        RESTART => {
            end(&unit, &file_path).unwrap();
            start(&unit, &file_path).unwrap();
        }
        LIST => show_list(&file_path).unwrap(),
        a => custom(a, &file_path).unwrap(),
    }
}
fn get_launch_path(args: &[String]) -> PathBuf {
    let to_path = |path: &str| {
        let p: PathBuf = path.into();
        if p.is_dir() {
            return p.join("Launch.toml");
        }
        return p;
    };

    match args.flag("--path") {
        Some(path) => return to_path(path),
        None => {}
    }
    match args.flag("-P") {
        Some(path) => return to_path(path),
        None => {}
    }
    let home = home_dir().unwrap_or("/".into());
    home.join("Launch.toml")
}
fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        env::var_os("HOME").map(PathBuf::from)
    }

    #[cfg(windows)]
    {
        env::var_os("USERPROFILE")
            .or_else(|| {
                let drive = env::var_os("HOMEDRIVE");
                let path = env::var_os("HOMEPATH");
                match (drive, path) {
                    (Some(d), Some(p)) => Some(PathBuf::from(format!("{d}{p}"))),
                    _ => None,
                }
            })
            .map(PathBuf::from)
    }
}

trait Flag {
    fn flag(&self, flag: &str) -> Option<&str>;
}
impl Flag for [String] {
    fn flag(&self, flag: &str) -> Option<&str> {
        let mut iter = self.iter();
        while let Some(f) = iter.next() {
            if *f == flag {
                break;
            }
        }

        iter.next().and_then(|s| Some(s.as_str()))
    }
}

fn show_list(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_string = std::fs::read_to_string(file_path)?;
    let doc: DocumentMut = file_string.parse()?;

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum State {
        Started,
        Ended,
        Single,
    }
    impl AsRef<str> for State {
        fn as_ref(&self) -> &'static str {
            match self {
                State::Started => "Started",
                State::Ended => "Ended",
                State::Single => "Single Task",
            }
        }
    }

    impl State {
        fn display(&self) -> colored::ColoredString {
            match self {
                State::Started => self.as_ref().bright_green().bold(),
                State::Ended => self.as_ref().dimmed(),
                State::Single => self.as_ref().normal(),
            }
        }
    }

    let mut list: Vec<(&str, State)> = Vec::new();

    for (name, item) in doc.as_table() {
        match item {
            toml_edit::Item::Table(_) => list.push((name, State::Single)),
            toml_edit::Item::ArrayOfTables(arr) => {
                let mut started = false;
                for proc in arr {
                    if proc.get("id").is_some() {
                        started = true;
                    }
                }

                if started {
                    list.push((name, State::Started));
                } else {
                    list.push((name, State::Ended));
                }
            }
            _ => {}
        }
    }
    list.sort_by(|a, b| a.1.cmp(&b.1));
    list.into_iter()
        .for_each(|(name, state)| println!("   {:^12}|  {}", state.display(), name.bold()));

    Ok(())
}
