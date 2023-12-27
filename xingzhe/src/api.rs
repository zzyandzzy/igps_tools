use reqwest::header::{HeaderMap, COOKIE};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetMonthList {
    data: GetMonthInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetMonthInfo {
    wo_info: Vec<GetMonthInfoItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetMonthInfoItem {
    #[serde(rename = "id")]
    pub(crate) workout_id: u128,
    pub(crate) title: String,
}

/// https://www.imxingzhe.com/api/v4/segment_workout/?workout_id=
pub(crate) async fn segment(workout_id: u128, cookie: &str) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://www.imxingzhe.com/api/v4/segment_workout/?workout_id={}",
        workout_id
    );
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookie.parse().unwrap());
    let client = reqwest::Client::builder().build().unwrap();
    client.get(url).headers(headers).send().await?.text().await
}

/// https://www.imxingzhe.com/api/v1/pgworkout/{}/points/
pub(crate) async fn points(workout_id: u128, cookie: &str) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://www.imxingzhe.com/api/v1/pgworkout/{}/points/",
        workout_id
    );
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookie.parse().unwrap());
    let client = reqwest::Client::builder().build().unwrap();
    client.get(url).headers(headers).send().await?.text().await
}

/// https://www.imxingzhe.com/api/v4/user_month_info/?user_id={}&year={}&month={}
pub(crate) async fn get_month_list(
    user_id: u64,
    year: u32,
    month: u32,
    cookie: &str,
) -> Vec<GetMonthInfoItem> {
    let url = format!(
        "https://www.imxingzhe.com/api/v4/user_month_info/?user_id={}&year={}&month={}",
        user_id, year, month
    );
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookie.parse().unwrap());
    let client = reqwest::Client::builder().build().unwrap();
    let res = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .unwrap()
        .json::<GetMonthList>()
        .await
        .unwrap();
    res.data.wo_info
}
