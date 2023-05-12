#[derive(Debug, Default, serde::Deserialize, PartialEq)]
pub struct Args {
    pub log_level: String,
    /// Web server port
    pub port: u16,
}