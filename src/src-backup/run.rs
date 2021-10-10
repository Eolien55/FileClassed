use std::collections::{HashMap};
use std::fs;
use std::path;
use scan_dir::ScanDir;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread::sleep_ms;
use crate::config;

fn handle(name: path::PathBuf, dest: &String, codes: &HashMap<String, String>, month_codes: &HashMap<String, String>) {
    if !path::Path::new(&name).exists() {
        return;
    }

    println!("Handling file {:?}", name);

    let timestamp = fs::metadata(name).unwrap().created();
}

pub fn run(my_config : config::Config) -> (u16,) {
    let should_end = Arc::new(AtomicBool::new(false));
    let s = should_end.clone();
    
    ctrlc::set_handler(move || {
        println!("Received CTRL+C, ending.");
        s.store(true, Ordering::SeqCst)
    }).expect("??");

    while !should_end.load(Ordering::SeqCst) {
        let _ : Vec<Vec<_>> = 
            my_config.dirs.iter().map(|dir| ScanDir::files().walk(dir, |iter| {
                iter.filter(|&(_, ref name)| 
                    name.matches('.').count() > 1 && 
                    my_config.codes.contains_key::<String>(
                        &name.chars()
                             .skip(0)
                             .take(
                                 name.find('.').unwrap()
                                )
                             .collect())).map(
                                 |(entry, _)| handle(
                                     entry.path(), &my_config.dest, &my_config.codes, &my_config.months
                                    ) 
                   ).collect()
            }).unwrap()).collect();

            if my_config.once {
                break;
            }
            sleep_ms(my_config.sleep);
    }

    return (0,);
}