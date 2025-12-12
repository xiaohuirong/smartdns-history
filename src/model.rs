use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub client_ip: String,
    pub domain: String,
    pub qtype: u16,
    pub time_ms: u64,
    pub speed_ms: f64,
    pub group: String,
    pub result: String,
}
