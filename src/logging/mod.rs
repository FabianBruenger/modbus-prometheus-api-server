use crate::{utils};

pub fn print_start(config: String){
    log::info!("Starting server...");
    log::info!("Initializing  Clients...");
    log::info!("Trying to read local stored config files...");
    log::info!("Location for config files: {}", &config);
    // Get all local config files
    if let Ok(config_files) = utils::get_local_config_files(config, true){
        log::info!("Found {} config files, so {} client/s will be initialized.", config_files.len(),config_files.len());
        for config_file in config_files{
            log::info!("Config file: {}", config_file);
        }
    } else {
        log::info!("No config files found. No clients will be initialized.");
    }
}