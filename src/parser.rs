use regex::Regex;
use crate::model::LogEntry;
use std::sync::OnceLock;

static LOG_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn parse_line(line: &str) -> Option<LogEntry> {
    let re = LOG_REGEX.get_or_init(|| {
        Regex::new(r"^\[(.*?)\]\s+(\S+)\s+query\s+(\S+),\s+type\s+(\d+),\s+time\s+(\d+)ms,\s+speed:\s+(\S+)ms,\s+group\s+(\S+),\s+result\s+(.*)$").expect("Invalid regex")
    });

    let caps = re.captures(line)?;

    Some(LogEntry {
        timestamp: caps.get(1)?.as_str().to_string(),
        client_ip: caps.get(2)?.as_str().to_string(),
        domain: caps.get(3)?.as_str().to_string(),
        qtype: caps.get(4)?.as_str().parse().ok()?,
        time_ms: caps.get(5)?.as_str().parse().ok()?,
        speed_ms: caps.get(6)?.as_str().parse().ok()?,
        group: caps.get(7)?.as_str().to_string(),
        result: caps.get(8)?.as_str().to_string(),
    })
}
