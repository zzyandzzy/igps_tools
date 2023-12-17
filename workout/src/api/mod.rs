pub(crate) mod utils;

use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Response;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IGPSRequestBody {
    pub(crate) data: WorkoutData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkoutData {
    /// "workoutType": "bike"
    #[serde(rename = "workoutType")]
    pub workout_type: String,

    /// "fromTP": false
    #[serde(rename = "fromTP")]
    pub from_tp: bool,

    /// "allowDeletion": true
    #[serde(rename = "allowDeletion")]
    pub allow_deletion: bool,

    /// "totalTime": 3000
    #[serde(rename = "totalTime")]
    pub total_time: u32,

    /// "title": "训练"
    pub title: String,

    /// "description": "123"
    pub description: String,

    pub structure: Vec<WorkoutDataStructure>,
}

impl Default for WorkoutData {
    fn default() -> Self {
        Self {
            workout_type: "bike".to_string(),
            from_tp: false,
            allow_deletion: true,
            total_time: 0,
            title: Default::default(),
            description: Default::default(),
            structure: Default::default(),
        }
    }
}

/// ```json
/// [
///   {
///     "intensityClass": "WarmUp",
///     "openDuration": "false",
///     "uuid": "xxx",
///     "type": "Step",
///     "length": {
///       "value": 600,
///       "unit": "Second"
///     },
///     "name": "热身"
///   }
/// ]
/// ```
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct WorkoutDataStructure {
    /// "type": "Step"
    /// "type": "Repetition"
    #[serde(rename = "type")]
    pub workout_type: String,

    /// "name": "热身"
    pub name: String,

    /// "uuid": "xxx"
    pub uuid: String,

    /// "openDuration": "false"
    #[serde(rename = "openDuration")]
    pub open_duration: String,

    pub length: WorkoutDataStructureLength,

    /// "intensityClass": "WarmUp"
    #[serde(rename = "intensityClass")]
    pub intensity_class: String,

    #[serde(rename = "intensityTarget")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intensity_target: Option<WorkoutDataStructureLength>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<WorkoutDataStructure>>,
}

/// ```json
/// {
///     "minValue": 195,
///     "maxValue": 215,
///     "value": 0,
///     "unit": "PowerCustom"
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct WorkoutDataStructureLength {
    /// "unit": "Power"
    /// "unit": "PowerCustom"
    /// "unit": "Repetition"
    pub unit: String,

    /// "value": 600
    /// "value": 0
    pub value: u32,

    /// "maxValue": 215
    #[serde(rename = "maxValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<u32>,

    /// "minValue": 195
    #[serde(rename = "minValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<u32>,
}

const API_EDIT_CUSTOM_WORKOUT_URL: &str =
    "https://prod.zh.igpsport.com/service/mobile/api/WorkOut/EditCustomWorkOut";

pub async fn push_to_igps(workout_json: String, token: String) -> Response {
    let client = reqwest::Client::builder().build().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, token.parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    client
        .post(API_EDIT_CUSTOM_WORKOUT_URL)
        .headers(headers)
        .body(workout_json)
        .send()
        .await
        .unwrap()
}
