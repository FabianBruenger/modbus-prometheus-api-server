use crate::routes::helpers;

pub fn print_start(){
    log::info!("Starting server...");
}

pub fn print_init_clients(){
    log::info!("Initializing  Clients...");
    log::info!("Trying to read local stored config files...");
    log::info!("Location for config files: {}", crate::CLIENT_CONFIG_PATH);
    // Get all local config files
    if let Ok(config_files) = helpers::get_local_config_files(){
        log::info!("Found {} config files, so {} client/s will be initialized.", config_files.len(),config_files.len());
        for config_file in config_files{
            log::info!("Config file: {}", config_file);
        }
    } else {
        log::info!("No config files found. No clients will be initialized.");
    }
}

pub fn print_init_prometheus_metrics(){
    log::info!("Initializing Prometheus Metrics for all initial Clients...");
    log::info!("For the initial local config files, it is assumed they follow the correct standard.");

}