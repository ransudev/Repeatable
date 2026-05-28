use crate::models::{Config, MacroFile};
use crate::storage::paths;
use std::fs;
use std::io;

pub fn load_all() -> Vec<MacroFile> {
    let _ = paths::ensure_dirs();
    let mut macros = Vec::new();

    let Ok(entries) = fs::read_dir(paths::macros_dir()) else {
        return macros;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(macro_file) = serde_json::from_str::<MacroFile>(&text) else {
            continue;
        };

        if macro_file.version == 1 {
            macros.push(macro_file);
        }
    }

    macros.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    macros
}

pub fn save(macro_file: &MacroFile) -> io::Result<()> {
    paths::ensure_dirs()?;
    let json = serde_json::to_string_pretty(macro_file).map_err(io::Error::other)?;
    fs::write(paths::macro_path(&macro_file.id), json)
}

pub fn delete(id: &str) -> io::Result<()> {
    paths::ensure_dirs()?;
    let path = paths::macro_path(id);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn load_config() -> Config {
    let _ = paths::ensure_dirs();
    let Ok(text) = fs::read_to_string(paths::config_path()) else {
        return Config::default();
    };
    serde_json::from_str::<Config>(&text).unwrap_or_default()
}

pub fn save_config(config: &Config) -> io::Result<()> {
    paths::ensure_dirs()?;
    let json = serde_json::to_string_pretty(config).map_err(io::Error::other)?;
    fs::write(paths::config_path(), json)
}
