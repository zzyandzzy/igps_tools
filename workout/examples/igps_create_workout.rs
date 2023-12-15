use dotenv::dotenv;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use std::fs;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");
    let workout_json = fs::read_to_string("examples/workout.json").unwrap();
    let client = init_client();
    let api_url = std::env::var("IGPS_EDIT_CUSTOM_WORKOUT_URL").unwrap();
    let mut headers = HeaderMap::new();
    let token = std::env::var("TOKEN").unwrap();
    headers.insert(AUTHORIZATION, token.parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    let res = client
        .post(api_url)
        .headers(headers)
        .body(workout_json)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("res: {res}");
}

fn init_client() -> reqwest::Client {
    reqwest::Client::builder().build().unwrap()
}
