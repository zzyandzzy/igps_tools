use crate::api::WorkoutDataStructure;
use crate::{api, FitWorkoutArgs};
use fit_rust::protocol::data_field::DataField;
use fit_rust::protocol::message_type::MessageType;
use fit_rust::protocol::value::Value;
use fit_rust::Fit;
use uuid::Uuid;

pub fn build_workout_json(fit_file: Vec<u8>, fit_workout_args: &Option<FitWorkoutArgs>) -> String {
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
                        let mut max_value: u32 = workout_step.target_value_high - 1000;
                        let mut min_value: u32 = workout_step.target_value_low - 1000;
                        let mut duration_value: u32 = workout_step.duration_value / 1000;
                        match fit_workout_args {
                            None => {}
                            Some(args) => {
                                args.apply_operation(
                                    &mut min_value,
                                    &mut max_value,
                                    &mut duration_value,
                                );
                            }
                        }

                        workout_data.structure.push(WorkoutDataStructure {
                            workout_type: "Step".to_string(),
                            name: format!("{}-{}", workout_step.step_name, workout_step.index),
                            uuid: Uuid::new_v4().to_string(),
                            intensity_class: intensity_class.into(),
                            intensity_target: Some(api::WorkoutDataStructureLength {
                                unit: "PowerCustom".to_string(),
                                value: 0,
                                max_value: Some(max_value),
                                min_value: Some(min_value),
                            }),
                            length: api::WorkoutDataStructureLength {
                                unit: "Second".to_string(),
                                value: duration_value,
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
