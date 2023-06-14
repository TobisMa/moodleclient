mod arguments;
mod authenticate;
mod config;
mod courses;

use arguments::Arguments;
use authenticate::{login, logout};
use clap::Parser;
use config::{get_config, Config};

use dialoguer::console::set_colors_enabled;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use reqwest::blocking::Client;

use crate::courses::get_courses;

fn main() {
    let terminal_args: Arguments = Arguments::parse();
    let mut config: Config = match get_config(terminal_args.home_dir) {
        Some(cnf) => cnf,
        None => {
            eprintln!("Could not read config file in home folder or at specified home location");
            return;
        }
    };

    if !config.moodle_url.ends_with("/") {
        config.moodle_url = config.moodle_url + "/";
    }

    set_colors_enabled(true);
    let client: Client = Client::builder().cookie_store(true).build().unwrap();
    let session_key = match login(&config, &client) {
        Some(value) => value,
        None => {
            eprintln!("Failed to login");
            return;
        }
    };

    let courses = match get_courses(&config, &client, &session_key) {
        Ok(courses) => courses,
        Err(_) => {
            eprintln!("Error collection courses");
            return;
        }
    };

    let select_course_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please select your course:")
        .default(0)
        .items(
            &courses
                .iter()
                .map(|course| course.get_repr())
                .collect::<Vec<_>>(),
        )
        .interact()
        .unwrap();

    println!("{}", &courses[select_course_index].view_url);

    logout(&config, &client);
    dbg!(client
        .get(&config.moodle_url)
        .send()
        .unwrap()
        .url()
        .as_str());
}
