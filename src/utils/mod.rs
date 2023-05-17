use crate::clients::Client;
use crate::errors::impls::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio_modbus::prelude::*;
use serde_json::Value;
use regex::Regex;
use std::fs::{File, self};
use std::io::Write;

pub async fn create_ctx(
    client: &Client,
) -> Result<tokio_modbus::client::Context, ErrorRuntimeNoRejection> {
    log::debug!("Creating ipaddress: {}", &client.ip_address);
    let ipaddress = match client.ip_address.parse::<Ipv4Addr>() {
        Ok(ipv4) => {
            log::debug!("Created ipaddress: {}", &ipv4);
            ipv4
        }
        Err(_) => {
            log::error!(
                "Invalid IP address: {}. Skip reading from this modbus client",
                &client.ip_address
            );
            return Err(ErrorRuntimeNoRejection::InvalidIpAddress);
        }
    };
    log::debug!("Creating socket address:");
    let socket_addr_v4 = SocketAddr::new(IpAddr::V4(ipaddress), client.port);
    log::debug!("Connecting to client: {}", &socket_addr_v4);
    let ctx = match tcp::connect(socket_addr_v4.to_owned()).await {
        Ok(ctx) => ctx,
        Err(_) => {
            log::error!(
                "Could not connect to modbus client: {}. Skip reading from this modbus client",
                &client.ip_address
            );
            return Err(ErrorRuntimeNoRejection::CouldNotConnect);
        }
    };
    Ok(ctx)
}

/// Write JSON to local file. Filename <client name>.json
/// 
/// # Arguments
/// 
/// * `client` - The client struct to write to file
/// * `config` - The main config path. Under this path all and ONLY client JSON configs should be stored.
/// 
/// # Returns
/// 
/// * `Ok(())` - If the client config file was written
/// * `Err(ErrorRuntime::JSONSerializeError)` - If the client could not be serialized. Will be forwarded as warp rejection
/// * `Err(ErrorRuntime::FSFileCreateError)` - If the client config file could not be written. Will be forwarded as warp rejection
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


/// Deletes one specific client config file from the main config path.
/// 
/// # Arguments
/// 
/// * `name` - The name of the client. This is also the name of the config file without the .json extension
/// * `config` - The main config path. Under this path all and ONLY client JSON configs should be stored.
/// 
/// # Returns
/// 
/// * `Ok(())` - If the client config file was deleted
/// * `Err(ErrorRuntime::FSFileDeleteError)` - If the client config file could not be deleted. Will be forwarded as warp rejection
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

/// Check if all strings in the input are valid. Valid strings are lowercase, numbers and underscores.
/// 
/// # Arguments
/// 
/// * `input` - The input to check. This must be an JSON object
/// 
/// # Returns
/// 
/// * `Ok(())` - If all strings are valid
/// * `Err(ErrorRuntime::RegexError)` - If one of the strings is not valid. Will be forwarded as warp rejection
pub fn check_client_strings(input: &Value) -> Result<(), ErrorRuntime> {
    match input {
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

/// Based on the main config path, get all the config files in that path and check if the client already exists.
///
/// # Arguments
///
/// * `client_file_name` - The name of the client config file
/// * `config` - The main config path. Under this path all and ONLY client JSON configs should be stored.
///
/// # Returns
///
/// * `Ok(())` - If the client does not exist
/// * `Err(ErrorRuntime::ClientExists)` - If the client already exists. Will be forwarded as warp rejection
pub fn check_if_client_exist(client_file_name: &str, config: &str) -> Result<(), ErrorRuntime> {
    log::debug!("Check if the provided client name already exists in the config path ",);
    if let Ok(client_files_local) = get_local_config_files(config.to_string(), false) {
        if client_files_local.len() > 0 {
            if client_files_local.contains(&client_file_name.to_string()) {
                return Err(ErrorRuntime::ClientExists);
            }
        }
    }
    Ok(())
}

/// Based on the main config path, get all the config files in that path.
/// If full_path is true, the full path to the config file is returned. Else it will just return the filename.
/// The config files should be JSON files with the client name as filename
/// and should be stored in /etc/modbus-prometheus-api-server/config for linux and Mac OS
///
/// # Arguments
///
/// * `config` - The main config path. Under this path all and ONLY client JSON configs should be stored.
/// * `full_path` - If true, the full path to the config file is returned. Else it will just return the filename.
///
/// # Returns
///
/// * `config_files` - A vector of strings with the full path to the config files
///
/// # Errors
///
/// * `ErrorRuntime::FSReadDirError` - Could not read the main config path, Maybe user issues on OS level
/// * `ErrorRuntime::FSDirEntryError` - Could not read the config file
pub fn get_local_config_files(
    config_path: String,
    full_path: bool,
) -> Result<Vec<String>, ErrorRuntime> {
    let mut config_files: Vec<String> = Vec::new();
    let config_list: fs::ReadDir = match fs::read_dir(config_path) {
        Ok(read_dir) => read_dir,
        Err(_) => return Err(ErrorRuntime::FSReadDirError),
    };
    for config in config_list {
        let dir_entry = match config {
            Ok(dir_entry) => dir_entry,
            Err(_) => return Err(ErrorRuntime::FSDirEntryError),
        };
        if full_path {
            match dir_entry.path().to_str() {
                Some(full_path) => config_files.push(full_path.to_string()),
                None => return Err(ErrorRuntime::FSPathNotUTF8Error),
            };
        } else {
            match dir_entry.file_name().to_str() {
                Some(file_name) => config_files.push(file_name.to_string()),
                None => return Err(ErrorRuntime::FSPathNotUTF8Error),
            };
        }
    }
    Ok(config_files)
}

#[cfg(test)]
mod test_utils {
    use super::*;

    const TEST_CLIENT_JSON_NOT_OK: &str = r#"{
        "name": "wrong name",
        "ip_address": "127.0.0.1",
        "port": 502,
        "protocol": "tcp",
        "registers": [
          {
            "name": "test_register_1",
            "objecttype": "holding",
            "address": 0,
            "length": 1,
            "datatype": "int16",
            "factor": 0,
            "value": 0
          },
          {
            "name": "test_register_2",
            "objecttype": "holding",
            "address": 1,
            "length": 1,
            "datatype": "int16",
            "factor": 0,
            "value": 0
          },
          {
            "name": "test_register_3",
            "objecttype": "input",
            "address": 0,
            "length": 1,
            "datatype": "int16",
            "factor": 0,
            "value": 0
          }
        ],
        "coils": [
          {
            "name": "test_coil_1",
            "objecttype": "coil",
            "address": 0,
            "value": false
          },
          {
            "name": "test_coil_2",
            "objecttype": "discrete",
            "address": 0,
            "value": false
          }
        ]
      }"#;

    #[test]
    fn test_get_local_config_files_full_path() {
        let config_files =
            get_local_config_files("/etc/modbus-prometheus-api-server/config".to_string(), true)
                .unwrap();
        assert_eq!(config_files.len(), 1);
        assert_eq!(
            config_files[0],
            "/etc/modbus-prometheus-api-server/config/test_client.json"
        );
    }

    #[test]
    fn test_get_local_config_files_file_name() {
        let config_files = get_local_config_files(
            "/etc/modbus-prometheus-api-server/config".to_string(),
            false,
        )
        .unwrap();
        assert_eq!(config_files.len(), 1);
        assert_eq!(config_files[0], "test_client.json");
    }

    #[test]
    fn test_check_if_client_exist() {
        let client_file_name = "test_client.json";
        let config = "/etc/modbus-prometheus-api-server/config";
        let result = check_if_client_exist(client_file_name, config);
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_check_client_strings(){
        let client_json = serde_json::from_str(TEST_CLIENT_JSON_NOT_OK).unwrap();
        let result = check_client_strings(&client_json);
        assert_eq!(result.is_ok(), false);
    }
}
