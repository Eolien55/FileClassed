use super::lib;

pub fn get_default() -> lib::Config {
    let default: lib::Config = lib::Config {
        dirs: vec![]
            .iter()
            .map(|x : &&str| x.to_string())
            .collect(),
        dest: "".to_string(),
        sleep: 1000,
        codes: []
        .iter()
        .map(|tuple : &(&str, &str) | (tuple.0.to_string(), tuple.1.to_string()))
        .collect(),
        timeinfo: false,
        once: false,
        static_mode: false,
    };

    default
}

pub fn get_build_default() -> lib::BuildConfig {
    let default: lib::Config = get_default();
    
    let build_default: lib::BuildConfig = lib::BuildConfig {
        dirs: Some(default.dirs.iter().cloned().collect()),
        dest: Some(default.dest),
        sleep: Some(default.sleep),
        codes: Some(
            default
                .codes
                .iter()
                .map(|x| (x.0.clone(), x.1.clone()))
                .collect(),
        ),
        timeinfo: default.timeinfo,
        once: default.once,
        static_mode: default.static_mode,
    };

    build_default
}
