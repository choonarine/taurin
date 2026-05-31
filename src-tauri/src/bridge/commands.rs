use std::sync::Mutex;

use crate::audio::{KiraAudioEngine, RpgMakerAudioCommand};

use super::types::BridgeResponse;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NwWindowResizeByRequest {
    pub width_delta: f64,
    pub height_delta: f64,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NwWindowResizeToRequest {
    pub width: f64,
    pub height: f64,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NwWindowMoveByRequest {
    pub x_delta: f64,
    pub y_delta: f64,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NwWindowMoveToRequest {
    pub x: f64,
    pub y: f64,
}

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
pub fn nw_window_resize_by(
    window: tauri::Window,
    request: NwWindowResizeByRequest,
) -> BridgeResponse<()> {
    println!(
        "[taurin:nw-window] resize_by requested: width_delta={}, height_delta={}",
        request.width_delta, request.height_delta
    );

    let scale_factor = match window.scale_factor() {
        Ok(scale_factor) => scale_factor,
        Err(error) => {
            return BridgeResponse::error("window_scale_factor_failed", error.to_string());
        }
    };
    let size = match window.inner_size() {
        Ok(size) => size.to_logical::<f64>(scale_factor),
        Err(error) => return BridgeResponse::error("window_size_read_failed", error.to_string()),
    };

    resize_window_to(
        &window,
        size.width + request.width_delta,
        size.height + request.height_delta,
    )
}

#[tauri::command]
pub fn nw_window_resize_to(
    window: tauri::Window,
    request: NwWindowResizeToRequest,
) -> BridgeResponse<()> {
    println!(
        "[taurin:nw-window] resize_to requested: width={}, height={}",
        request.width, request.height
    );

    resize_window_to(&window, request.width, request.height)
}

#[tauri::command]
pub fn nw_window_move_by(
    window: tauri::Window,
    request: NwWindowMoveByRequest,
) -> BridgeResponse<()> {
    println!(
        "[taurin:nw-window] move_by requested: x_delta={}, y_delta={}",
        request.x_delta, request.y_delta
    );

    let scale_factor = match window.scale_factor() {
        Ok(scale_factor) => scale_factor,
        Err(error) => {
            return BridgeResponse::error("window_scale_factor_failed", error.to_string());
        }
    };
    let position = match window.outer_position() {
        Ok(position) => position.to_logical::<f64>(scale_factor),
        Err(error) => {
            return BridgeResponse::error("window_position_read_failed", error.to_string());
        }
    };

    move_window_to(
        &window,
        position.x + request.x_delta,
        position.y + request.y_delta,
    )
}

#[tauri::command]
pub fn nw_window_move_to(
    window: tauri::Window,
    request: NwWindowMoveToRequest,
) -> BridgeResponse<()> {
    println!(
        "[taurin:nw-window] move_to requested: x={}, y={}",
        request.x, request.y
    );

    move_window_to(&window, request.x, request.y)
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

fn resize_window_to(window: &tauri::Window, width: f64, height: f64) -> BridgeResponse<()> {
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return BridgeResponse::error(
            "invalid_window_size",
            format!("invalid window size: {width}x{height}"),
        );
    }

    println!("[taurin:nw-window] applying size: {width}x{height}");

    match window.set_size(tauri::LogicalSize::new(width, height)) {
        Ok(()) => {
            println!("[taurin:nw-window] size applied");
            BridgeResponse::ok(())
        }
        Err(error) => BridgeResponse::error("window_resize_failed", error.to_string()),
    }
}

fn move_window_to(window: &tauri::Window, x: f64, y: f64) -> BridgeResponse<()> {
    if !x.is_finite() || !y.is_finite() {
        return BridgeResponse::error(
            "invalid_window_position",
            format!("invalid window position: {x},{y}"),
        );
    }

    println!("[taurin:nw-window] applying position: {x},{y}");

    match window.set_position(tauri::LogicalPosition::new(x, y)) {
        Ok(()) => {
            println!("[taurin:nw-window] position applied");
            BridgeResponse::ok(())
        }
        Err(error) => BridgeResponse::error("window_move_failed", error.to_string()),
    }
}
