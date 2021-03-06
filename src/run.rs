use chrono::{offset::TimeZone, Local, NaiveDateTime};
use locale::Time;
use rayon::prelude::*;
use scan_dir::ScanDir;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time;

use crate::conf::lib;
use crate::conf::lib::{Config, DeclaredType};

#[inline]
pub fn expand_last(code: &String, last: &Vec<String>, last_token: char) -> String {
    let mut length = last.len();
    let mut code_tmp = code.to_owned();
    let mut result: String = code.to_owned();

    while length > 0 {
        result = "".to_string();
        let scheme_to_find: String = (0..length).map(|_| last_token).collect();

        while let Some(scheme_index) = code_tmp.find(&scheme_to_find) {
            result.push_str(&code_tmp[..scheme_index]);

            result.push_str(&last[length - 1]);

            code_tmp = code_tmp[scheme_index + length - 1..].to_string();

            if code_tmp.len() == 0 {
                break;
            }

            code_tmp = code_tmp[1..].to_string();
        }

        result.push_str(&code_tmp);
        code_tmp = result.clone();

        length -= 1;
    }

    result
}

#[inline]
pub fn decode(code: &String, codes: &HashMap<String, String>) -> String {
    codes.get(code).unwrap_or(code).to_owned()
}

#[inline]
pub fn find_first_valid_opening_bracket(
    input: &str,
    begin_var: char,
    end_var: char,
) -> Option<usize> {
    let result: Option<usize>;
    let mut offset = 0;
    let mut mut_input = input;

    loop {
        match mut_input.find(begin_var) {
            Some(naive_first) => match mut_input[naive_first + 1..].find(end_var) {
                Some(naive_first_closing_after_first) => match mut_input[naive_first + 1..]
                    .find(begin_var)
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

// fvop stands for First Valid Opening Bracket
#[inline]
pub fn expand(
    input: &str,
    codes: &HashMap<String, String>,
    begin_var: char,
    end_var: char,
    fvob: Option<usize>,
    last: &mut Vec<String>,
    last_token: char,
) -> String {
    if let Some(mut next_seq_beg) =
        fvob.or_else(|| find_first_valid_opening_bracket(input, begin_var, end_var))
    {
        let mut result = String::with_capacity(input.len());

        let mut input_str = input;

        loop {
            result.push_str(&input_str[..next_seq_beg]);

            input_str = &input_str[next_seq_beg..];
            if input_str.is_empty() {
                break;
            }

            next_seq_beg = match input_str.find(end_var) {
                Some(res) => {
                    let code = &input_str[1..res].to_string();
                    let code = expand_last(code, last, last_token);
                    result.push_str(&decode(&code, codes));
                    last.push(code.clone());
                    res + 1
                }
                None => next_seq_beg + 1,
            };

            input_str = &input_str[next_seq_beg..];
            next_seq_beg = find_first_valid_opening_bracket(input_str, begin_var, end_var)
                .unwrap_or(input_str.len());
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
    separator: (char, usize),
    var: (char, char),
    last_token: char,
) -> Result<(path::PathBuf, path::PathBuf), Box<dyn Error>> {
    let mut year: String = "".to_string();
    let month_nb: usize;
    let mut month_str: String = "".to_string();

    let timeinfo = timestamp.is_some();
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
    let mut last = vec![];
    let mut current: String;
    while next.matches(separator.0).count() > separator.1 {
        splitted = next.split_at(next.find(separator.0).unwrap() + 1);
        let current_str = splitted.0;
        current = current_str[..current_str.len() - 1].to_string();
        next = splitted.1;

        let mut should_be_decoded = true;

        while let Some(fvob) = find_first_valid_opening_bracket(&current, var.0, var.1) {
            current = expand(
                &current,
                codes,
                var.0,
                var.1,
                Some(fvob),
                &mut last,
                last_token,
            );
            should_be_decoded = false;
        }

        if should_be_decoded {
            ending_path.push(decode(&expand_last(&current, &last, last_token), codes));
        } else {
            ending_path.push(current.clone());
        }

        last.push(current);
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
    separator: (char, usize),
    var: (char, char),
    last_token: char,
) {
    if !path::Path::new(name.to_str().unwrap()).exists() {
        log::warn!(
            "File `{:#}` disappeared before I could handle it !",
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
        separator,
        var,
        last_token,
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

static OPERATING: AtomicBool = AtomicBool::new(false);
static SHOULD_STOP_PROCESSING: AtomicBool = AtomicBool::new(false);

pub fn run(mut my_config: Config, declared: DeclaredType, mut config_file: String) {
    log::trace!("Creating tables");
    make_tables(&my_config.codes, my_config.dest.to_str().unwrap());

    let handle_for_real_handle = |path: &path::Path, my_config: &lib::Config| -> Result<(), ()> {
        if SHOULD_STOP_PROCESSING.load(Ordering::SeqCst) {
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
            (my_config.separator, my_config.filename_separators),
            (my_config.begin_var, my_config.end_var),
            my_config.last_token,
        );

        Ok(())
    };

    let cleanup = || {
        SHOULD_STOP_PROCESSING.store(true, Ordering::SeqCst);
        while OPERATING.load(Ordering::SeqCst) {}

        log::info!("Goodbye");
        std::process::exit(exitcode::OK);
    };

    log::trace!("Setting up CTRL+C handler");
    ctrlc::set_handler(move || {
        println!("Received CTRL+C, ending.");

        cleanup();
    })
    .unwrap();

    let mut dirs = my_config.dirs.clone();

    let mut old_last_change = time::SystemTime::now();

    log::trace!("Starting my job");
    'outer: loop {
        OPERATING.store(true, Ordering::SeqCst);

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
        OPERATING.store(false, Ordering::SeqCst);
        sleep(time::Duration::from_millis(my_config.sleep as u64));
        if SHOULD_STOP_PROCESSING.load(Ordering::SeqCst) {
            break 'outer;
        }

        if !my_config.static_mode {
            let opened_config_file = fs::File::open(&config_file);

            match opened_config_file {
                Ok(file) => {
                    let new_last_change = file.metadata().unwrap().modified().unwrap();

                    if old_last_change < new_last_change {
                        log::info!("Config changed ! Loading it");

                        my_config.add_or_update_from_file(&mut config_file, &declared);

                        if my_config.clean(true) {
                            SHOULD_STOP_PROCESSING.store(true, Ordering::SeqCst);
                        }

                        make_tables(&my_config.codes, my_config.dest.to_str().unwrap());
                        old_last_change = new_last_change;
                    };
                }

                Err(_) => {
                    log::warn!(
                        "Config file `{:#}` doesn't exist anymore ! Can't use it",
                        config_file
                    );
                }
            }
        }

        let non_existing_dirs: HashSet<path::PathBuf> = my_config
            .dirs
            .par_iter()
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

    if SHOULD_STOP_PROCESSING.load(Ordering::SeqCst) {
        loop {
            sleep(time::Duration::from_secs(100));
        }
    }
}
