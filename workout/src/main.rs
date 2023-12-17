use clap::{Args, Parser};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod api;
mod util;

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
pub(crate) struct Cli {
    #[clap(flatten)]
    pub(crate) fit_res: FitResource,

    /// iGPS user token(choose one of token and username/password)
    #[arg(short, long)]
    pub(crate) token: Option<String>,

    /// iGPS user username(choose one of token and username/password)
    #[arg(short, long)]
    pub(crate) username: Option<String>,

    /// iGPS user password(choose one of token and username/password)
    #[arg(short, long)]
    pub(crate) password: Option<String>,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub(crate) struct FitResource {
    /// The path of a single fit file
    #[arg(short, long)]
    pub(crate) fit: Option<String>,

    /// The path of multiple fit folders
    #[arg(short, long)]
    pub(crate) fit_folder: Option<String>,

    /// The path of the fit zip package
    #[arg(short, long)]
    pub(crate) fit_zip: Option<String>,
}

const TMP_FOLDER: &str = "./.tmp";

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let token = match (&cli.token, &cli.username, &cli.password) {
        (Some(t), None, None) => t.clone(),
        (None, Some(username), Some(password)) => {
            let response = auth::get_token(username, password).await;
            match response.access_token {
                Some(token) => format!("Bearer {}", token),
                None => {
                    panic!("Login fail, msg: {:?}", response);
                }
            }
        }
        _ => {
            panic!("Error: Must use either 'token' or both 'username' and 'password'");
        }
    };

    match &cli.fit_res {
        FitResource {
            fit: Some(fit_path),
            ..
        } => {
            let path = Path::new(&fit_path).to_path_buf();
            if is_fit(&path) {
                do_push_fit(path, token).await;
            } else {
                panic!("{:?} it's not a fit file!", path);
            }
        }
        FitResource {
            fit_folder: Some(folder_path),
            ..
        } => {
            do_push_fit_folder(folder_path, token).await;
        }
        FitResource {
            fit_zip: Some(zip_path),
            ..
        } => {
            match util::unzip_file(zip_path, TMP_FOLDER) {
                Ok(_) => {
                    do_push_fit_folder(&TMP_FOLDER.to_string(), token).await;
                }
                Err(e) => {
                    eprintln!("Err, msg: {e}");
                }
            }
            fs::remove_dir_all(TMP_FOLDER).unwrap();
        }
        _ => unreachable!(),
    }
}

async fn do_push_fit(fit_path: PathBuf, token: String) {
    let p = fit_path.clone().to_path_buf();
    let fit_file = fs::read(fit_path).unwrap();
    let workout_json = api::utils::build_workout_json(fit_file);
    let res = api::push_to_igps(workout_json, token).await;
    let status = res.status();
    let res = res.text().await.unwrap();
    println!("path: {:?}, response status: {:?}, body: {res}", p, status);
}

async fn do_push_fit_folder(fit_folder: &String, token: String) {
    let fit_folder = Path::new(&fit_folder);
    let mut fit_folder_vec: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(fit_folder) {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Error: {}", err);
                continue;
            }
        };
        let path = entry.path().to_path_buf();
        if path.is_dir() {
            continue;
        }
        if is_fit(&path) {
            fit_folder_vec.push(path);
        } else {
            eprintln!("{:?} it's not a fit file!", path);
        }
    }

    fit_folder_vec.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    for path in fit_folder_vec {
        do_push_fit(path, token.clone()).await;
    }
}

fn is_fit(path_buf: &PathBuf) -> bool {
    path_buf.is_file() && path_buf.extension().and_then(|s| s.to_str()) == Some("fit")
}
