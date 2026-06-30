//! Player-facing settings persisted to disk (currently just difficulty).

use crate::ecs::Difficulty;

const SETTINGS_PATH: &str = "cobalt_settings.txt";

pub fn load() -> Difficulty {
    std::fs::read_to_string(SETTINGS_PATH)
        .ok()
        .and_then(|text| text.trim().parse::<u8>().ok())
        .map(Difficulty::from_code)
        .unwrap_or_default()
}

pub fn save(difficulty: Difficulty) {
    let _ = std::fs::write(SETTINGS_PATH, difficulty.code().to_string());
}
