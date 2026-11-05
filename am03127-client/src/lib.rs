pub mod config;
mod transport;

use am03127_commands::{page::Page, realtime_clock::DateTime, schedule::Schedule};
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BuildInfo {
    pub version: String,
    pub build_time: String,
    pub build_date: String,
}

/// Async HTTP client for a single AM03127 panel.
///
/// Construct with [`PanelClient::new`] and pass the panel's IP address.
/// Cloning is cheap; on native targets the connection pool is shared.
#[derive(Clone)]
pub struct PanelClient {
    http: transport::Http,
    base_url: String,
}

impl PanelClient {
    pub fn new(address: &str) -> Self {
        Self {
            http: transport::Http::new(),
            base_url: format!("http://{address}"),
        }
    }

    // ── Status ────────────────────────────────────────────────────────────────

    pub async fn get_status(&self) -> Result<BuildInfo> {
        self.http.get::<BuildInfo>(&self.url("/status")).await
    }

    // ── Clock ─────────────────────────────────────────────────────────────────

    pub async fn set_clock(&self, dt: &DateTime) -> Result<()> {
        self.http.post(&self.url("/clock"), Some(dt)).await
    }

    // ── Pages ─────────────────────────────────────────────────────────────────

    pub async fn get_page(&self, id: char) -> Result<Page> {
        self.http
            .get::<Page>(&self.url(format!("/page/{id}")))
            .await
    }

    pub async fn set_page(&self, page: &Page) -> Result<()> {
        self.http
            .post(&self.url(format!("/page/{}", page.id)), Some(page))
            .await
    }

    pub async fn delete_page(&self, id: char) -> Result<()> {
        self.http.delete(&self.url(format!("/page/{id}"))).await
    }

    pub async fn get_pages(&self) -> Result<Vec<Page>> {
        self.http.get::<Vec<Page>>(&self.url("/pages")).await
    }

    pub async fn set_pages(&self, pages: &[Page]) -> Result<()> {
        self.http.post(&self.url("/pages"), Some(pages)).await
    }

    // ── Schedules ─────────────────────────────────────────────────────────────

    pub async fn get_schedule(&self, id: char) -> Result<Schedule> {
        self.http
            .get::<Schedule>(&self.url(format!("/schedule/{id}")))
            .await
    }

    pub async fn set_schedule(&self, schedule: &Schedule) -> Result<()> {
        self.http
            .post(
                &self.url(format!("/schedule/{}", schedule.id)),
                Some(schedule),
            )
            .await
    }

    pub async fn delete_schedule(&self, id: char) -> Result<()> {
        self.http.delete(&self.url(format!("/schedule/{id}"))).await
    }

    pub async fn get_schedules(&self) -> Result<Vec<Schedule>> {
        self.http
            .get::<Vec<Schedule>>(&self.url("/schedules"))
            .await
    }

    pub async fn set_schedules(&self, schedules: &[Schedule]) -> Result<()> {
        self.http
            .post(&self.url("/schedules"), Some(schedules))
            .await
    }

    // ── Control ───────────────────────────────────────────────────────────────

    /// Deletes all pages and schedules from the panel.
    pub async fn reset(&self) -> Result<()> {
        self.http.post::<()>(&self.url("/reset"), None).await
    }

    /// Uploads new firmware via OTA. The panel reboots automatically after a
    /// successful upload.
    pub async fn update_firmware(&self, firmware: &[u8]) -> Result<()> {
        self.http.put_bytes(&self.url("/ota"), firmware).await
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn url(&self, path: impl AsRef<str>) -> String {
        format!("{}{}", self.base_url, path.as_ref())
    }
}
