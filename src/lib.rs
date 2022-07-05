use serde_json::{json, Value};
use std::env;
use std::fmt::Write;

const DEFAULT_POST_TIME: &str = "07:00";

/// Checks if there are peanuts in todays lunch items
pub fn peanut_check(additional_diet_info: &Value) -> bool {
    if let Value::Object(diet_info) = additional_diet_info {
        if let Some(Value::String(allergens)) = diet_info.get("allergens") {
            allergens.to_lowercase().contains("pähkinä")
        } else {
            false
        }
    } else {
        false
    }
}

/// Builds a message for Discord webhook API
///
/// return the message in String format
pub fn build_message(sodexo_response: Value) -> String {
    let mut string = String::new();
    if let Some(Value::Object(lunch)) = sodexo_response.get("courses") {
        write!(
            string,
            "**{} Lounas {}**",
            sodexo_response
                .get("meta")
                .unwrap_or(&Value::Null)
                .get("ref_title")
                .unwrap_or(&Value::Null)
                .as_str()
                .unwrap_or("Unknown restaurant"),
            chrono::Local::today().naive_local()
        )
        .unwrap_or_else(|_| println!("Could not write to message string"));
        string.push_str("```\n");

        lunch.iter().for_each(|course| {
            if let Value::Object(food_item) = course.1 {
                let peanuts =
                    peanut_check(food_item.get("additionalDietInfo").unwrap_or(&Value::Null));
                write!(
                    string,
                    "{}: {} {} {}\n\n",
                    food_item
                        .get("title_fi")
                        .unwrap_or(&Value::Null)
                        .as_str()
                        .unwrap_or("?"),
                    food_item
                        .get("price")
                        .unwrap_or(&Value::Null)
                        .as_str()
                        .unwrap_or("? €"),
                    food_item
                        .get("dietcodes")
                        .unwrap_or(&Value::Null)
                        .as_str()
                        .unwrap_or(""),
                    if peanuts { "🥜" } else { "" },
                )
                .unwrap_or_else(|_| println!("Could not write to message string"));
            }
        });
        string.push_str("```");
    } else {
        string.push_str("Nothing for today")
    }
    string
}

/// Parses env variables and return a Result
///
/// If parsing is successful the order of values is (sodexo_url, webhook_url, post_time)
pub fn parse_env() -> Result<(String, String, String), dotenv::Error> {
    let sodexo_url = match env::var("sodexo_url") {
        Ok(sodexo_url) => sodexo_url,
        Err(_) => dotenv::var("sodexo_url")?,
    };
    let webhook_url = match env::var("webhook_url") {
        Ok(webhook_url) => webhook_url,
        Err(_) => dotenv::var("webhook_url")?,
    };
    let post_time = match env::var("post_time") {
        Ok(post_time) => post_time,
        Err(_) => match dotenv::var("post_time") {
            Ok(post_time) => post_time,
            Err(_) => {
                println!(
                    "No post_time specified in environment. Defaulting to {DEFAULT_POST_TIME}"
                );
                DEFAULT_POST_TIME.to_string()
            }
        },
    };
    Ok((sodexo_url, webhook_url, post_time))
}

/// Get daily lunch info from sodexo API
///
/// Returns lunch as JSON
pub async fn get_lunch(sodexo_url: &str) -> Value {
    let lunch_request = reqwest::get(format!(
        "{sodexo_url}{}",
        chrono::Local::today().naive_local(),
    ))
    .await;

    let lunch: Value = match lunch_request {
        Ok(lunch) => lunch.json().await.unwrap_or_else(|err| {
            println!("{}", { err });
            json!({
                "meta": Value::Null,
                "courses": Value::Null,
            })
        }),
        Err(_) => json!({
            "courses": Value::Null,
            "meta": Value::Null,
        }),
    };
    lunch
}

#[cfg(test)]
mod peanut_check_tests {
    use super::*;
    #[test]
    fn return_true_on_peanut() {
        let peanuts = json!({"allergens": "pähkinä"});
        assert!(peanut_check(&peanuts));
    }
    #[test]
    fn return_false_on_no_peanut() {
        let no_peanuts = json!({"allergens": "kala"});
        assert!(!peanut_check(&no_peanuts));
    }
    #[test]
    fn return_false_on_null() {
        assert!(!peanut_check(&Value::Null));
    }
}
#[cfg(test)]
mod build_message_tests {
    use super::*;
    #[test]
    fn return_expected_message() {
        let test_data = json! ({
            "meta": {
                "ref_title": "Test Restaurant"
            },
            "courses": {
                "1": {
                    "title_fi": "Test food",
                    "price": "10,45 € / 2,70 €",
                    "dietcodes": "L"
                }
            }
        });
        let today = chrono::Local::today().naive_local();
        let expected_message =
            format!("**Test Restaurant Lounas {today}**```\nTest food: 10,45 € / 2,70 € L \n\n```");

        assert_eq!(build_message(test_data), expected_message);
    }
}
