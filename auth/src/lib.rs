use reqwest::header::{HeaderMap, CONTENT_TYPE};
use serde::Deserialize;
use std::collections::HashMap;

const API_TOKEN_URL: &str = "https://prod.zh.igpsport.com/service/auth/connect/token";

#[derive(Deserialize, Debug)]
pub struct AuthResponse {
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub access_token: Option<String>,
}

pub async fn get_token(username: &str, password: &str) -> AuthResponse {
    let mut params = HashMap::new();
    params.insert("username", username);
    params.insert("password", password);
    params.insert("grant_type", "password");
    params.insert("client_secret", "0d9b5544-6871-2999-13a9-59198788054b");
    params.insert(
        "scope",
        "openid offline_access mobile.api user.api device.api activity.api IdentityServerApi",
    );
    params.insert("client_id", "qiwu.mobile");
    let client = reqwest::Client::builder().build().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        "application/x-www-form-urlencoded".parse().unwrap(),
    );

    client
        .post(API_TOKEN_URL)
        .headers(headers)
        .form(&params)
        .send()
        .await
        .unwrap()
        .json::<AuthResponse>()
        .await
        .unwrap()
}
