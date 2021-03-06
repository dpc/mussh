use STDOUT_SW;
use clap::ArgMatches;
use slog::{DrainExt, Logger};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use toml;

pub const CONFIG_FILE_NAME: &'static str = "mussh.toml";
pub const DOT_DIR: &'static str = ".mussh";
pub const STDOUT_FILE: &'static str = "stdout.log";
pub const STDERR_FILE: &'static str = "stderr.log";

#[derive(Debug, Default, RustcDecodable)]
pub struct MusshToml {
    hostlist: Option<HashMap<String, Hosts>>,
    hosts: Option<HashMap<String, Host>>,
    cmd: Option<HashMap<String, Command>>,
}

#[derive(Debug, Default, RustcDecodable)]
pub struct Hosts {
    hostnames: Vec<String>,
}

#[derive(Debug, Default, RustcDecodable)]
pub struct Host {
    hostname: String,
    pem: Option<String>,
    port: Option<u16>,
    username: String,
    alias: Vec<Alias>,
}

#[derive(Debug, Default, RustcDecodable)]
pub struct Command {
    command: String,
}

#[derive(Debug, Default, RustcDecodable)]
pub struct Alias {
    command: String,
    aliasfor: String,
}

impl MusshToml {
    pub fn new(matches: &ArgMatches) -> MusshToml {
        let mut toml: MusshToml = Default::default();

        for path in &paths(matches.value_of("config")) {
            if let Ok(mut config_file) = File::open(path) {
                let mut toml_buf = vec![];
                if let Ok(_) = config_file.read_to_end(&mut toml_buf) {
                    let toml_str = String::from_utf8_lossy(&toml_buf);
                    if let Some(parsed) = toml::decode_str(&toml_str) {
                        toml = parsed;
                        break;
                    }
                }
            }
        }

        let stdout = Logger::root(STDOUT_SW.drain().fuse(), o!());
        trace!(stdout, "MusshToml::new"; "toml" => format!("{:?}", toml));
        toml
    }

    pub fn hostlist(&self) -> Option<&HashMap<String, Hosts>> {
        match self.hostlist {
            Some(ref h) => Some(h),
            None => None,
        }
    }

    pub fn hosts(&self) -> Option<&HashMap<String, Host>> {
        match self.hosts {
            Some(ref h) => Some(h),
            None => None,
        }
    }

    pub fn cmd(&self) -> Option<&HashMap<String, Command>> {
        match self.cmd {
            Some(ref c) => Some(c),
            None => None,
        }
    }
}

impl Hosts {
    pub fn hostnames(&self) -> &Vec<String> {
        &self.hostnames
    }
}

impl Host {
    pub fn hostname(&self) -> &String {
        &self.hostname
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn pem(&self) -> Option<&String> {
        match self.pem {
            Some(ref p) => Some(p),
            None => None,
        }
    }

    pub fn alias(&self) -> Option<HashMap<String, String>> {
        let mut aliases = HashMap::new();
        for alias in &self.alias {
            aliases.insert(alias.aliasfor().clone(), alias.command().clone());
        }

        if aliases.is_empty() {
            None
        } else {
            Some(aliases)
        }
    }
}

impl Command {
    pub fn command(&self) -> &String {
        &self.command
    }
}

impl Alias {
    pub fn command(&self) -> &String {
        &self.command
    }

    pub fn aliasfor(&self) -> &String {
        &self.aliasfor
    }
}

fn paths(arg: Option<&str>) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(dir) = arg {
        paths.push(PathBuf::from(dir));
    }

    if let Ok(mut cur_dir) = env::current_dir() {
        cur_dir.push(DOT_DIR);
        cur_dir.push(CONFIG_FILE_NAME);
        paths.push(cur_dir);
    }

    if let Some(mut home_dir) = env::home_dir() {
        home_dir.push(DOT_DIR);
        home_dir.push(CONFIG_FILE_NAME);
        paths.push(home_dir);
    }

    add_system_path(&mut paths);
    paths
}

#[cfg(windows)]
fn add_system_path(paths: &mut Vec<PathBuf>) {
    if let Ok(appdata) = env::var("APPDATA") {
        let mut appdata_path = PathBuf::from(appdata);
        appdata_path.push(DOT_DIR);
        appdata_path.push(CONFIG_FILE_NAME);
        paths.push(appdata_path);
    }
}

#[cfg(unix)]
fn add_system_path(paths: &mut Vec<PathBuf>) {
    let mut appdata = PathBuf::from("/etc");
    appdata.push("goopd");
    appdata.push(CONFIG_FILE_NAME);
    paths.push(appdata);
}
