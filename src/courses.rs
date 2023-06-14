use std::error::Error;

use reqwest::blocking::Client;

use crate::config::Config;
use serde_json::json;

pub fn get_courses(
    config: &Config,
    client: &Client,
    session_key: &String,
) -> Result<Vec<Course>, Box<dyn Error>> {
    let form_data = json!([
        {
            "index": 0,
            "methodname": "core_course_get_enrolled_courses_by_timeline_classification",
            "args": {
                "offset": 0,
                "limit": 48,
                "classification": "all",
                "sort": "fullname",
                "customfieldname": "",
                "customfieldvalue": ""
            }
        }
    ]);
    let response = client
        .post(
            String::from(&config.moodle_url)
                + "lib/ajax/service.php?sesskey="
                + session_key
                + "&info=core_course_get_recent_courses",
        )
        .json(&form_data)
        .send()?;
    let json: serde_json::Value = serde_json::from_str(&response.text().unwrap())?;
    if json[0]["error"].as_bool().unwrap() {
        return Err("Getting courses threw an error by moodle".into());
    }

    let mut courses = Vec::<Course>::new();
    for course_obj in json[0]["data"]["courses"].as_array().unwrap() {
        courses.push(Course::new(
            course_obj["id"].as_i64().unwrap() as i32,
            String::from(course_obj["fullname"].as_str().unwrap()),
            String::from(course_obj["shortname"].as_str().unwrap()),
            String::from(course_obj["coursecategory"].as_str().unwrap()),
            String::from(course_obj["viewurl"].as_str().unwrap()),
            course_obj["isfavourite"].as_bool().unwrap(),
            if course_obj["hasprogress"].as_bool().unwrap() {
                Some(course_obj["progress"].as_i64().unwrap() as i8)
            } else {
                None
            },
        ));
    }
    dbg!(&courses);
    return Ok(courses);
}

#[derive(Debug)]
pub struct Course {
    pub fullname: String,
    pub shortname: String,
    pub category: String,
    pub progress: Option<i8>,
    pub view_url: String,
    pub is_fav: bool,
    pub id: i32,
}

impl Course {
    fn new(
        id: i32,
        fullname: String,
        shortname: String,
        category: String,
        view_url: String,
        is_fav: bool,
        progress: Option<i8>,
    ) -> Self {
        return Course {
            fullname: fullname,
            shortname: shortname,
            category: category,
            progress: progress,
            view_url: view_url,
            is_fav: is_fav,
            id: id,
        };
    }

    pub fn get_repr(&self) -> String {
        let mut res = String::new();
        if self.is_fav {
            res.push('★');
        } else {
            res.push(' ');
        }
        res.push(' ');
        res.push_str(&self.category);
        res.push_str(" - ");
        res.push_str(&self.fullname);
        return res;
    }
}
