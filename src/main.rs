use chrono::{Duration, Local};
use serde_json::json;
use sodexo_webhook::{build_message, get_lunch, parse_env};
use tokio::time::{self, Instant};

#[tokio::main]
async fn main() {
    let (sodexo_url, webhook_url, post_time) = parse_env().unwrap_or_else(|error| {
        panic!("{error}, please check that you have defined sodexo_url, webhook_url and post_time")
    });

    // Parse hh:mm string to hours and minutes
    let [hours, minutes, seconds] = match post_time
        .split(':')
        .map(|str| str.parse().unwrap_or(0))
        .collect::<Vec<u32>>()[..]
    {
        [hours, minutes] => [hours, minutes, 0],
        [hours] => [hours, 0, 0],
        _ => [0, 0, 0],
    };

    let client = reqwest::Client::new(); // Client for reqwest requests

    // Get current time and next post time
    let now = Local::now();

    // If we are past today's post time, set post time to start from tomorrow's post time
    let days = if now
        .date()
        .and_hms(hours, minutes, seconds)
        .signed_duration_since(now)
        < Duration::zero()
    {
        1
    } else {
        0
    };

    // Set start time based on days variable
    let interval_start = (now + Duration::days(days))
        .date()
        .and_hms(hours, minutes, seconds)
        .signed_duration_since(now);

    // Construct interval
    let mut interval = time::interval_at(
        Instant::now()
            + interval_start
                .to_std()
                .unwrap_or(time::Duration::from_millis(0)),
        chrono::Duration::days(1)
            .to_std()
            .unwrap_or(time::Duration::from_millis(0)),
    );

    // Wait for interval in a loop and post to Discord when it's time
    loop {
        interval.tick().await;
        let message_string = build_message(get_lunch(&sodexo_url).await);
        match client
            .post(&webhook_url)
            .json(&json!({
                "content": message_string,
            }))
            .send()
            .await
        {
            Ok(_) => println!("Post OK"),
            Err(err) => println!("Post failed with error: {err}"),
        }
    }
}
