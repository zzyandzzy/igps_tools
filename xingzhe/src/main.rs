use std::fs;

mod api;
mod build_fit;

#[tokio::main]
async fn main() {
    build_fit::generate_fit();
}

async fn download(year: u32, user_id: u64, cookie: &str) {
    for i in 1..=12 {
        let save_folder = format!("xingzhe/data/{}-{}", year, i);
        fs::create_dir_all(save_folder.clone()).unwrap_or_default();
        let list = api::get_month_list(user_id, year, i as u32, cookie).await;
        for item in list {
            let segment_json = api::segment(item.workout_id, cookie)
                .await
                .unwrap_or_default();
            let points_json = api::points(item.workout_id, cookie)
                .await
                .unwrap_or_default();
            fs::write(
                format!("{}/{}-segment.json", save_folder.clone(), item.title),
                segment_json,
            )
            .unwrap_or_default();
            fs::write(
                format!("{}/{}-points.json", save_folder.clone(), item.title),
                points_json,
            )
            .unwrap_or_default();
            println!(
                "download {}-{}, workout_id: {}, title: {} success.",
                year, i, item.workout_id, item.title
            )
        }
    }
}
