use std::path::PathBuf;

pub fn app_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(dirs::data_dir)
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("Repeatable")
}

pub fn macros_dir() -> PathBuf {
    app_dir().join("Macros")
}

pub fn config_path() -> PathBuf {
    app_dir().join("config.json")
}

pub fn macro_path(id: &str) -> PathBuf {
    macros_dir().join(format!("{id}.json"))
}

pub fn ensure_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(macros_dir())
}
