//! Minimal HTTP transport: reqwest on native targets, browser fetch on wasm.

use anyhow::{Result, anyhow};
use serde::{Serialize, de::DeserializeOwned};

/// Error for a non-2xx response. The firmware puts the reason in the body as plain text.
fn status_error(status: u16, body: &str) -> anyhow::Error {
    match body.trim() {
        "" => anyhow!("HTTP {status}"),
        reason => anyhow!("{reason} (HTTP {status})"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct Http(reqwest::Client);

#[cfg(not(target_arch = "wasm32"))]
impl Http {
    pub fn new() -> Self {
        Self(reqwest::Client::new())
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        Ok(check(self.0.get(url).send().await?).await?.json().await?)
    }

    pub async fn post<B: Serialize + ?Sized>(&self, url: &str, body: Option<&B>) -> Result<()> {
        let mut req = self.0.post(url);
        if let Some(body) = body {
            req = req.json(body);
        }
        check(req.send().await?).await?;
        Ok(())
    }

    pub async fn delete(&self, url: &str) -> Result<()> {
        check(self.0.delete(url).send().await?).await?;
        Ok(())
    }

    pub async fn put_bytes(&self, url: &str, bytes: &[u8]) -> Result<()> {
        let req = self
            .0
            .put(url)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", bytes.len())
            .body(bytes.to_vec());
        check(req.send().await?).await?;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn check(resp: reqwest::Response) -> Result<reqwest::Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    Err(status_error(status.as_u16(), &resp.text().await.unwrap_or_default()))
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct Http;

#[cfg(target_arch = "wasm32")]
impl Http {
    pub fn new() -> Self {
        Self
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = check(gloo_net::http::Request::get(url).send().await?).await?;
        Ok(resp.json().await?)
    }

    pub async fn post<B: Serialize + ?Sized>(&self, url: &str, body: Option<&B>) -> Result<()> {
        let req = gloo_net::http::Request::post(url);
        let resp = match body {
            Some(body) => req.json(body)?.send().await?,
            None => req.send().await?,
        };
        check(resp).await?;
        Ok(())
    }

    pub async fn delete(&self, url: &str) -> Result<()> {
        check(gloo_net::http::Request::delete(url).send().await?).await?;
        Ok(())
    }

    pub async fn put_bytes(&self, url: &str, bytes: &[u8]) -> Result<()> {
        let req = gloo_net::http::Request::put(url)
            .header("Content-Type", "application/octet-stream")
            .body(js_sys::Uint8Array::from(bytes))?;
        check(req.send().await?).await?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
async fn check(resp: gloo_net::http::Response) -> Result<gloo_net::http::Response> {
    if resp.ok() {
        return Ok(resp);
    }
    Err(status_error(resp.status(), &resp.text().await.unwrap_or_default()))
}
