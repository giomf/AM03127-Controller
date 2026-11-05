//! Response of the firmware's `GET /status`.

use alloc::{format, string::String, vec, vec::Vec};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Status {
    /// Short git hash of the build, `-dirty` if the tree had uncommitted changes
    pub version: String,
    /// Build time (UTC)
    pub build_time: String,
    /// Build date (UTC)
    pub build_date: String,
    /// Seconds since the controller started
    pub uptime_secs: u64,
    /// Wi-Fi signal strength in dBm, `None` until measured after connecting
    pub rssi: Option<i32>,
    /// Free heap in bytes
    pub heap_free: u32,
    /// Used heap in bytes
    pub heap_used: u32,
    /// Whether the panel answered the last command, `None` if none was sent since start
    pub panel_ok: Option<bool>,
}

impl Status {
    /// Uptime in its two largest units, e.g. "2 d 3 h" or "5 min".
    pub fn uptime(&self) -> String {
        let secs = self.uptime_secs;
        let (days, hours, minutes) = (secs / 86_400, secs / 3_600 % 24, secs / 60 % 60);
        match (days, hours) {
            (0, 0) => format!("{minutes} min"),
            (0, _) => format!("{hours} h {minutes} min"),
            _ => format!("{days} d {hours} h"),
        }
    }

    /// Short texts for the health fields, e.g. `["up 2 d 3 h", "Wi-Fi -61 dBm"]`.
    pub fn details(&self) -> Vec<String> {
        let mut details = vec![format!("up {}", self.uptime())];
        if let Some(rssi) = self.rssi {
            details.push(format!("Wi-Fi {rssi} dBm"));
        }
        details.push(format!("{} KB free", self.heap_free / 1024));
        match self.panel_ok {
            Some(true) => details.push("panel OK".into()),
            Some(false) => details.push("panel not responding".into()),
            None => {}
        }
        details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details() {
        let status = Status {
            uptime_secs: 2 * 86_400 + 3 * 3_600 + 59,
            rssi: Some(-61),
            heap_free: 52 * 1024,
            panel_ok: Some(false),
            ..Default::default()
        };
        assert_eq!(
            status.details(),
            ["up 2 d 3 h", "Wi-Fi -61 dBm", "52 KB free", "panel not responding"]
        );
        assert_eq!(Status::default().details(), ["up 0 min", "0 KB free"]);
    }
}
