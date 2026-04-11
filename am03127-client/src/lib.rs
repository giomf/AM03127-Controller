use am03127::{page::Page, realtime_clock::DateTime, schedule::Schedule};
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
/// The underlying [`reqwest::Client`] is cloned cheaply; the actual connection
/// pool is shared.
#[derive(Clone)]
pub struct PanelClient {
    client: reqwest::Client,
    base_url: String,
}

impl PanelClient {
    pub fn new(address: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: format!("http://{address}"),
        }
    }

    // ── Status ────────────────────────────────────────────────────────────────

    pub async fn get_status(&self) -> Result<BuildInfo> {
        let info = self
            .client
            .get(self.url("/status"))
            .send()
            .await?
            .error_for_status()?
            .json::<BuildInfo>()
            .await?;
        Ok(info)
    }

    // ── Clock ─────────────────────────────────────────────────────────────────

    pub async fn set_clock(&self, dt: &DateTime) -> Result<()> {
        self.client
            .post(self.url("/clock"))
            .json(dt)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    // ── Pages ─────────────────────────────────────────────────────────────────

    pub async fn get_page(&self, id: char) -> Result<Page> {
        let page = self
            .client
            .get(self.url(format!("/page/{id}")))
            .send()
            .await?
            .error_for_status()?
            .json::<Page>()
            .await?;
        Ok(page)
    }

    pub async fn set_page(&self, page: &Page) -> Result<()> {
        self.client
            .post(self.url(format!("/page/{}", page.id)))
            .json(page)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn delete_page(&self, id: char) -> Result<()> {
        self.client
            .delete(self.url(format!("/page/{id}")))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_pages(&self) -> Result<Vec<Page>> {
        let pages = self
            .client
            .get(self.url("/pages"))
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Page>>()
            .await?;
        Ok(pages)
    }

    pub async fn set_pages(&self, pages: &[Page]) -> Result<()> {
        self.client
            .post(self.url("/pages"))
            .json(pages)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    // ── Schedules ─────────────────────────────────────────────────────────────

    pub async fn get_schedule(&self, id: char) -> Result<Schedule> {
        let schedule = self
            .client
            .get(self.url(format!("/schedule/{id}")))
            .send()
            .await?
            .error_for_status()?
            .json::<Schedule>()
            .await?;
        Ok(schedule)
    }

    pub async fn set_schedule(&self, schedule: &Schedule) -> Result<()> {
        self.client
            .post(self.url(format!("/schedule/{}", schedule.id)))
            .json(schedule)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn delete_schedule(&self, id: char) -> Result<()> {
        self.client
            .delete(self.url(format!("/schedule/{id}")))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_schedules(&self) -> Result<Vec<Schedule>> {
        let schedules = self
            .client
            .get(self.url("/schedules"))
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Schedule>>()
            .await?;
        Ok(schedules)
    }

    pub async fn set_schedules(&self, schedules: &[Schedule]) -> Result<()> {
        self.client
            .post(self.url("/schedules"))
            .json(schedules)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    // ── Control ───────────────────────────────────────────────────────────────

    /// Deletes all pages and schedules from the panel.
    pub async fn reset(&self) -> Result<()> {
        self.client
            .post(self.url("/reset"))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Uploads new firmware via OTA. The panel reboots automatically after a
    /// successful upload.
    pub async fn update_firmware(&self, firmware: &[u8]) -> Result<()> {
        let len = firmware.len();
        self.client
            .put(self.url("/ota"))
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", len)
            .body(firmware.to_vec())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn url(&self, path: impl AsRef<str>) -> String {
        format!("{}{}", self.base_url, path.as_ref())
    }
}
