use fit_rust::protocol::data_field::DataField;
use fit_rust::protocol::message_type::MessageType;
use fit_rust::protocol::value::Value;
use fit_rust::Fit;
use std::fs;

fn main() {
    let file = fs::read("examples/W2_5_125.fit").unwrap();
    let fit: Fit = Fit::read(file).unwrap();
    for data in &fit.data {
        match data.message.message_type {
            MessageType::FileId => {
                println!("FileId: {:?}", data.message);
            }
            MessageType::Workout => {
                print_workout(&data.message.values);
            }
            MessageType::WorkoutStep => {
                print_workout_step(&data.message.values);
            }
            _ => {
                println!("_: {:?}", data.message);
            }
        }
    }
}

#[derive(Debug, Default)]
struct FitWorkout {
    sport: String,
    name: String,
    num_valid_steps: u16,
}

fn print_workout(data_field_vec: &Vec<DataField>) {
    let mut workout = FitWorkout {
        ..Default::default()
    };
    for item in data_field_vec {
        match item.field_num {
            4 => {
                // sport
                workout.sport = <Value as Clone>::clone(&item.value).into();
            }
            6 => {
                // num_valid_steps
                workout.num_valid_steps = <Value as Clone>::clone(&item.value).into();
            }
            8 => {
                // wkt_name
                workout.name = <Value as Clone>::clone(&item.value).into();
            }
            _ => {}
        }
    }
    println!("FitWorkout: {:?}", workout);
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

fn print_workout_step(data_field_vec: &Vec<DataField>) {
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
    println!("FitWorkoutStep: {:?}", workout_step_fit);
}
