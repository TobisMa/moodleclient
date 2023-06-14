use std::{env, fs, path::PathBuf};
use serde::Deserialize;

const CONFIG_FILE_NAME: &'static str = ".moodle.config";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub username: String,
    pub login_url: String,
    pub moodle_url: String,
    pub store: Option<bool>,
    pub user_id: i32
}

pub fn get_config(home: Option<String>) -> Option<Config> {
    let home_folder: PathBuf = PathBuf::from(home.unwrap_or(String::from(env::home_dir().unwrap().to_str().unwrap())));
    println!("{}", home_folder.to_str().unwrap());
    return match fs::read_to_string(home_folder.join(CONFIG_FILE_NAME)) {
        Ok(contents) => Some(
            toml::from_str(&contents).unwrap()
        ),
        Err(_) => None
    };
}
