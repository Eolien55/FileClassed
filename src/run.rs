use scan_dir::ScanDir;
use chrono::{offset::TimeZone, Local, NaiveDateTime};
use locale::Time;

use std::collections::{HashMap};
use std::path;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread::sleep;
use std::time;
use std::fs;
use std::error::Error;

use crate::main_config::{clean};
use crate::conf::file::get_config;
use crate::conf::lib::{Config, DeclaredType};


fn get_new_name(
    name: &path::PathBuf,
    dest: &String,
    codes: &HashMap<String, String>,
    timestamp : time::SystemTime
) -> 
// New path and directory
Result<(path::PathBuf, path::PathBuf), Box<dyn Error>> {
    let timestamp = timestamp.duration_since(time::UNIX_EPOCH)?
                             .as_secs();

    let datetime = Local.from_utc_datetime(
        &NaiveDateTime::from_timestamp(timestamp as i64, 0)
    ).format("%Y %m").to_string();

    let year : String = datetime.chars()
                                .skip(0)
                                .take(datetime.find(' ').unwrap())
                                .collect();
    let month : usize = datetime.chars()
                                .skip(datetime.find(' ').unwrap() + 1)
                                .collect::<String>()
                                .parse::<usize>()
                                .unwrap() - 1;

    let mut month = Time::load_user_locale()?.long_month_name(month);

    if let Some(r) = month.get_mut(0..1) {
        r.make_ascii_uppercase();
    }

    let mut ending_path : path::PathBuf = path::PathBuf::new();
    ending_path.push(dest);
    ending_path.push(year);

    let string_name : &str = &name.to_str()
                                  .unwrap()
                                  .chars()
                                  .skip(
                                      name.to_str()
                                          .unwrap()
                                          .rfind(path::MAIN_SEPARATOR)
                                          .unwrap() + 1
                                    )
                                  .collect::<String>();
    
    let mut next = string_name;
    let mut splitted : (&str, &str) = ("", "");
    while next.matches('.').count() > 1 {
        splitted = next.split_at(next.find('.').unwrap() + 1);
        let current = splitted.0;
        let current : &str = &current.chars()
                             .take(current.chars().count() - 1)
                             .collect::<String>();
        next = splitted.1;
        
        if codes.contains_key(current) {
            ending_path.push(&codes[current]);
        } else {
            ending_path.push(current);
        }
    }

    ending_path.push(month);
    let dir = ending_path.clone();
    ending_path.push(splitted.1);

    return Ok((ending_path, dir));
}

fn handle(name: path::PathBuf, dest: &String, codes: &HashMap<String, String>) {
    if !path::Path::new(name.to_str().unwrap()).exists() {
        ()
    }

    let timestamp = fs::metadata(&name).unwrap().created().unwrap();

    match get_new_name(&name, dest, codes, timestamp) {
        Ok(result) => {
            fs::create_dir_all(&result.1).unwrap();
            fs::rename(&name, &result.0).unwrap();

            log::info!("Moved path from {:?} to {:?}", name, result.0)
        },
        
        Err(e) => log::error!("Error happened with file {:?} : {}", name, e.to_string())
    }
}

fn make_tables(codes : &HashMap<String, String>, dest : &String) {
    if path::Path::new(&format!(
        "{}{}Tables", dest, path::MAIN_SEPARATOR
    )).exists() {
        fs::remove_dir_all(format!(
            "{}{}Tables", 
            dest, path::MAIN_SEPARATOR
        )).unwrap();
    }

    for (key, value) in codes {
        log::trace!(
            "Creating dir \"{}{}Tables{}{} = {}\"",
            dest, path::MAIN_SEPARATOR, path::MAIN_SEPARATOR,
            key, value
        );
        fs::create_dir_all(
            format!(
                "{}{}Tables{}{} = {}", 
                dest, path::MAIN_SEPARATOR, path::MAIN_SEPARATOR,
                key, value
            )
        ).unwrap();
    }
}

pub fn run(mut my_config : Config, declared : DeclaredType, mut config_file : String) {
    // Note : the <variable>_s is to read : "shared <variable>"
    let should_end = Arc::new(AtomicBool::new(false));
    let should_end_s = should_end.clone();
    let should_end_s_s = should_end_s.clone();

    let config_changed = Arc::new(AtomicBool::new(false));
    let config_changed_s = config_changed.clone();
    
    let config_file_s = config_file.clone();
    let mut background_thread : Option<std::thread::JoinHandle<()>> = None;

    if path::Path::new(&config_file).exists() && !my_config.once {
        log::trace!("Setting up config watcher");
        
        background_thread = Some(std::thread::spawn(move || {
            let mut opened_config_file : std::io::Result<fs::File>;
            opened_config_file = fs::File::open(&config_file_s);
            let mut old_last_change = opened_config_file.unwrap().metadata().unwrap().modified().unwrap();
            loop {
                opened_config_file = fs::File::open(&config_file_s);
                
                if opened_config_file.is_err() {
                    log::warn!("Config file {} doesn't exist anymore ! Can't use it", config_file_s);
                } else {
                    let new_last_change = opened_config_file.unwrap().metadata().unwrap().modified().unwrap();
                    if old_last_change < new_last_change {
                        config_changed_s.store(true, Ordering::SeqCst);
                        old_last_change = new_last_change;
                    };
                }

                if should_end_s_s.load(Ordering::SeqCst) {
                    break;
                }

                sleep(time::Duration::from_secs(2));
            }
        }));
    }
    
    log::trace!("Setting up CTRL+C handler");
    ctrlc::set_handler(move || {
        println!("Received CTRL+C, ending.");
        should_end_s.store(true, Ordering::SeqCst)
    }).unwrap();
    
    log::trace!("Creating tables");
    make_tables(&my_config.codes, &my_config.dest);

    log::trace!("Starting my job");
    'outer : while !should_end.load(Ordering::SeqCst) {
        for dir in &my_config.dirs {
            let files : Vec<fs::DirEntry> = ScanDir::files().walk(dir, |iter| {
                iter.filter(|&(_, ref name)| {
                    name.matches('.').count() > 1 && 
                            my_config.codes.contains_key::<String>(
                                &name.chars()
                                     .take(
                                         name.find('.').unwrap()
                                        )
                                     .collect())
                }).map(|(entry, _)| entry)
                  .collect()
            }).unwrap();
            
            for entry in files {
                if should_end.load(Ordering::SeqCst) {
                    break 'outer;
                }

                handle(
                    entry.path(), &my_config.dest, &my_config.codes
                );
            }

            if should_end.load(Ordering::SeqCst) {
                break 'outer;
            }
        }

        if my_config.once {
            break;
        }

        sleep(time::Duration::from_millis(my_config.sleep as u64));

        if config_changed.load(Ordering::SeqCst) {
            log::info!("Config changed ! Loading it");

            config_changed.store(false, Ordering::SeqCst);
            get_config(&mut my_config, &mut config_file, &declared);

            if clean(&mut my_config) {
                should_end.store(true, Ordering::SeqCst);
            }

            make_tables(&my_config.codes, &my_config.dest);
        }
    }

    if !background_thread.is_none() {
        log::trace!("Waiting for my watching child to end");
        background_thread.unwrap().join().unwrap();
    }
}
