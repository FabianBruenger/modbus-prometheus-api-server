use crate::errors::impls::ErrorRuntime;
use crate::routes::Client;
use regex::Regex;
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;

// Write JSON to local file. Filename <client name>.json
pub fn write_config(client: &Client, config: &str) -> Result<(), ErrorRuntime> {
    let config_name = format!("{}.json", &client.name);
    let config_path = format!("{}/{}", config, &config_name);
    
    let config_json = match serde_json::to_string_pretty(&client){
        Ok(config_json) => config_json,
        Err(_) => return Err(ErrorRuntime::JSONSerializeError),
    };
    let mut config_file = match File::create(&config_path){
        Ok(config_file) => config_file,
        Err(_) => return Err(ErrorRuntime::FSFileCreateError),
    };
    if let Err(_) = config_file.write_all(config_json.as_bytes()) {
        return Err(ErrorRuntime::FSFileCreateError);
    }
    log::info!(
        "Created new client via POST /clients. Stored the client config to {}",
        &config_path);
    Ok(())
}
// Delete JSON to local file. Filename <client name>.json
pub fn delete_config(name: &str, config: &str) -> Result<(), ErrorRuntime> {
    let config_name = format!("{}.json", name);
    let config_path = format!("{}/{}", config, &config_name);
    if let Err(_) = fs::remove_file(&config_path) {
        return Err(ErrorRuntime::FSFileDeleteError);
    }
    log::info!(
        "Deleted client via DELETE /clients/<name>. Removed the client config from {}",
        &config_path);
    Ok(())
}
// Check if the JSON config has the right format. All string should be lowercase and have a underscore for seperation
pub fn check_client_strings(input: &Value) -> Result<(), ErrorRuntime> {
    let value = serde_json::to_value(input).unwrap();

    match value {
        Value::String(s) => {
            let regex = Regex::new(r"^[a-z0-9_]+$").unwrap();
            if !regex.is_match(&s) {
                return Err(ErrorRuntime::RegexError);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                check_client_strings(&v)?;
            }
        }
        Value::Object(obj) => {
            for (key, v) in obj {
                if key != "ip_address" {
                    check_client_strings(&v)?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}
// Check if the client already exists
pub fn check_fs_config(client_file_name: &str, config: &str) -> Result<(), ErrorRuntime> {
    log::info!(
        "Trying to create new client via POST /clients. Checking if the client already exists from local JSON."
    );

    if let Ok(client_files_local) = get_local_config_files(config) {
        if client_files_local.len() > 0 {
            if client_files_local.contains(&client_file_name.to_string()) {
                return Err(ErrorRuntime::ClientExists);
            }
        }
    }

    Ok(())
}
// Get local config files
pub fn get_local_config_files(config: &str) -> Result<Vec<String>, ErrorRuntime> {
    let mut config_files: Vec<String> = Vec::new();

    let config_list: fs::ReadDir = match fs::read_dir(config) {
        Ok(config_list) => config_list,
        Err(_) => return Err(ErrorRuntime::FSReadDirError),
    };

    for config in config_list {
        let dir_entry = match config {
            Ok(dir_entry) => dir_entry,
            Err(_) => return Err(ErrorRuntime::FSDirEntryError),
        };
        config_files.push(dir_entry.file_name().to_str().unwrap().to_string());
    }

    Ok(config_files)
}
