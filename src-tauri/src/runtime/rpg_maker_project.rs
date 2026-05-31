use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::protocol::rpg_maker_assets;

const FALLBACK_RPG_MAKER_TITLE: &str = "taurin";
const FALLBACK_WINDOW_WIDTH: f64 = 800.0;
const FALLBACK_WINDOW_HEIGHT: f64 = 600.0;

pub struct RpgMakerProject {
    www_dir: PathBuf,
    initial_title: String,
    window_width: f64,
    window_height: f64,
    window_icon: Option<PathBuf>,
    index_url: tauri::Url,
}

impl RpgMakerProject {
    pub fn discover() -> Result<Self, Box<dyn std::error::Error>> {
        let www_dir = rpg_maker_www_dir()?;
        let package = rpg_maker_package(&www_dir);
        let initial_title = rpg_maker_initial_title(&www_dir, package.as_ref());
        let (window_width, window_height) = rpg_maker_window_size(package.as_ref());
        let window_icon = rpg_maker_window_icon(&www_dir, package.as_ref());
        let index_url = rpg_maker_assets::index_url()?;

        Ok(Self {
            www_dir,
            initial_title,
            window_width,
            window_height,
            window_icon,
            index_url,
        })
    }

    pub fn www_dir(&self) -> &Path {
        &self.www_dir
    }

    pub fn initial_title(&self) -> &str {
        &self.initial_title
    }

    pub fn window_width(&self) -> f64 {
        self.window_width
    }

    pub fn window_height(&self) -> f64 {
        self.window_height
    }

    pub fn window_icon(&self) -> Option<&Path> {
        self.window_icon.as_deref()
    }

    pub fn index_url(&self) -> &tauri::Url {
        &self.index_url
    }
}

#[derive(Deserialize)]
struct RpgMakerPackage {
    window: Option<RpgMakerPackageWindow>,
}

#[derive(Deserialize)]
struct RpgMakerPackageWindow {
    title: Option<String>,
    width: Option<f64>,
    height: Option<f64>,
    icon: Option<String>,
}

fn rpg_maker_www_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let executable_path = env::current_exe()?;
    let executable_dir = executable_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "failed to resolve the RPG Maker runtime executable directory",
        )
    })?;
    let www_dir = executable_dir.join("www");

    if !www_dir.join("index.html").is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing RPG Maker MV entrypoint: {}",
                www_dir.join("index.html").display()
            ),
        )
        .into());
    }

    Ok(www_dir)
}

fn rpg_maker_package(www_dir: &Path) -> Option<RpgMakerPackage> {
    fs::read_to_string(www_dir.join("package.json"))
        .ok()
        .and_then(|package| serde_json::from_str(&package).ok())
}

fn rpg_maker_initial_title(www_dir: &Path, package: Option<&RpgMakerPackage>) -> String {
    if let Some(title) = package
        .and_then(|package| package.window.as_ref())
        .and_then(|window| window.title.as_deref())
        .map(str::trim)
        .filter(|title| !title.is_empty())
    {
        return title.to_string();
    }

    fs::read_to_string(www_dir.join("index.html"))
        .ok()
        .and_then(|html| extract_html_title(&html))
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| FALLBACK_RPG_MAKER_TITLE.to_string())
}

fn rpg_maker_window_size(package: Option<&RpgMakerPackage>) -> (f64, f64) {
    let window = package.and_then(|package| package.window.as_ref());
    let width = window
        .and_then(|window| window.width)
        .filter(|width| width.is_finite() && *width > 0.0)
        .unwrap_or(FALLBACK_WINDOW_WIDTH);
    let height = window
        .and_then(|window| window.height)
        .filter(|height| height.is_finite() && *height > 0.0)
        .unwrap_or(FALLBACK_WINDOW_HEIGHT);

    (width, height)
}

fn rpg_maker_window_icon(www_dir: &Path, package: Option<&RpgMakerPackage>) -> Option<PathBuf> {
    package
        .and_then(|package| package.window.as_ref())
        .and_then(|window| window.icon.as_deref())
        .map(str::trim)
        .filter(|icon| !icon.is_empty())
        .map(|icon| www_dir.join(icon))
        .filter(|icon| icon.is_file())
}

fn extract_html_title(html: &str) -> Option<String> {
    let lower_html = html.to_lowercase();
    let title_start = lower_html.find("<title")?;
    let content_start = html[title_start..].find('>')? + title_start + 1;
    let content_end = lower_html[content_start..].find("</title>")? + content_start;

    Some(decode_html_entities(
        html[content_start..content_end].trim(),
    ))
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
