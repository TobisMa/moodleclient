use dialoguer::{theme::ColorfulTheme, Confirm, Password};
use keyring::Entry;
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use crate::config::Config;

pub fn login(config: &Config, client: &Client) -> Option<String> {
    let password: String = get_password(&config.username, &config.store);
    println!("Starting authentication");
    let login_get = client.get(&config.login_url).send();

    // retrieving token
    let mut html = match login_get {
        Ok(response) => {
            // verifying login is still necessary
            let url_str = response.url().as_str();
            if !url_str.ends_with("login") {
                return get_session_key(&config, &client);
            }
            Html::parse_document(response.text().unwrap().as_str())
        }
        Err(_) => {
            eprintln!("Something went wrong getting {}", config.login_url);
            return None;
        }
    };

    let token: &str = html
        .select(&Selector::parse(r#"input[name="_token"]"#).unwrap())
        .next()
        .unwrap()
        .value()
        .attr("value")
        .unwrap();
    println!("Token: {}", token);

    // login
    let form_data = [
        ("rpIdmPrimaryPrincipalName", &config.username),
        ("password", &password),
        ("_token", &String::from(token)),
    ];
    let login_post = client
        .post(config.login_url.clone())
        .form(&form_data)
        .send();

    if login_post.is_err() {
        eprintln!("Something went wrong on login");
        return None;
    }

    // verfiy non-js-environment
    let login_response = login_post.unwrap();
    dbg!(login_response.url().as_str());

    let verify_page = client.get(&config.moodle_url).send();
    if verify_page.is_err() {
        eprintln!("Verfiying non-js environment failed");
        return None;
    }
    let verfiy_response = verify_page.unwrap();
    dbg!(verfiy_response.url().as_str());

    html = Html::parse_document(verfiy_response.text().unwrap().as_str());
    let saml_response = html
        .select(&Selector::parse(r#"input[name="SAMLResponse"]"#).unwrap())
        .next()
        .unwrap()
        .value()
        .attr("value")
        .unwrap();
    let relay_state = html
        .select(&Selector::parse(r#"input[name="RelayState"]"#).unwrap())
        .next()
        .unwrap()
        .value()
        .attr("value")
        .unwrap();
    let form_url = html
        .select(&Selector::parse(r#"form"#).unwrap())
        .next()
        .unwrap()
        .value()
        .attr("action")
        .unwrap();

    let verfiy_form_data = [("SAMLResponse", saml_response), ("RelayState", relay_state)];

    let end_url_response = client.post(form_url).form(&verfiy_form_data).send();
    if end_url_response.is_err() {
        eprint!("Failed verfiying non-js-environment");
        return None;
    }

    dbg!(end_url_response.unwrap().url().as_str());
    dbg!(
        "Moodle url?",
        client
            .get(&config.moodle_url)
            .send()
            .unwrap()
            .url()
            .as_str()
    );

    return get_session_key(&config, &client);
}

fn get_session_key(config: &Config, client: &Client) -> Option<String> {
    let home_page = client.get(&config.moodle_url).send();
    if home_page.is_err() {
        return None;
    }
    let html = Html::parse_document(home_page.unwrap().text().unwrap().as_str());
    let selector = Selector::parse(r#"input[name="sesskey"]"#).unwrap();
    let mut selection = html.select(&selector);
    return Some(String::from(
        selection.next().unwrap().value().attr("value").unwrap(),
    ));
}

fn get_password(username: &String, store: &Option<bool>) -> String {
    let keyring_entry = Entry::new("ilias_uploader", &username).unwrap();
    let stored_password = keyring_entry.get_password();
    return match stored_password {
        Ok(pw) => pw,
        Err(_) => {
            let pw = Password::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Moodle/Schulcampus password for user `{}`",
                    &username
                ))
                .interact()
                .unwrap();

            if store.unwrap_or(false) {
                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!("Confirm to save password"))
                    .interact()
                    .unwrap()
                {
                    keyring_entry.set_password(&pw).unwrap();
                    println!("Password saved");
                }
            }

            return pw;
        }
    };
}

pub fn logout(config: &Config, client: &Client) {
    let logout_page = Html::parse_document(
        client
            .get(&config.moodle_url)
            .send()
            .unwrap()
            .text()
            .unwrap()
            .as_str(),
    );
    let logout_url = logout_page
        .select(&Selector::parse(r#"a[data-title="logout,moodle"]"#).unwrap())
        .next()
        .unwrap()
        .value()
        .attr("href")
        .unwrap();
    let logout_result = client.get(logout_url).send();
    dbg!(logout_result.unwrap().url().as_str());
}
