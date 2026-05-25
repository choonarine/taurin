use std::sync::Mutex;

use tauri::{WebviewUrl, WebviewWindowBuilder};

use crate::{
    audio::KiraAudioEngine, bridge::commands, protocol::rpg_maker_assets,
    runtime::rpg_maker_project::RpgMakerProject,
};

pub fn run() {
    let project = RpgMakerProject::discover().expect("failed to resolve RPG Maker project");
    let www_dir = project.www_dir().to_path_buf();
    let audio_engine =
        KiraAudioEngine::new(www_dir.clone()).expect("failed to initialize audio engine");
    let index_url = project.index_url().clone();
    let initial_title = project.initial_title().to_string();

    tauri::Builder::default()
        .manage(Mutex::new(audio_engine))
        .register_uri_scheme_protocol(
            rpg_maker_assets::RPG_MAKER_PROTOCOL_SCHEME,
            move |_context, request| rpg_maker_assets::serve(&www_dir, &request),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::rpg_maker_audio_fade_out,
            commands::rpg_maker_audio_play,
            commands::rpg_maker_audio_stop,
            commands::sync_rpg_maker_window_title
        ])
        .setup(move |app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::CustomProtocol(index_url.clone()))
                .title(initial_title.clone())
                .inner_size(800.0, 600.0)
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
