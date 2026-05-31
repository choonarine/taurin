use std::sync::Mutex;

use tauri::{image::Image, WebviewUrl, WebviewWindowBuilder};

use crate::{
    audio::KiraAudioEngine,
    bridge::commands,
    protocol::rpg_maker_assets,
    runtime::{rpg_maker_project::RpgMakerProject, system_diagnostics},
};

const NW_WINDOW_INIT_SCRIPT: &str = r#"
(() => {
  const isRpgMakerPage =
    window.location.protocol === 'rpgmv:' ||
    window.location.hostname === 'rpgmv.localhost';

  if (!isRpgMakerPage) {
    return;
  }

  const invoke = (command, args) => {
    console.info('[taurin:nw-window]', command, args);

    if (window.__TAURI__?.core?.invoke) {
      return window.__TAURI__.core.invoke(command, args).catch((error) => {
        console.error(`NW.js window command failed: ${command}`, error);
      });
    }

    if (window.__TAURI_INTERNALS__?.invoke) {
      return window.__TAURI_INTERNALS__.invoke(command, args).catch((error) => {
        console.error(`NW.js window command failed: ${command}`, error);
      });
    }

    console.warn('[taurin:nw-window] Tauri invoke API is not available');
      return Promise.resolve();
  };

  const toFiniteNumber = (value) => {
    const number = Number(value);
    return Number.isFinite(number) ? number : 0;
  };

  const updateGraphics = () => {
    if (!window.Graphics || typeof window.Graphics._updateAllElements !== 'function') {
      return;
    }

    window.Graphics._updateAllElements();
  };

  window.resizeBy = (widthDelta, heightDelta) => {
    console.info('[taurin:nw-window] resizeBy intercepted', widthDelta, heightDelta);
    void invoke('nw_window_resize_by', {
      request: {
        widthDelta: toFiniteNumber(widthDelta),
        heightDelta: toFiniteNumber(heightDelta),
      },
    }).then(updateGraphics);
  };

  window.resizeTo = (width, height) => {
    console.info('[taurin:nw-window] resizeTo intercepted', width, height);
    void invoke('nw_window_resize_to', {
      request: {
        width: toFiniteNumber(width),
        height: toFiniteNumber(height),
      },
    }).then(updateGraphics);
  };

  window.moveBy = (xDelta, yDelta) => {
    console.info('[taurin:nw-window] moveBy intercepted', xDelta, yDelta);
    void invoke('nw_window_move_by', {
      request: {
        xDelta: toFiniteNumber(xDelta),
        yDelta: toFiniteNumber(yDelta),
      },
    });
  };

  window.moveTo = (x, y) => {
    console.info('[taurin:nw-window] moveTo intercepted', x, y);
    void invoke('nw_window_move_to', {
      request: {
        x: toFiniteNumber(x),
        y: toFiniteNumber(y),
      },
    });
  };
})();
"#;

pub fn run() {
    system_diagnostics::log_startup_environment();

    let project = RpgMakerProject::discover().expect("failed to resolve RPG Maker project");
    let www_dir = project.www_dir().to_path_buf();
    let audio_engine =
        KiraAudioEngine::new(www_dir.clone()).expect("failed to initialize audio engine");
    let index_url = project.index_url().clone();
    let initial_title = project.initial_title().to_string();
    let window_width = project.window_width();
    let window_height = project.window_height();
    let window_icon = project.window_icon().map(ToOwned::to_owned);

    println!(
        "[taurin] starting RPG Maker runtime: www={}, title={:?}, initial_size={}x{}",
        www_dir.display(),
        initial_title,
        window_width,
        window_height
    );

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
            commands::nw_window_move_by,
            commands::nw_window_move_to,
            commands::nw_window_resize_by,
            commands::nw_window_resize_to,
            commands::sync_rpg_maker_window_title
        ])
        .setup(move |app| {
            let mut window_builder = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::CustomProtocol(index_url.clone()),
            )
            .title(initial_title.clone())
            .inner_size(window_width, window_height)
            .resizable(false)
            .initialization_script(NW_WINDOW_INIT_SCRIPT);

            if let Some(icon) = window_icon.as_ref() {
                println!("[taurin] applying window icon: {}", icon.display());
                window_builder = window_builder.icon(Image::from_path(icon)?)?;
            }

            let window = window_builder.build()?;
            println!("[taurin] main window created");
            system_diagnostics::log_window_displays(&window);

            #[cfg(debug_assertions)]
            {
                println!("[taurin] opening WebView devtools");
                window.open_devtools();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
