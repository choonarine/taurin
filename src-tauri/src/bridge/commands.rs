use std::sync::Mutex;

use crate::audio::{KiraAudioEngine, RpgMakerAudioCommand};

use super::types::BridgeResponse;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpgMakerAudioRequest {
    pub kind: String,
    pub name: Option<String>,
    pub volume: Option<i32>,
    pub pitch: Option<i32>,
    pub pan: Option<i32>,
    pub position: Option<f64>,
    pub duration: Option<f64>,
}

impl From<RpgMakerAudioRequest> for RpgMakerAudioCommand {
    fn from(request: RpgMakerAudioRequest) -> Self {
        Self {
            kind: request.kind,
            name: request.name,
            volume: request.volume,
            pitch: request.pitch,
            pan: request.pan,
            position: request.position,
            duration: request.duration,
        }
    }
}

#[tauri::command]
pub fn sync_rpg_maker_window_title(window: tauri::Window, title: String) -> BridgeResponse<()> {
    if title.trim().is_empty() {
        return BridgeResponse::ok(());
    }

    match window.set_title(&title) {
        Ok(()) => BridgeResponse::ok(()),
        Err(error) => BridgeResponse::error("window_title_sync_failed", error.to_string()),
    }
}

#[tauri::command]
pub fn rpg_maker_audio_play(
    audio_engine: tauri::State<'_, Mutex<KiraAudioEngine>>,
    request: RpgMakerAudioRequest,
) -> BridgeResponse<()> {
    with_audio_engine(audio_engine, |engine| engine.play(request.into()))
}

#[tauri::command]
pub fn rpg_maker_audio_stop(
    audio_engine: tauri::State<'_, Mutex<KiraAudioEngine>>,
    kind: String,
) -> BridgeResponse<()> {
    with_audio_engine(audio_engine, |engine| engine.stop(&kind))
}

#[tauri::command]
pub fn rpg_maker_audio_fade_out(
    audio_engine: tauri::State<'_, Mutex<KiraAudioEngine>>,
    request: RpgMakerAudioRequest,
) -> BridgeResponse<()> {
    with_audio_engine(audio_engine, |engine| engine.fade_out(request.into()))
}

fn with_audio_engine(
    audio_engine: tauri::State<'_, Mutex<KiraAudioEngine>>,
    operation: impl FnOnce(&mut KiraAudioEngine) -> Result<(), String>,
) -> BridgeResponse<()> {
    match audio_engine.lock() {
        Ok(mut engine) => match operation(&mut engine) {
            Ok(()) => BridgeResponse::ok(()),
            Err(error) => BridgeResponse::error("native_audio_failed", error),
        },
        Err(error) => BridgeResponse::error("native_audio_lock_failed", error.to_string()),
    }
}
