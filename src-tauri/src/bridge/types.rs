use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeResponse<T>
where
    T: Serialize,
{
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<BridgeError>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeError {
    pub code: &'static str,
    pub message: String,
    pub details: Option<String>,
}

impl<T> BridgeResponse<T>
where
    T: Serialize,
{
    pub fn ok(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(BridgeError {
                code,
                message: message.into(),
                details: None,
            }),
        }
    }
}
