pub mod html;
pub mod json;

pub use html::{escape_html, generate_html_report};
pub use json::{generate_json_report, JsonReportError};
