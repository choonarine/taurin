use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use crate::protocol::rpg_maker_assets;

const FALLBACK_RPG_MAKER_TITLE: &str = "taurin";

pub struct RpgMakerProject {
    www_dir: PathBuf,
    initial_title: String,
    index_url: tauri::Url,
}

impl RpgMakerProject {
    pub fn discover() -> Result<Self, Box<dyn std::error::Error>> {
        let www_dir = rpg_maker_www_dir()?;
        let initial_title = rpg_maker_initial_title(&www_dir);
        let index_url = rpg_maker_assets::index_url()?;

        Ok(Self {
            www_dir,
            initial_title,
            index_url,
        })
    }

    pub fn www_dir(&self) -> &Path {
        &self.www_dir
    }

    pub fn initial_title(&self) -> &str {
        &self.initial_title
    }

    pub fn index_url(&self) -> &tauri::Url {
        &self.index_url
    }
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

fn rpg_maker_initial_title(www_dir: &Path) -> String {
    fs::read_to_string(www_dir.join("index.html"))
        .ok()
        .and_then(|html| extract_html_title(&html))
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| FALLBACK_RPG_MAKER_TITLE.to_string())
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
