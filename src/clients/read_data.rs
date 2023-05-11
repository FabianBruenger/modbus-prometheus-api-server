use super::Clients;
use crate::prometheus::PrometheusMetrics;
use crate::utils;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_modbus::prelude::*;

// Side thread for gathering data of all registered modbus client. The data is then stored in the prometheus Variables
pub async fn read_data(registry: Arc<Mutex<PrometheusMetrics>>, clients: Arc<Mutex<Clients>>) {
    let mut read_data_interval =
        tokio::time::interval(Duration::from_millis(crate::DATA_COLLECTOR_TIME as u64));
    loop {
        read_data_interval.tick().await;
        for client in clients.lock().await.clients.values_mut() {
            log::debug!(
                "Reading from client: {} with IP address: {} on port: {}",
                &client.name,
                &client.ip_address,
                &client.port
            );
            let mut ctx = match utils::create_ctx(client).await {
                Ok(ctx) => ctx,
                Err(e) => {
                    log::error!(
                        "Could not connect to modbus client: {}. Skip reading from this modbus client. Error: {:?}",
                        &client.ip_address,
                        e
                    );
                    continue;
                }
            };
            // Read all registers from the client. Depending on the register objecttype
            for register in client.registers.iter_mut() {
                let mut data_to_write: Vec<u16> = Vec::new();
                // check if register is input or holding register
                match register.objecttype.as_str() {
                    "input" => {
                        log::debug!(
                            "Reading input register: {}_{}",
                            &client.name,
                            &register.name
                        );
                        data_to_write = match ctx
                            .read_input_registers(register.address, register.length)
                            .await
                        {
                            Ok(data) => data,
                            Err(_) => {
                                log::error!(
                                    "Could not read data from modbus client: {} on register: {}. Skip reading from this register",
                                    &client.ip_address,
                                    &register.address
                                );
                                continue;
                            }
                        };
                    }
                    "holding" => {
                        log::debug!(
                            "Reading holding register: {}_{}",
                            &client.name,
                            &register.name
                        );
                        data_to_write = match ctx
                            .read_holding_registers(register.address, register.length)
                            .await
                        {
                            Ok(data) => data,
                            Err(_) => {
                                log::error!(
                                    "Could not read data from modbus client: {} on register: {}. Skip reading from this register",
                                    &client.ip_address,
                                    &register.address
                                );
                                continue;
                            }
                        };
                    }
                    _ => {
                        log::error!(
                            "Invalid objecttype: {} for register: {}_{}. Skip reading from this register",
                            &register.objecttype,
                            &client.name,
                            &register.name
                        );
                        continue;
                    }
                };
                log::debug!(
                    "Data: {:?} from {}_{}",
                    data_to_write,
                    client.name,
                    register.name
                );
                register.value = data_to_write[0];
                // Final value for registry is calculated by the register itself
                let value_final = match register.calc_final_value_for_registry() {
                    Ok(value) => value,
                    Err(_) => {
                        log::error!(
                            "Could not calculate final value for register: {}_{}. Skip writing to prometheus registry",
                            client.name,
                            register.name
                        );
                        continue;
                    }
                };
                registry
                    .lock()
                    .await
                    .counters
                    .get_mut(&format!("{}_{}", client.name, register.name))
                    .unwrap()
                    .set(value_final);
            }
            // Read all coils from the client. Depending on the objecttype
            for coil in client.coils.iter_mut() {
                let mut data_to_write: Vec<bool> = Vec::new();
                // check if coil is coil or discrete
                match coil.objecttype.as_str() {
                    "coil" => {
                        log::debug!(
                            "Reading coil: {}_{}",
                            &client.name,
                            &coil.name
                        );
                        data_to_write = match ctx
                            .read_coils(coil.address, 1)
                            .await
                        {
                            Ok(data) => data,
                            Err(_) => {
                                log::error!(
                                            "Could not read data from modbus client: {} on coil: {}. Skip reading from this coil",
                                            &client.ip_address,
                                            &coil.address
                                        );
                                continue;
                            }
                        };
                    }
                    "discrete" => {
                        log::debug!(
                            "Reading discrete input: {}_{}",
                            &client.name,
                            &coil.name
                        );
                        data_to_write = match ctx
                            .read_discrete_inputs(coil.address, 1)
                            .await
                        {
                            Ok(data) => data,
                            Err(_) => {
                                log::error!(
                                            "Could not read data from modbus client: {} on discrete input: {}. Skip reading from this coil",
                                            &client.ip_address,
                                            &coil.address
                                        );
                                continue;
                            }
                        };
                    }
                    _ => {
                        log::error!(
                                    "Invalid objecttype: {} for coil: {}_{}. Skip reading from this coil",
                                    &coil.objecttype,
                                    &client.name,
                                    &coil.name
                                );
                        continue;
                    }
                };
                log::debug!(
                    "Data: {:?} from {}_{}",
                    data_to_write,
                    client.name,
                    coil.name
                );
                coil.value = data_to_write[0];

                registry
                    .lock()
                    .await
                    .counters
                    .get_mut(&format!("{}_{}", client.name, coil.name))
                    .unwrap()
                    .set(convert_bool_to_f64(data_to_write[0]));
            }
            ctx.disconnect().await.unwrap();
        }
    }
}

fn convert_bool_to_f64(input: bool) -> f64 {
    match input {
        true => 1.0,
        false => 0.0,
    }
}