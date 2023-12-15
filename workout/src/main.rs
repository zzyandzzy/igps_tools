#![feature(slice_take)]

use crate::api::WorkoutDataStructure;
use dotenv::dotenv;
use fit_rust::protocol::data_field::DataField;
use fit_rust::protocol::message_type::MessageType;
use fit_rust::protocol::value::Value;
use fit_rust::Fit;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

mod api;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");
    let fit_folder = std::env::var("FIT_FOLDER").expect("Environment variable FIT_FOLDER not set");
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
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("fit") {
            fit_folder_vec.push(path);
        }
    }

    fit_folder_vec.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    for path in fit_folder_vec {
        push_to_igps(path).await;
    }
}

async fn push_to_igps(path: PathBuf) {
    let client = init_client();
    let api_url = std::env::var("IGPS_EDIT_CUSTOM_WORKOUT_URL")
        .expect("Environment variable IGPS_EDIT_CUSTOM_WORKOUT_URL not set");
    let mut headers = HeaderMap::new();
    let token = std::env::var("TOKEN").expect("Environment variable TOKEN not set");
    headers.insert(AUTHORIZATION, token.parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    let p = path.clone().to_path_buf();
    let fit_file = fs::read(path).unwrap();
    let workout_json = build_workout_json(fit_file);
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
    println!("path: {:?}, response body: {res}", p);
}

fn build_workout_json(fit_file: Vec<u8>) -> String {
    let fit: Fit = Fit::read(fit_file).unwrap();
    let mut workout_data = api::WorkoutData {
        ..Default::default()
    };
    for data in &fit.data {
        match data.message.message_type {
            MessageType::FileId | MessageType::Workout => {
                for item in &data.message.values {
                    match item.field_num {
                        8 => {
                            if data.message.message_type == MessageType::FileId {
                                workout_data.description =
                                    <Value as Clone>::clone(&item.value).into();
                            } else {
                                workout_data.title = <Value as Clone>::clone(&item.value).into();
                            }
                        }
                        _ => {}
                    }
                }
            }
            MessageType::WorkoutStep => {
                let workout_step = get_workout_step(&data.message.values);
                match workout_step.duration_type.as_str() {
                    "time" => {
                        workout_data.total_time += workout_step.duration_value / 1000;
                        let intensity_class = match workout_data.structure.len() {
                            0 => "WarmUp",
                            _ => "Active",
                        };
                        workout_data.structure.push(WorkoutDataStructure {
                            workout_type: "Step".to_string(),
                            name: format!("{}-{}", workout_step.step_name, workout_step.index),
                            uuid: Uuid::new_v4().to_string(),
                            intensity_class: intensity_class.into(),
                            intensity_target: Some(api::WorkoutDataStructureLength {
                                unit: "PowerCustom".to_string(),
                                value: 0,
                                max_value: Some(workout_step.target_value_high - 1000),
                                min_value: Some(workout_step.target_value_low - 1000),
                            }),
                            length: api::WorkoutDataStructureLength {
                                unit: "Second".to_string(),
                                value: workout_step.duration_value / 1000,
                                max_value: None,
                                min_value: None,
                            },
                            open_duration: "false".to_string(),
                            steps: None,
                        });
                    }
                    "repeat_until_steps_cmplt" => {
                        // index - duration_value
                        let count =
                            workout_step.index as usize - workout_step.duration_value as usize;
                        let repeat_data_vec = workout_data
                            .structure
                            .split_off(workout_data.structure.len() - count);
                        workout_data.structure.push(WorkoutDataStructure {
                            workout_type: "Repetition".to_string(),
                            name: format!("{}-{}", workout_step.step_name, workout_step.index),
                            uuid: Uuid::new_v4().to_string(),
                            intensity_class: "Active".into(),
                            intensity_target: None,
                            length: api::WorkoutDataStructureLength {
                                unit: "Repetition".to_string(),
                                value: workout_step.target_value,
                                max_value: None,
                                min_value: None,
                            },
                            open_duration: "false".to_string(),
                            steps: Some(repeat_data_vec),
                        });
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }

    serde_json::to_string(&api::IGPSRequestBody { data: workout_data }).unwrap()
}

#[derive(Debug, Default)]
struct FitWorkoutStep {
    step_name: String,
    duration_type: String,
    duration_value: u32,
    target_type: String,
    target_value: u32,
    target_value_low: u32,
    target_value_high: u32,
    intensity: String,
    index: u16,
}

fn get_workout_step(data_field_vec: &Vec<DataField>) -> FitWorkoutStep {
    let mut workout_step_fit = FitWorkoutStep {
        ..Default::default()
    };
    for item in data_field_vec {
        match item.field_num {
            0 => {
                // wkt_step_name
                workout_step_fit.step_name = <Value as Clone>::clone(&item.value).into();
            }
            1 => {
                // duration_type
                workout_step_fit.duration_type = <Value as Clone>::clone(&item.value).into();
            }
            2 => {
                // duration_value
                workout_step_fit.duration_value = <Value as Clone>::clone(&item.value).into();
            }
            3 => {
                // target_type
                workout_step_fit.target_type = <Value as Clone>::clone(&item.value).into();
            }
            4 => {
                // target_value
                workout_step_fit.target_value = <Value as Clone>::clone(&item.value).into();
            }
            5 => {
                // custom_target_value_low
                workout_step_fit.target_value_low = <Value as Clone>::clone(&item.value).into();
            }
            6 => {
                // custom_target_value_high
                workout_step_fit.target_value_high = <Value as Clone>::clone(&item.value).into();
            }
            7 => {
                // intensity
                workout_step_fit.intensity = <Value as Clone>::clone(&item.value).into();
            }
            254 => {
                // message_index
                workout_step_fit.index = <Value as Clone>::clone(&item.value).into();
            }
            _ => {}
        }
    }
    workout_step_fit
}
fn init_client() -> reqwest::Client {
    reqwest::Client::builder().build().unwrap()
}
