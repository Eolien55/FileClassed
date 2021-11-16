use chrono::{offset::TimeZone, Local, NaiveDateTime};
use locale::Time;
use rayon::prelude::*;
use scan_dir::ScanDir;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread::sleep;
use std::time;

use crate::conf::lib;
use crate::conf::lib::{Config, DeclaredType};

macro_rules! decode {
    ($code:expr, $codes:expr) => {
        $codes.get($code).unwrap_or($code)
    };
}

#[inline]
pub fn find_first_valid_opening_bracket(input: &str) -> Option<usize> {
    let result: Option<usize>;
    let mut offset = 0;
    let mut mut_input = input;

    loop {
        match mut_input.find('{') {
            Some(naive_first) => match mut_input[naive_first + 1..].find('}') {
                Some(naive_first_closing_after_first) => match mut_input[naive_first + 1..]
                    .find('{')
                {
                    Some(naive_next) => {
                        let naive_next = naive_next + naive_first + 1;
                        if naive_first < naive_next && naive_next < naive_first_closing_after_first
                        {
                            offset += naive_next;
                            mut_input = &mut_input[naive_next..];
                            continue;
                        } else {
                            result = Some(naive_first + offset);
                            break;
                        }
                    }
                    None => {
                        result = Some(naive_first + offset);
                        break;
                    }
                },
                None => {
                    result = None;
                    break;
                }
            },
            None => {
                result = None;
                break;
            }
        }
    }

    result
}

pub fn expand(input: &str, codes: &HashMap<String, String>) -> String {
    if let Some(mut next_seq_beg) = find_first_valid_opening_bracket(input) {
        let mut result = String::with_capacity(input.len());

        let mut input_str = input;

        loop {
            result.push_str(&input_str[..next_seq_beg]);

            input_str = &input_str[next_seq_beg..];
            if input_str.is_empty() {
                break;
            }

            next_seq_beg = match input_str.find('}') {
                Some(res) => {
                    let code = &input_str[1..res];
                    result.push_str(decode!(&code.to_string(), codes));
                    res + 1
                }
                None => next_seq_beg + 1,
            };

            input_str = &input_str[next_seq_beg..];
            next_seq_beg = find_first_valid_opening_bracket(input_str).unwrap_or(input_str.len());
        }
        result
    } else {
        input.to_owned()
    }
}

pub fn get_new_name(
    name: &str,
    dest: &path::Path,
    codes: &HashMap<String, String>,
    timestamp: Option<time::SystemTime>,
    timeinfo: bool,
    separator: char,
    filename_separators: usize,
) -> Result<(path::PathBuf, path::PathBuf), Box<dyn Error>> {
    let mut year: String = "".to_string();
    let month_nb: usize;
    let mut month_str: String = "".to_string();

    if timeinfo {
        let timestamp = timestamp
            .unwrap()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs();

        let datetime = Local
            .from_utc_datetime(&NaiveDateTime::from_timestamp(timestamp as i64, 0))
            .format("%Y %m")
            .to_string();

        year = datetime
            .chars()
            .skip(0)
            .take(datetime.find(' ').unwrap())
            .collect();
        month_nb = datetime
            .chars()
            .skip(datetime.find(' ').unwrap() + 1)
            .collect::<String>()
            .parse::<usize>()?
            - 1;

        month_str = Time::load_user_locale()?.long_month_name(month_nb);

        if let Some(r) = month_str.get_mut(0..1) {
            r.make_ascii_uppercase();
        }
    }

    let mut ending_path: path::PathBuf = path::PathBuf::new();
    ending_path.push(dest);
    if timeinfo {
        ending_path.push(year);
    }

    let mut next: &str = name;
    let mut splitted: (&str, &str) = ("", "");
    while next.matches(separator).count() > filename_separators {
        splitted = next.split_at(next.find(separator).unwrap() + 1);
        let current = splitted.0;
        let mut current: String = current[..current.len() - 1].to_string();
        next = splitted.1;

        while find_first_valid_opening_bracket(&current).is_some() {
            current = expand(&current, codes);
        }

        ending_path.push(decode!(&current, codes));
    }

    if timeinfo {
        ending_path.push(month_str);
    }

    let dir = ending_path.clone();
    ending_path.push(splitted.1);

    Ok((ending_path, dir))
}

fn handle(
    name: &path::Path,
    dest: &path::Path,
    codes: &HashMap<String, String>,
    timeinfo: bool,
    separator: char,
    filename_separators: usize,
) {
    if !path::Path::new(name.to_str().unwrap()).exists() {
        log::warn!(
            "File `{}` disappeared before I could handle it !",
            name.to_str().unwrap_or("ERROR WHEN DISPLAYING THE FILE")
        );
        return;
    }

    let timestamp: Option<time::SystemTime> = if timeinfo {
        Some(fs::metadata(&name).unwrap().created().unwrap())
    } else {
        None
    };

    match get_new_name(
        name.file_name().unwrap().to_str().unwrap(),
        dest,
        codes,
        timestamp,
        timeinfo,
        separator,
        filename_separators,
    ) {
        Ok(result) => match fs::create_dir_all(&result.1) {
            Ok(_) => match fs::rename(&name, &result.0) {
                Ok(_) => log::info!("Moved path from {:?} to {:?}", name, result.0),
                Err(_) => {
                    log::warn!("File `{:?}` disappeared before I could handle it !", name)
                }
            },
            Err(_) => log::warn!("File `{:?}` disappeared before I could handle it !", name),
        },

        Err(e) => log::error!("Error happened with file {:?} : {}", name, e.to_string()),
    }
}

fn make_tables(codes: &HashMap<String, String>, dest: &str) {
    if path::Path::new(&format!("{}{}shortcuts", dest, path::MAIN_SEPARATOR)).exists() {
        fs::remove_file(format!("{}{}shortcuts", dest, path::MAIN_SEPARATOR)).unwrap();
    }

    if path::Path::new(&format!("{}{}fcs-should_end", dest, path::MAIN_SEPARATOR)).exists() {
        fs::remove_file(&format!("{}{}fcs-should_end", dest, path::MAIN_SEPARATOR)).unwrap();
    }

    let mut shortcuts_file =
        fs::File::create(format!("{}{}shortcuts", dest, path::MAIN_SEPARATOR)).unwrap();

    let mut shortcuts = String::new();
    for (key, value) in codes {
        shortcuts += &format!("\t{} = {}\n", key, value);
    }

    shortcuts_file.write_all(shortcuts.as_bytes()).unwrap();
    log::debug!("Codes are : \n{}", shortcuts);
}

pub fn run(mut my_config: Config, declared: DeclaredType, mut config_file: String) {
    log::trace!("Creating tables");
    make_tables(&my_config.codes, my_config.dest.to_str().unwrap());

    // Note : the <variable>_s is to read : "shared <variable>"
    let should_end = Arc::new(AtomicBool::new(false));
    let should_end_s = should_end.clone();

    let config_changed = Arc::new(AtomicBool::new(false));
    let config_changed_s = config_changed.clone();

    let config_file_s = config_file.clone();
    let mut background_thread: Option<std::thread::JoinHandle<()>> = None;
    let (tx, rx) = mpsc::channel::<bool>();

    let handle_for_real_handle = |path: &path::Path, my_config: &lib::Config| -> Result<(), ()> {
        if should_end.load(Ordering::SeqCst) {
            log::trace!("I'm supposed to end while handling files");
            return Err(());
        }

        if !lib::test_path!(&my_config.dest, "dir") {
            log::error!(
                "Destination `{:#?}` doesn't exist anymore ! Exiting !",
                my_config.dest
            );
            return Err(());
        }

        handle(
            path,
            &my_config.dest,
            &my_config.codes,
            my_config.timeinfo,
            my_config.separator,
            my_config.filename_separators,
        );

        Ok(())
    };

    if path::Path::new(&config_file).exists() && !my_config.once && !my_config.static_mode {
        log::trace!("Setting up config watcher");

        let should_end_s_s = should_end_s.clone();
        let dest_s = my_config.dest.clone();

        background_thread = Some(std::thread::spawn(move || {
            let mut old_last_change = time::SystemTime::now();

            let mut should_run = true;
            loop {
                let opened_config_file = fs::File::open(&config_file_s);

                match opened_config_file {
                    Ok(result) => match result.metadata() {
                        Ok(res) => match res.modified() {
                            Ok(r) => {
                                old_last_change = r;
                                break;
                            }
                            Err(e) => log::warn!(
                            "Unable to get modified field of config file. Still running. Note : {}",
                            e.to_string()
                        ),
                        },
                        Err(e) => log::warn!(
                            "Unable to get metadata of config file. Still running. Note : {}",
                            e.to_string()
                        ),
                    },
                    Err(_) => log::warn!(
                        "Config file `{}` doesn't seem to exist anymore. Child exiting",
                        config_file_s
                    ),
                }

                match rx.recv() {
                    Ok(mesg) => {
                        if mesg {
                            should_run = false;
                            break;
                        }
                    }
                    Err(e) => {
                        log::error!("Critical ! I'm disconnected from my parent process ! Exiting ! Note : {}", e.to_string());
                        should_run = false;
                        break;
                    }
                }
            }

            if should_run {
                'main_loop: loop {
                    match rx.recv() {
                        Ok(mesg) => {
                            if mesg {
                                break 'main_loop;
                            }
                        }
                        Err(e) => {
                            log::error!("Critical ! I'm disconnected from my parent process ! Exiting ! Note : {}", e.to_string());
                            break 'main_loop;
                        }
                    }

                    let opened_config_file = fs::File::open(&config_file_s);

                    match opened_config_file {
                        Ok(file) => {
                            let new_last_change = file.metadata().unwrap().modified().unwrap();

                            if old_last_change < new_last_change {
                                config_changed_s.store(true, Ordering::SeqCst);
                                old_last_change = new_last_change;
                            };
                        }

                        Err(_) => {
                            log::warn!(
                                "Config file `{}` doesn't exist anymore ! Can't use it",
                                config_file_s
                            );
                        }
                    }

                    if path::Path::new(&format!(
                        "{}{}fcs-should_end",
                        dest_s.to_str().unwrap(),
                        path::MAIN_SEPARATOR
                    ))
                    .exists()
                    {
                        should_end_s_s.store(true, Ordering::SeqCst);
                        fs::remove_file(&format!(
                            "{}{}fcs-should_end",
                            dest_s.to_str().unwrap(),
                            path::MAIN_SEPARATOR
                        ))
                        .unwrap();
                        break 'main_loop;
                    }
                }
            }
        }));
    }

    log::trace!("Setting up CTRL+C handler");
    ctrlc::set_handler(move || {
        println!("Received CTRL+C, ending.");
        should_end_s.store(true, Ordering::SeqCst)
    })
    .unwrap();

    let mut dirs = my_config.dirs.clone();

    log::trace!("Starting my job");
    'outer: while !should_end.load(Ordering::SeqCst) {
        for dir in &dirs {
            if !lib::test_path!(&dir, "dir") {
                break;
            }

            let files: Vec<path::PathBuf> = ScanDir::files()
                .walk(dir, |iter| {
                    iter.filter(|&(_, ref name)| {
                        name.matches(my_config.separator).count() > my_config.filename_separators
                    })
                    .map(|(entry, _)| entry.path())
                    .collect()
                })
                .unwrap();

            if !lib::test_path!(&my_config.dest, "dir") {
                log::error!(
                    "Destination `{:#?}` doesn't exist anymore ! Exiting !",
                    my_config.dest
                );
                break 'outer;
            }

            if should_end.load(Ordering::SeqCst) {
                break 'outer;
            }

            let error_hapenned: bool = files
                .par_iter()
                .map(|entry| handle_for_real_handle(&entry.to_owned(), &my_config))
                .any(|res| res.is_err());

            if error_hapenned {
                break 'outer;
            }
        }

        if my_config.once {
            break 'outer;
        }

        // Beyond this, users running with -o won't ever have to suffer the wait
        // of sending info to the other thread or to reload a configuration file,
        // or even worse, just sleeping
        sleep(time::Duration::from_millis(my_config.sleep as u64));

        if !my_config.static_mode {
            if config_changed.load(Ordering::SeqCst) {
                log::info!("Config changed ! Loading it");

                config_changed.store(false, Ordering::SeqCst);
                my_config.add_or_update_from_file(&mut config_file, &declared);

                if my_config.clean() {
                    should_end.store(true, Ordering::SeqCst);
                }

                make_tables(&my_config.codes, my_config.dest.to_str().unwrap());
            }

            match tx.send(false) {
                Ok(_) => (),
                Err(e) => {
                    log::error!(
                        "Critical ! I'm disconnected from my child ! Exiting ! Note {}",
                        e.to_string()
                    );
                    should_end.store(true, Ordering::SeqCst);
                    break 'outer;
                }
            }
        }

        let non_existing_dirs: HashSet<path::PathBuf> = my_config
            .dirs
            .iter()
            .filter(|dir| !lib::test_path!(&dir, "dir"))
            .map(|dir| dir.to_owned())
            .collect();
        for dir in &non_existing_dirs {
            log::warn!(
                "Watching directory `{:#?}` doesn't exist anymore ! Not using it",
                dir
            );
        }

        dirs = my_config
            .dirs
            .difference(&non_existing_dirs)
            .map(|dir| dir.to_owned())
            .collect();
        if my_config.dirs.is_empty() {
            log::error!("No directory available anymore ! Exiting");
            break 'outer;
        }

        if !lib::test_path!(&my_config.dest, "dir") {
            log::error!(
                "Destination `{:#?}` doesn't exist anymore ! Exiting !",
                my_config.dest
            );
            break 'outer;
        }
    }

    if let Some(thread) = background_thread {
        log::trace!("Waiting for my watching child to end");
        match tx.send(true) {
            Ok(_) => (),
            Err(_) => {
                log::warn!("I am disconnected from my child, but I was ending anyway so...");
            }
        }
        thread.join().ok();
    }
}
