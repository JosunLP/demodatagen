//! Synthetic log-file generator.
//!
//! Produces realistic-looking log lines in one of three styles: Apache/NGINX
//! common access logs, BSD syslog, or structured JSON logs.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::faker;
use crate::error::{GenResult, GenerationError};
use rand::{Rng, RngExt};

/// Generator for log files.
pub struct LogGenerator;

const METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD"];
const PATHS: &[&str] = &[
    "/",
    "/index.html",
    "/api/v1/users",
    "/api/v1/orders",
    "/login",
    "/logout",
    "/static/app.js",
    "/static/style.css",
    "/health",
    "/metrics",
    "/favicon.ico",
    "/products",
    "/cart/checkout",
];
const STATUSES: &[u32] = &[
    200, 200, 200, 201, 204, 301, 302, 304, 400, 401, 403, 404, 500, 502, 503,
];
const LEVELS: &[&str] = &["DEBUG", "INFO", "INFO", "INFO", "WARN", "ERROR"];
const PROCESSES: &[&str] = &[
    "sshd", "cron", "kernel", "systemd", "nginx", "postfix", "dockerd",
];
const MONTHS: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl Generator for LogGenerator {
    fn format_name(&self) -> &str {
        "LOG"
    }

    fn file_extension(&self) -> &str {
        "log"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (lines, style) = match &config.format_options {
            FormatOptions::Log { lines, style } => (*lines, style.clone()),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "LOG generator requires Log options".to_string(),
                ))
            }
        };

        let rng = &mut config.rng;
        let mut out = String::new();
        for _ in 0..lines {
            let line = match style.to_lowercase().as_str() {
                "syslog" => syslog_line(rng),
                "json" => json_line(rng),
                _ => apache_line(rng),
            };
            out.push_str(&line);
            out.push('\n');
        }
        Ok(out.into_bytes())
    }
}

/// Apache/NGINX common log format.
fn apache_line<R: Rng>(rng: &mut R) -> String {
    let ip = faker::ipv4(rng);
    let day = rng.random_range(1..=28);
    let month = MONTHS[rng.random_range(0..12)];
    let year = rng.random_range(2020..=2025);
    let (h, m, s) = (
        rng.random_range(0..24),
        rng.random_range(0..60),
        rng.random_range(0..60),
    );
    let method = METHODS[rng.random_range(0..METHODS.len())];
    let path = PATHS[rng.random_range(0..PATHS.len())];
    let status = STATUSES[rng.random_range(0..STATUSES.len())];
    let size = rng.random_range(0..1_048_576u32);
    format!(
        "{ip} - - [{day:02}/{month}/{year}:{h:02}:{m:02}:{s:02} +0000] \"{method} {path} HTTP/1.1\" {status} {size}"
    )
}

/// BSD syslog line.
fn syslog_line<R: Rng>(rng: &mut R) -> String {
    let month = MONTHS[rng.random_range(0..12)];
    let day = rng.random_range(1..=28);
    let (h, m, s) = (
        rng.random_range(0..24),
        rng.random_range(0..60),
        rng.random_range(0..60),
    );
    let host = format!("host{:02}", rng.random_range(1..20));
    let process = PROCESSES[rng.random_range(0..PROCESSES.len())];
    let pid = rng.random_range(100..30000);
    let messages = [
        "connection accepted",
        "session opened for user root",
        "disk usage at 82%",
        "service restarted",
        "authentication failure",
        "request completed in 23ms",
    ];
    let msg = messages[rng.random_range(0..messages.len())];
    format!("{month} {day:2} {h:02}:{m:02}:{s:02} {host} {process}[{pid}]: {msg}")
}

/// Structured JSON log line.
fn json_line<R: Rng>(rng: &mut R) -> String {
    let level = LEVELS[rng.random_range(0..LEVELS.len())];
    let ts = faker::datetime(rng);
    let latency = rng.random_range(1..2000);
    let path = PATHS[rng.random_range(0..PATHS.len())];
    let status = STATUSES[rng.random_range(0..STATUSES.len())];
    let value = serde_json::json!({
        "ts": ts,
        "level": level,
        "msg": "request handled",
        "path": path,
        "status": status,
        "latency_ms": latency,
    });
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::log_config;

    #[test]
    fn test_log_line_count() {
        let mut config = log_config(50, "apache");
        let result = LogGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert_eq!(text.lines().count(), 50);
    }

    #[test]
    fn test_apache_format() {
        let mut config = log_config(5, "apache");
        let result = LogGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("HTTP/1.1"));
    }

    #[test]
    fn test_json_log_parses() {
        let mut config = log_config(5, "json");
        let result = LogGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        for line in text.lines() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(v.get("level").is_some());
        }
    }

    #[test]
    fn test_syslog_format() {
        let mut config = log_config(5, "syslog");
        let result = LogGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains('['));
    }
}
