use crate::clients::{Client};
use crate::errors::impls::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::fs;
use tokio_modbus::prelude::*;

pub async fn create_ctx(client: &Client) -> Result<tokio_modbus::client::Context, ErrorRuntimeNoRejection>{
    log::debug!("Creating ipaddress: {}", &client.ip_address);
    let ipaddress = match client.ip_address.parse::<Ipv4Addr>() {
        Ok(ipv4) => {
            log::debug!("Created ipaddress: {}", &ipv4);
            ipv4
        },
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


/// Based on the main config path, get all the config files in that path
/// The config files should be JSON files with the client name as filename 
/// and should be stored in /etc/modbus-prometheus-api-server/config for linux and Mac OS
/// 
/// # Arguments
/// 
/// * `config` - The main config path. Under this path all and ONLY client JSON configs should be stored
/// 
/// # Returns
/// 
/// * `config_files` - A vector of strings with the full path to the config files
/// 
/// # Errors
/// 
/// * `ErrorRuntime::FSReadDirError` - Could not read the main config path, Maybe user issues on OS level
/// * `ErrorRuntime::FSDirEntryError` - Could not read the config file
pub fn get_local_config_files_full_path(config_path: String) -> Result<Vec<String>, ErrorRuntime> {
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
        match dir_entry.path().to_str(){
            Some(full_path) => config_files.push(full_path.to_string()),
            None => return Err(ErrorRuntime::FSPathNotUTF8Error),
        };
    }
    Ok(config_files)
}

#[cfg(test)]
mod test_utils{
    use super::*;

    #[test]
    fn test_get_local_config_files_full_path() {
        let config_files = get_local_config_files_full_path("/etc/modbus-prometheus-api-server/config".to_string()).unwrap();
        assert_eq!(config_files.len(), 1);
        assert_eq!(config_files[0], "/etc/modbus-prometheus-api-server/config/test_client.json");
    }
}
