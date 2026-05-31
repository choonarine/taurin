use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

use tauri::http::{header, HeaderName, HeaderValue, Method, Response, StatusCode};

pub const RPG_MAKER_PROTOCOL_SCHEME: &str = "rpgmv";

pub fn index_url() -> Result<tauri::Url, Box<dyn std::error::Error>> {
    tauri::Url::parse(&format!(
        "{RPG_MAKER_PROTOCOL_SCHEME}://localhost/index.html"
    ))
    .map_err(Into::into)
}

pub fn serve(www_dir: &Path, request: &tauri::http::Request<Vec<u8>>) -> Response<Vec<u8>> {
    if request.method() == Method::OPTIONS {
        return response(StatusCode::NO_CONTENT, "text/plain", Vec::new());
    }

    let Some(path) = resolve_rpg_maker_asset_path(www_dir, request.uri().path()) else {
        return response(
            StatusCode::BAD_REQUEST,
            "text/plain",
            b"invalid RPG Maker asset path".to_vec(),
        );
    };

    match fs::read(&path) {
        Ok(contents) => rpg_maker_asset_response(request, &path, contents),
        Err(error)
            if error.kind() == io::ErrorKind::NotFound
                && request.uri().path() == "/VirtualController.js" =>
        {
            response(
                StatusCode::OK,
                "text/javascript; charset=utf-8",
                b"// Optional RPG Maker mobile controller shim for desktop runtime.\n".to_vec(),
            )
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            response(StatusCode::NOT_FOUND, "text/plain", b"not found".to_vec())
        }
        Err(error) => response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "text/plain",
            format!("failed to read RPG Maker asset: {error}").into_bytes(),
        ),
    }
}

fn resolve_rpg_maker_asset_path(www_dir: &Path, request_path: &str) -> Option<PathBuf> {
    let asset_path = request_path.trim_start_matches('/');
    let asset_path = if asset_path.is_empty() {
        "index.html".to_string()
    } else {
        percent_decode(asset_path)?
    };

    let mut safe_asset_path = PathBuf::new();
    for component in Path::new(&asset_path).components() {
        match component {
            Component::Normal(path) => safe_asset_path.push(path),
            Component::CurDir => {}
            _ => return None,
        }
    }

    Some(www_dir.join(safe_asset_path))
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = from_hex(*bytes.get(index + 1)?)?;
            let low = from_hex(*bytes.get(index + 2)?)?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded).ok()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn rpg_maker_asset_response(
    request: &tauri::http::Request<Vec<u8>>,
    path: &Path,
    contents: Vec<u8>,
) -> Response<Vec<u8>> {
    if let Some((start, end)) = request
        .headers()
        .get(header::RANGE)
        .and_then(|range| range.to_str().ok())
        .and_then(|range| parse_range(range, contents.len()))
    {
        let body = contents[start..=end].to_vec();
        return response_with_headers(
            StatusCode::PARTIAL_CONTENT,
            content_type(path),
            body,
            &[
                (
                    header::CONTENT_RANGE,
                    format!("bytes {start}-{end}/{}", contents.len()),
                ),
                (header::CONTENT_LENGTH, (end - start + 1).to_string()),
            ],
        );
    }

    let content_length = contents.len().to_string();
    response_with_headers(
        StatusCode::OK,
        content_type(path),
        contents,
        &[(header::CONTENT_LENGTH, content_length)],
    )
}

fn parse_range(range: &str, length: usize) -> Option<(usize, usize)> {
    let range = range.strip_prefix("bytes=")?;
    let (start, end) = range.split_once('-')?;

    if length == 0 {
        return None;
    }

    let start = if start.is_empty() {
        let suffix_length = end.parse::<usize>().ok()?.min(length);
        length - suffix_length
    } else {
        start.parse::<usize>().ok()?
    };
    let end = if end.is_empty() {
        length - 1
    } else {
        end.parse::<usize>().ok()?.min(length - 1)
    };

    (start <= end).then_some((start, end))
}

fn response(status: StatusCode, content_type: &'static str, body: Vec<u8>) -> Response<Vec<u8>> {
    response_with_headers(status, content_type, body, &[])
}

fn response_with_headers(
    status: StatusCode,
    content_type: &'static str,
    body: Vec<u8>,
    headers: &[(HeaderName, String)],
) -> Response<Vec<u8>> {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        )
        .header(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, OPTIONS"),
        )
        .header(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("*"),
        )
        .header(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));

    for (name, value) in headers {
        builder = builder.header(name, value);
    }

    builder
        .body(body)
        .expect("failed to build RPG Maker asset response")
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ogg") => "audio/ogg",
        Some("m4a") => "audio/mp4",
        Some("ttf") => "font/ttf",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}
