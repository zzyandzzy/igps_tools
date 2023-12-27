use fit_rust::protocol::data_field::DataField;
use fit_rust::protocol::message_type::MessageType;
use fit_rust::protocol::value::Value;
use fit_rust::protocol::{
    DataMessage, DefinitionMessage, FieldDefinition, FitDataMessage, FitDefinitionMessage,
    FitHeader, FitMessage, FitMessageHeader,
};
use fit_rust::Fit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct XingZhePoint {
    points: Vec<Point>,
    encoding_points: String,
}

#[derive(Serialize, Deserialize)]
struct Point {
    heartrate: f32,
    power: f32,
    time: f64,
    altitude: f32,
    speed: f32,
    cadence: f32,
}

#[derive(Serialize, Deserialize)]
struct Segment {
    workout: WorkoutSession,
}

#[derive(Serialize, Deserialize)]
struct WorkoutSession {
    title: String,
    elevation_gain: u16,
    elevation_loss: u16,
    start_time: u64,
    end_time: u64,
    distance: u32,
    down_distance: u32,
    up_distance: u32,
    calories: u32,
    max_speed: f32,
    max_altitude: u16,
    max_grade: i16,
    min_grade: i16,
    avg_speed: f32,
    avg_heartrate: u8,
    max_heartrate: u8,
    avg_cadence: u8,
    max_cadence: u8,
    #[serde(rename = "powerMax")]
    power_max: u16,
    #[serde(rename = "powerAvg")]
    power_avg: u16,
    #[serde(rename = "powerTSS")]
    power_tss: f32,
    #[serde(rename = "powerIF")]
    power_if: f32,
    #[serde(rename = "powerNP")]
    power_np: f32,
}

#[derive(Clone, Debug)]
struct FitRecord {
    /// lat
    pub lat: f32,
    /// long
    pub long: f32,
    /// alt
    pub alt: u16,
    /// heart
    pub heart: u8,
    /// cadence
    pub cadence: u8,
    /// distance
    pub distance: u32,
    /// speed
    pub speed: u16,
    /// power
    pub power: u16,
    /// temperature
    pub temperature: i8,
    /// timestamp
    pub timestamp: u32,
}

pub fn generate_fit(segment: String, points: String) {
    let segment: Segment = serde_json::from_str(&segment).unwrap();
    let session = segment.workout;
    let xinzhe: XingZhePoint = serde_json::from_str(&points).unwrap();
    let decoded_polyline = polyline::decode_polyline(&xinzhe.encoding_points, 5)
        .unwrap()
        .0;

    let mut write_fit: Fit = Fit {
        header: FitHeader {
            header_size: 14,
            protocol_version: 16,
            profile_version: 2132,
            data_size: 0,
            data_type: ".FIT".to_string(),
            crc: Some(1),
        },
        data: vec![],
    };
    let mut fit_data: Vec<FitMessage> = vec![];
    let start_time = (&session.start_time / 1000) as u32;
    let end_time = (&session.end_time / 1000) as u32;
    fit_data.push(build_igps_file_id_def());
    fit_data.push(build_igps_file_id(start_time));
    fit_data.push(build_file_creator_def());
    fit_data.push(build_file_creator());
    fit_data.push(build_igps_device_info_def());
    fit_data.push(build_igps_device_info(start_time));
    fit_data.push(build_event_def());
    fit_data.push(build_event(vec![
        DataField::new(253, Value::Time(start_time)),
        DataField::new(0, Value::Enum("timer")),
        DataField::new(1, Value::Enum("start")),
    ]));
    fit_data.push(build_record_def());

    // record
    let mut current_distance = 0_u32;
    let mut min_alt = u16::MAX;
    let mut total_alt = 0_u64;
    for (i, v) in xinzhe.points.iter().enumerate() {
        let coordinate = decoded_polyline[i];
        let alt = (v.altitude * 5.0 + 500.0) as u16;
        total_alt += alt as u64;
        if alt < min_alt {
            min_alt = alt;
        }
        let speed = (v.speed * 1000.0) as u16;
        current_distance += speed as u32 * 100;
        let record = FitRecord {
            lat: coordinate.y as f32,
            long: coordinate.x as f32,
            alt: alt,
            heart: v.heartrate as u8,
            cadence: v.cadence as u8,
            distance: current_distance,
            speed,
            power: v.power as u16,
            temperature: 25,
            timestamp: (v.time / 1000.0) as u32,
        };
        fit_data.push(build_record(record));
    }
    let avg_alt = (total_alt / xinzhe.points.len() as u64) as u16;

    fit_data.push(build_event(vec![
        DataField::new(253, Value::Time(end_time)),
        DataField::new(0, Value::Enum("timer")),
        DataField::new(1, Value::Enum("stop")),
    ]));

    fit_data.push(build_event(vec![
        DataField::new(253, Value::Time(end_time)),
        DataField::new(0, Value::Enum("timer")),
        DataField::new(1, Value::Enum("stop_all")),
    ]));

    fit_data.push(build_event(vec![
        DataField::new(253, Value::Time(end_time)),
        DataField::new(0, Value::Enum("session")),
        DataField::new(1, Value::Enum("stop_disable_all")),
    ]));

    // build session
    fit_data.push(build_session_def());
    fit_data.push(build_session(get_session_vec(&session, min_alt, avg_alt)));

    fit_data.push(build_activity_def());
    fit_data.push(build_activity(vec![
        DataField::new(253, Value::Time(end_time)),
        DataField::new(0, Value::U32(end_time - start_time)),
        DataField::new(5, Value::Time(start_time)),
        DataField::new(1, Value::U16(0)),
        DataField::new(2, Value::Enum("manual")),
        DataField::new(3, Value::Enum("activity")),
        DataField::new(4, Value::Enum("stop")),
    ]));

    fit_data.push(build_sport_def());
    fit_data.push(build_sport());

    write_fit.data = fit_data;
    write_fit
        .write(format!("xingzhe/data/{}.fit", session.title))
        .unwrap();
}

fn get_session_vec(session: &WorkoutSession, min_alt: u16, avg_alt: u16) -> Vec<DataField> {
    let start_time = (&session.start_time / 1000) as u32;
    let end_time = (&session.end_time / 1000) as u32;
    let max_alt = session.max_altitude * 5 + 500;
    vec![
        // end time
        DataField::new(253, Value::Time(end_time)),
        // start time
        DataField::new(2, Value::Time(start_time)),
        // total_elapsed_time
        DataField::new(7, Value::U32((end_time - start_time) * 1000)),
        // total_timer_time
        DataField::new(8, Value::U32((end_time - start_time) * 1000)),
        // total_distance
        DataField::new(9, Value::U32(session.distance * 100)),
        // total_moving_time
        DataField::new(59, Value::U32((end_time - start_time) * 1000)),
        // message_index field
        DataField::new(254, Value::U16(1)),
        // total_calories
        DataField::new(11, Value::U16((session.calories / 1000) as u16)),
        // avg_speed field
        DataField::new(14, Value::U16((session.avg_speed / 3.6 * 1000.0) as u16)),
        // max_speed
        DataField::new(15, Value::U16((session.max_speed / 3.6 * 1000.0) as u16)),
        // avg power
        DataField::new(20, Value::U16(session.power_avg)),
        // max power
        DataField::new(21, Value::U16(session.power_max)),
        // total_ascent
        DataField::new(22, Value::U16(session.elevation_gain)),
        // total_descent
        DataField::new(23, Value::U16(session.elevation_loss)),
        // first_lap_index field
        DataField::new(25, Value::U16(0)),
        // num_laps field
        DataField::new(26, Value::U16(1)),
        // normalized_power field
        DataField::new(34, Value::U16(session.power_np as u16)),
        // training_stress_score field
        DataField::new(35, Value::U16(session.power_tss as u16)),
        // intensity_factor field
        DataField::new(36, Value::U16((session.power_if * 1000.0) as u16)),
        // // left_right_balance field
        // DataField::new(37, Value::I16(i16::MAX)),
        // avg_altitude
        DataField::new(49, Value::U16(avg_alt)),
        // max_altitude
        DataField::new(50, Value::U16(max_alt)),
        // max_pos_grade
        DataField::new(55, Value::I16(session.max_grade * 100)),
        // max_neg_grade
        DataField::new(56, Value::I16(session.min_grade * 100)),
        // min_altitude
        DataField::new(71, Value::U16(min_alt)),
        DataField::new(0, Value::Enum("session")),
        DataField::new(1, Value::Enum("stop")),
        DataField::new(5, Value::Enum("cycling")),
        DataField::new(6, Value::Enum("road")),
        // avg_heart_rate
        DataField::new(16, Value::U8(session.avg_heartrate)),
        // max_heart_rate
        DataField::new(17, Value::U8(session.max_heartrate)),
        // avg_cadence
        DataField::new(18, Value::U8(session.avg_cadence)),
        // max_cadence
        DataField::new(19, Value::U8(session.max_cadence)),
        // avg_temperature
        DataField::new(57, Value::I8(25)),
        // max_temperature
        DataField::new(58, Value::I8(25)),
        // min_heart_rate
        DataField::new(64, Value::U8(session.avg_heartrate)),
    ]
}

fn build_session_def() -> FitMessage {
    build_fit_def(
        3,
        vec![
            FieldDefinition::new(253, 4, true, 6),
            FieldDefinition::new(2, 4, true, 6),
            FieldDefinition::new(7, 4, true, 6),
            FieldDefinition::new(8, 4, true, 6),
            FieldDefinition::new(9, 4, true, 6),
            FieldDefinition::new(59, 4, true, 6),
            FieldDefinition::new(254, 2, true, 4),
            FieldDefinition::new(11, 2, true, 4),
            FieldDefinition::new(14, 2, true, 4),
            FieldDefinition::new(15, 2, true, 4),
            FieldDefinition::new(20, 2, true, 4),
            FieldDefinition::new(21, 2, true, 4),
            FieldDefinition::new(22, 2, true, 4),
            FieldDefinition::new(23, 2, true, 4),
            FieldDefinition::new(25, 2, true, 4),
            FieldDefinition::new(26, 2, true, 4),
            FieldDefinition::new(34, 2, true, 4),
            FieldDefinition::new(35, 2, true, 4),
            FieldDefinition::new(36, 2, true, 4),
            FieldDefinition::new(49, 2, true, 4),
            FieldDefinition::new(50, 2, true, 4),
            FieldDefinition::new(55, 2, true, 3),
            FieldDefinition::new(56, 2, true, 3),
            FieldDefinition::new(71, 2, true, 4),
            FieldDefinition::new(0, 1, true, 0),
            FieldDefinition::new(1, 1, true, 0),
            FieldDefinition::new(5, 1, true, 0),
            FieldDefinition::new(6, 1, true, 0),
            FieldDefinition::new(16, 1, true, 2),
            FieldDefinition::new(17, 1, true, 2),
            FieldDefinition::new(18, 1, true, 2),
            FieldDefinition::new(19, 1, true, 2),
            FieldDefinition::new(57, 1, true, 1),
            FieldDefinition::new(58, 1, true, 1),
            FieldDefinition::new(64, 1, true, 2),
        ],
        MessageType::Session,
    )
}

fn build_igps_file_id(start_time: u32) -> FitMessage {
    build_fit_message(
        6,
        MessageType::FileId,
        vec![
            DataField::new(
                3,
                Value::ArrU8(vec![
                    119, 39, 144, 3, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0,
                ]),
            ),
            DataField::new(4, Value::Time(start_time)),
            DataField::new(1, Value::Enum("igpsport")),
            DataField::new(2, Value::U16(200)),
            DataField::new(0, Value::Enum("activity")),
        ],
    )
}

fn build_igps_file_id_def() -> FitMessage {
    build_fit_def(
        6,
        vec![
            FieldDefinition::new(3, 32, true, 12),
            FieldDefinition::new(4, 4, true, 6),
            FieldDefinition::new(1, 2, true, 4),
            FieldDefinition::new(2, 2, true, 4),
            FieldDefinition::new(0, 1, false, 0),
        ],
        MessageType::FileId,
    )
}

fn build_record(record: FitRecord) -> FitMessage {
    build_fit_message(
        5,
        MessageType::Record,
        vec![
            DataField::new(0, Value::F32(record.lat)),
            DataField::new(1, Value::F32(record.long)),
            DataField::new(2, Value::U16(record.alt)),
            DataField::new(3, Value::U8(record.heart)),
            DataField::new(4, Value::U8(record.cadence)),
            DataField::new(5, Value::U32(record.distance)),
            DataField::new(6, Value::U16(record.speed)),
            DataField::new(7, Value::U16(record.power)),
            DataField::new(13, Value::I8(record.temperature)),
            DataField::new(253, Value::Time(record.timestamp)),
        ],
    )
}

fn build_record_def() -> FitMessage {
    build_fit_def(
        5,
        vec![
            FieldDefinition::new(0, 4, false, 8),
            FieldDefinition::new(1, 4, false, 8),
            FieldDefinition::new(2, 2, true, 4),
            FieldDefinition::new(3, 1, true, 2),
            FieldDefinition::new(4, 1, true, 2),
            FieldDefinition::new(5, 4, true, 6),
            FieldDefinition::new(6, 2, true, 4),
            FieldDefinition::new(7, 2, true, 4),
            FieldDefinition::new(13, 1, true, 1),
            FieldDefinition::new(253, 4, true, 6),
        ],
        MessageType::Record,
    )
}

fn build_file_creator() -> FitMessage {
    build_fit_message(
        7,
        MessageType::FileCreator,
        vec![
            DataField::new(0, Value::U16(139)),
            DataField::new(1, Value::U8(100)),
        ],
    )
}

fn build_file_creator_def() -> FitMessage {
    build_fit_def(
        7,
        vec![
            FieldDefinition::new(0, 2, true, 4),
            FieldDefinition::new(1, 1, false, 2),
        ],
        MessageType::FileCreator,
    )
}

fn build_igps_device_info(start_time: u32) -> FitMessage {
    build_fit_message(
        1,
        MessageType::DeviceInfo,
        vec![
            DataField::new(253, Value::Time(start_time)),
            DataField::new(
                3,
                Value::ArrU8(vec![
                    119, 39, 144, 3, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0,
                ]),
            ),
            DataField::new(2, Value::Enum("igpsport")),
            DataField::new(4, Value::U16(200)),
            DataField::new(5, Value::U16(139)),
            DataField::new(0, Value::U8(255)),
            DataField::new(1, Value::U8(0)),
            DataField::new(6, Value::U8(100)),
        ],
    )
}
fn build_igps_device_info_def() -> FitMessage {
    build_fit_def(
        1,
        vec![
            FieldDefinition::new(253, 4, true, 6),
            FieldDefinition::new(3, 32, true, 12),
            FieldDefinition::new(2, 2, true, 4),
            FieldDefinition::new(4, 2, true, 4),
            FieldDefinition::new(5, 2, true, 4),
            FieldDefinition::new(0, 1, false, 2),
            FieldDefinition::new(1, 1, false, 2),
            FieldDefinition::new(6, 1, false, 2),
        ],
        MessageType::DeviceInfo,
    )
}

fn build_event(values: Vec<DataField>) -> FitMessage {
    build_fit_message(0, MessageType::Event, values)
}

fn build_event_def() -> FitMessage {
    build_fit_def(
        3,
        vec![
            FieldDefinition::new(253, 4, true, 6),
            FieldDefinition::new(0, 1, false, 0),
            FieldDefinition::new(1, 1, false, 0),
        ],
        MessageType::Event,
    )
}

fn build_activity(values: Vec<DataField>) -> FitMessage {
    build_fit_message(4, MessageType::Activity, values)
}

fn build_activity_def() -> FitMessage {
    build_fit_def(
        4,
        vec![
            FieldDefinition::new(253, 4, true, 6),
            FieldDefinition::new(0, 4, true, 6),
            FieldDefinition::new(5, 4, true, 6),
            FieldDefinition::new(1, 2, true, 4),
            FieldDefinition::new(2, 1, false, 0),
            FieldDefinition::new(3, 1, false, 0),
            FieldDefinition::new(4, 1, false, 0),
        ],
        MessageType::Activity,
    )
}

fn build_sport() -> FitMessage {
    build_fit_message(
        10,
        MessageType::Sport,
        vec![
            DataField::new(3, Value::String("Road Cycling\0\0\0\0".to_string())),
            DataField::new(0, Value::Enum("cycling")),
            DataField::new(1, Value::Enum("road")),
        ],
    )
}

fn build_sport_def() -> FitMessage {
    build_fit_def(
        10,
        vec![
            FieldDefinition::new(3, 16, false, 7),
            FieldDefinition::new(0, 1, false, 0),
            FieldDefinition::new(1, 1, false, 0),
        ],
        MessageType::Sport,
    )
}

fn build_session(values: Vec<DataField>) -> FitMessage {
    build_fit_message(3, MessageType::Session, values)
}

fn build_fit_message(
    local_num: u8,
    message_type: MessageType,
    values: Vec<DataField>,
) -> FitMessage {
    FitMessage::Data(FitDataMessage {
        header: FitMessageHeader::new(false, local_num),
        data: DataMessage {
            message_type,
            values,
        },
    })
}

fn build_fit_def(local_num: u8, values: Vec<FieldDefinition>, msg_type: MessageType) -> FitMessage {
    FitMessage::Definition(FitDefinitionMessage {
        header: FitMessageHeader::new(true, local_num),
        data: DefinitionMessage::new(false, values.len() as u8, values, msg_type),
    })
}
