use crate::clients::{Client};
use crate::errors::impls::{ErrorRuntimeNoRejection};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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
