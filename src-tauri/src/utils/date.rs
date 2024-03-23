use chrono::Local;

/// <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
pub const TIMESTAMP: &str = "%F %T%.3f %:z";

pub fn now() -> String {
  Local::now().format(TIMESTAMP).to_string()
}
