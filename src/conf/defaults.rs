use super::lib;

pub fn get_default() -> lib::Config {
    let default: lib::Config = lib::Config {
        dirs: vec!["~/Scolaire", "~/usb"]
            .iter()
            .map(|x| x.to_string())
            .collect(),
        dest: "~/Scolaire".to_string(),
        sleep: 1000,
        codes: [
            ("chin", "Chinois"),
            ("en", "Anglais"),
            ("eps", "EPS"),
            ("fr", "Français"),
            ("glb", "Global"),
            ("gr", "Grec"),
            ("hg", "Histoire-Géographie"),
            ("info", "Informatique"),
            ("mt", "Mathématiques"),
            ("pc", "Physique-Chimie"),
            ("ses", "Sciences Économiques et Sociales"),
            ("svt", "SVT"),
            ("vdc", "Vie de Classe"),
        ]
        .iter()
        .map(|tuple| (tuple.0.to_string(), tuple.1.to_string()))
        .collect(),
        timeinfo: false,
        once: false,
        static_mode: false,
    };

    default
}

pub fn get_defaults() -> lib::BuildConfig {
    let default_pre: lib::Config = lib::Config {
        dirs: vec!["~/Scolaire", "~/usb"]
            .iter()
            .map(|x| x.to_string())
            .collect(),
        dest: "~/Scolaire".to_string(),
        sleep: 1000,
        codes: [
            ("chin", "Chinois"),
            ("en", "Anglais"),
            ("eps", "EPS"),
            ("fr", "Français"),
            ("glb", "Global"),
            ("gr", "Grec"),
            ("hg", "Histoire-Géographie"),
            ("info", "Informatique"),
            ("mt", "Mathématiques"),
            ("pc", "Physique-Chimie"),
            ("ses", "Sciences Économiques et Sociales"),
            ("svt", "SVT"),
            ("vdc", "Vie de Classe"),
        ]
        .iter()
        .map(|tuple| (tuple.0.to_string(), tuple.1.to_string()))
        .collect(),
        timeinfo: false,
        once: false,
        static_mode: false,
    };

    let default = default_pre.clone();

    let build_default: lib::BuildConfig = lib::BuildConfig {
        dirs: Some(default.dirs.iter().map(|x| x.clone()).collect()),
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
