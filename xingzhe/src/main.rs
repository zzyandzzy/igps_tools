use clap::Parser;

mod api;
mod build_fit;

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
pub(crate) struct Cli {
    /// XingZhe user cookie
    #[arg(short, long)]
    pub(crate) cookie: String,

    /// XingZhe user id
    #[arg(short, long)]
    pub(crate) user_id: u64,

    /// choose convert year
    #[arg(short, long)]
    pub(crate) year: Option<u32>,

    /// choose convert month
    #[arg(short, long)]
    pub(crate) month: Option<u32>,

    /// choose workout id
    #[arg(short, long)]
    pub(crate) workout_id: Option<u128>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    download(
        cli.workout_id,
        cli.year,
        cli.month,
        cli.user_id,
        cli.cookie.as_str(),
    )
    .await;
}

async fn download(
    workout_id: Option<u128>,
    year: Option<u32>,
    month: Option<u32>,
    user_id: u64,
    cookie: &str,
) {
    match (workout_id, year, month) {
        (Some(workout_id), _, _) => {
            generate(workout_id, cookie).await;
            println!("get workout_id: {} success.", workout_id);
        }
        (None, Some(year), Some(month)) => {
            let list = api::get_month_list(user_id, year, month, cookie).await;
            for item in list {
                generate(item.workout_id, cookie).await;
                println!(
                    "get {}-{}, workout_id: {}, title: {} success.",
                    year, month, item.workout_id, item.title
                );
            }
        }
        _ => {
            panic!("workout id or year/month maybe not set!")
        }
    }
}

async fn generate(workout_id: u128, cookie: &str) {
    let segment_json = api::segment(workout_id, cookie).await.unwrap_or_default();
    let points_json = api::points(workout_id, cookie).await.unwrap_or_default();
    build_fit::generate_fit(segment_json, points_json);
}
