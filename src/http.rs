use ehttp::{Headers, Request};
use serde::Serialize;

#[cfg(target_arch = "wasm32")]
use ehttp::Mode;


    /// Create a `POST` request with the given url and json body.
#[allow(clippy::needless_pass_by_value)]
pub fn put_json<T>(url: impl ToString, body: &T) -> serde_json::error::Result<Request>
where
    T: ?Sized + Serialize,
{
    Ok(Request {
        method: "PUT".to_owned(),
        url: url.to_string(),
        body: serde_json::to_string(body)?.into_bytes(),
        headers: Headers::new(&[("Accept", "*/*"), ("Content-Type", "application/json")]),
        #[cfg(target_arch = "wasm32")]
        mode: Mode::default(),
    })
}