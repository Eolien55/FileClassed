use std::collections::{HashMap};
use std::path;
use scan_dir::ScanDir;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread::sleep;
use std::time;
use chrono::NaiveDateTime;
use locale::Time;
use std::fs;
use crate::config;

fn get_new_name(
    name: &path::PathBuf,
    dest: &String,
    codes: &HashMap<String, String>,
    timestamp : time::SystemTime
) -> 
// New path and directory
(path::PathBuf, path::PathBuf) {
    let timestamp = timestamp.duration_since(time::UNIX_EPOCH)
                             .unwrap()
                             .as_secs();

    let datetime = NaiveDateTime::from_timestamp(timestamp as i64, 0)
                                 .format("%Y %m").to_string();

    let year : String = datetime.chars()
                                .skip(0)
                                .take(datetime.find(' ').unwrap())
                                .collect();
    let month : usize = datetime.chars()
                                .skip(datetime.find(' ').unwrap() + 1)
                                .collect::<String>()
                                .parse::<usize>()
                                .unwrap() - 1;

    let mut month = Time::load_user_locale().unwrap().long_month_name(month);

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

    (ending_path, dir)
}

fn handle(name: path::PathBuf, dest: &String, codes: &HashMap<String, String>) {
    if !path::Path::new(name.to_str().unwrap()).exists() {
        ()
    }

    let timestamp = fs::metadata(&name).unwrap().created().unwrap();

    let result = get_new_name(&name, dest, codes, timestamp);
    
    fs::create_dir_all(result.1).unwrap();
    fs::rename(name, result.0).unwrap();
}

pub fn run(my_config : config::Config) {
    let should_end = Arc::new(AtomicBool::new(false));
    let s = should_end.clone();
    
    ctrlc::set_handler(move || {
        println!("Received CTRL+C, ending.");
        s.store(true, Ordering::SeqCst)
    }).expect("??");

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
    }
}