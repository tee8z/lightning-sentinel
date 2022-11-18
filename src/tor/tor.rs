use dirs::config_dir;
use libtor::{
    log::LogDestination, log::LogLevel::Notice, HiddenServiceVersion, Tor, TorAddress, TorFlag,
};
use log::{error, info};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead};
use std::path::Path;

fn build_path_to_tor_log() -> String {
    let root = config_dir().unwrap();
    let the_way = root.to_str().unwrap();
    let true_way = the_way.to_owned();
    true_way + "/tor.log"
}

pub fn clear_old_tor_log() {
    let path = build_path_to_tor_log();
    let _ = OpenOptions::new().write(true).truncate(true).open(path);
}

//blocking
pub fn tor(proxy_port: u16) {
    match Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
        .flag(TorFlag::SocksPort(proxy_port))
        .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
        .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        .flag(TorFlag::LogTo(
            Notice,
            LogDestination::File(build_path_to_tor_log()),
        ))
        .flag(TorFlag::HiddenServicePort(
            TorAddress::Port(8000),
            None.into(),
        ))
        .start()
    {
        Ok(r) => {
            info!("(tor) exit result: {}", r);
        }
        Err(e) => {
            error!("(tor) error: {}", e);
        }
    };
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn parse_keystring() -> bool {
    let re = Regex::new(r"100%").unwrap();
    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines(build_path_to_tor_log()) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
                if re.captures(line.as_str()).is_some() {
                        return true;
                }
        }
    }
    false
}

pub fn watch() -> notify::Result<()> {
    let path = build_path_to_tor_log();
    let file_path = Path::new(&path);
    let (initialtx, initialrx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(initialtx)?;
    //NOTE: wait for log file to be created
    watcher.watch(file_path.parent().unwrap(), RecursiveMode::NonRecursive)?;
    if !file_path.exists() {
        match initialrx.recv() {
                Ok(_) => {
                    info!("(watch) found file");
                }
                Err(e) => {
                    println!("(watch) error: {:?}", e);
                }
        }

        watcher.unwatch(file_path.parent().unwrap())?;
    }
    info!("(watch) starting reading tor's logs, waiting for 100% bootstrapped");
    //NOTE: watch log file for 100% completion
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res| tx.send(res).unwrap())?;
    watcher.watch(file_path, RecursiveMode::NonRecursive)?;
    for res in rx {
        match res {
            Ok(_) => {
                if parse_keystring() {
                    info!("(watch) tor has reach 100% bootstrap, starting clients");
                    break;
                }
            }
            Err(e) => println!("(watch) error: {:?}", e),
        }
    }
    watcher.unwatch(file_path)?;

    Ok(())
}
