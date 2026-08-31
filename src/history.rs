use eframe::Storage;

use crate::color::{self, RgbaColor};
use crate::screen::ScreenSample;

const HISTORY_LIMIT: usize = 20;
const STORAGE_KEY: &str = "v_color_picker.history.v1";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PickRecord {
    pub color: RgbaColor,
    pub source: String,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub monitor: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PersistedHistory {
    history: Vec<PickRecord>,
}

impl PickRecord {
    pub fn starter() -> Self {
        Self {
            color: RgbaColor::new(80, 186, 255, 255),
            source: "starter".to_owned(),
            x: None,
            y: None,
            monitor: None,
        }
    }

    pub fn from_sample(sample: ScreenSample) -> Self {
        Self {
            color: sample.color,
            source: "screen".to_owned(),
            x: Some(sample.global_x),
            y: Some(sample.global_y),
            monitor: Some(sample.monitor_name),
        }
    }

    pub fn summary(&self) -> String {
        let hex = color::formats(self.color, self.color.a as f32 / 255.0)
            .into_iter()
            .find(|row| row.label == "HEX")
            .map(|row| row.value)
            .unwrap_or_else(|| "#000000".to_owned());

        match (self.x, self.y, &self.monitor) {
            (Some(x), Some(y), Some(monitor)) => format!("{hex}  {x},{y}  {monitor}"),
            _ => format!("{hex}  {}", self.source),
        }
    }
}

pub fn load(storage: Option<&dyn Storage>) -> Vec<PickRecord> {
    let mut history = match storage
        .and_then(|storage| storage.get_string(STORAGE_KEY))
        .and_then(|raw| serde_json::from_str::<PersistedHistory>(&raw).ok())
    {
        Some(state) => state.history,
        None => vec![PickRecord::starter()],
    };

    clamp(&mut history);
    history
}

pub fn save(storage: &mut dyn Storage, history: &[PickRecord]) {
    let mut history = history.to_vec();
    clamp(&mut history);

    if let Ok(raw) = serde_json::to_string(&PersistedHistory { history }) {
        storage.set_string(STORAGE_KEY, raw);
    }
}

pub fn clamp(history: &mut Vec<PickRecord>) {
    history.truncate(HISTORY_LIMIT);
}
