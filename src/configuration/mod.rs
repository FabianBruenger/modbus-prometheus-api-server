use config::Config;
#[derive(Debug, Default, serde::Deserialize, PartialEq)]
pub struct Args {
    log_level: String,
    /// Web server port
    port: u16,
    /// Interval in milliseconds to read data from modbus clients
    read_data_interval_ms: u16,
    /// local path for the configuration paths
    config_path: String,
}

impl Args {
    pub fn new() -> Self {
        // Read configuration file
        let config = match Config::builder()
            .add_source(config::File::with_name("setup"))
            .build()
        {
            Ok(config) => config,
            Err(e) => {
                panic!("Error reading configuration file: {}", e);
            }
        };
        let config = match config.try_deserialize::<Self>() {
            Ok(config) => config,
            Err(e) => {
                panic!("Error deserializing file: {}", e);
            }
        };
        Args {
            log_level: config.log_level,
            port: config.port,
            read_data_interval_ms: config.read_data_interval_ms,
            config_path: config.config_path,
        }
    }
    // Write getter for all entries
    pub fn get_log_level(&self) -> &str {
        &self.log_level
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
    pub fn get_read_data_interval_ms(&self) -> u16 {
        self.read_data_interval_ms
    }
    pub fn get_config_path(&self) -> &str {
        &self.config_path
    }
}
