use crate::clients::{self as Clients, Client};
use crate::errors::impls::ErrorRuntime as CustomErrors;
use crate::prometheus::PrometheusMetrics;
use crate::utils;
use prometheus::Encoder;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio_modbus::prelude::Writer;
use warp::{http::StatusCode, Rejection, Reply};

pub mod helpers;

// POST /clients - create new client
pub async fn create_client(
    registry: Arc<Mutex<PrometheusMetrics>>,
    clients: Arc<Mutex<Clients::Clients>>,
    client_input: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Check if the Configuration (Client) is not already present. Reject if it is. Then client can only be updated or deleted
    let client_name = client_input.name.clone();
    let client_config_json_name = format!("{}.json", &client_name);
    if let Err(e) = helpers::check_fs_config(&client_config_json_name) {
        return Err(warp::reject::custom(e));
    }
    // check if all string in the client have either lowercase, numbers or underscore
    if let Err(e) = helpers::check_client_strings(&serde_json::to_value(&client_input).unwrap()) {
        return Err(warp::reject::custom(e));
    }
    // Store the config to local FS
    if let Err(e) = helpers::write_config(&client_input) {
        return Err(warp::reject::custom(e));
    }
    // Add Counters for each register to the registry and register them
    if let Err(e) = registry.lock().await.register_client(&client_input) {
        return Err(warp::reject::custom(e));
    }
    // Add the config to the Clients struct
    clients.lock().await.add_client(client_name, client_input);

    Ok(warp::reply::reply())
}

// GET /clients - get all clients as string
pub async fn get_clients(
    clients: Arc<Mutex<Clients::Clients>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut clients_string = String::new();
    for (name, _) in clients.lock().await.clients.iter() {
        clients_string.push_str(&format!("{}\n", name));
    }
    Ok(warp::reply::html(clients_string))
}

// GET /clients/{name} - get client by name
pub async fn get_client(
    client: String,
    clients: Arc<Mutex<Clients::Clients>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let None = clients.lock().await.clients.get(&client) {
        return Err(warp::reject::custom(CustomErrors::ClientNotFound(Some(
            client.clone(),
        ))));
    };
    Ok(warp::reply::json(
        clients.lock().await.clients.get(&client).unwrap(),
    ))
}

// DELETE /clients/{name}  - delete one client by name
pub async fn delete_client(
    client: String,
    clients: Arc<Mutex<Clients::Clients>>,
    registry: Arc<Mutex<PrometheusMetrics>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Trying to delete client via DELETE /clients/{}.", &client);
    // Check if client exists
    if let None = clients.lock().await.clients.get(&client) {
        return Err(warp::reject::custom(CustomErrors::ClientNotFound(Some(
            client.clone(),
        ))));
    };
    // Unregister all client metrics from the registry
    if let Err(e) = registry
        .lock()
        .await
        .unregister_client(&clients.lock().await.clients.get(&client).unwrap().clone())
    {
        return Err(warp::reject::custom(e));
    }
    // Remove the client from the Clients struct
    clients.lock().await.delete_client(&client);
    // Remove the config from the local FS
    if let Err(e) = helpers::delete_config(&client) {
        return Err(warp::reject::custom(e));
    }
    Ok(warp::reply::reply())
}

// GET /metrics
pub async fn metrics_handler(
    registry: Arc<Mutex<PrometheusMetrics>>,
) -> Result<impl Reply, Rejection> {
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();

    // Gather the metrics.
    if let Err(_) = encoder.encode(&registry.lock().await.registry.gather(), &mut buffer) {
        return Err(warp::reject::custom(
            crate::errors::impls::ErrorRuntime::PrometheusErrorRegistry,
        ));
    };

    let res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(_) => {
            return Err(warp::reject::custom(
                crate::errors::impls::ErrorRuntime::PrometheusErrorEncoder,
            ))
        }
    };

    buffer.clear();

    Ok(res)
}

// PUT /clients/{name}/set-register?{register name }={value} - set a value for a key in a client
pub async fn write_register(
    client: String,
    params: HashMap<String, String>,
    clients: Arc<Mutex<Clients::Clients>>,
) -> Result<impl Reply, Rejection> {
    // Get parameter
    let param = params.iter().next().unwrap();
    // Check if the value can be parsed as u16
    let value = match param.1.parse::<u16>() {
        Ok(v) => v,
        Err(_) => {
            return Err(warp::reject::custom(CustomErrors::ValueNotParsableToU16(
                Some(param.1.clone()),
            )))
        }
    };
    // Check if client exist
    if let None = clients.lock().await.clients.get(&client) {
        return Err(warp::reject::custom(CustomErrors::ClientNotFound(Some(
            client.clone(),
        ))));
    };
    // Check if the register to write is a register in the client.registers
    if let None = clients
        .lock()
        .await
        .clients
        .get(&client)
        .unwrap()
        .get_register_by_name(param.0)
    {
        return Err(warp::reject::custom(CustomErrors::ClientRegisterNotFound(
            Some(param.0.clone()),
        )));
    };
    // Check if the register in writable = is input register
    if !clients
        .lock()
        .await
        .clients
        .get(&client)
        .unwrap()
        .is_register_input(param.0)
    {
        return Err(warp::reject::custom(CustomErrors::ClientRegisterNotInput(
            Some(param.0.clone()),
        )));
    };
    // Get the address of the register
    let address = clients
        .lock()
        .await
        .clients
        .get(&client)
        .unwrap()
        .get_register_by_name(param.0)
        .unwrap()
        .address;
    // Context for modbus client
    let mut ctx_tmp = match utils::create_ctx(clients.lock().await.clients.get(&client).unwrap())
        .await
    {
        Ok(ctx) => ctx,
        Err(_) => {
            return Err(warp::reject::custom(CustomErrors::ClientNotAbleToConnect(
                Some(clients.lock().await.clients.get(&client).unwrap().get_ip_address()),
            )));
        }
    };
    // Try to write register
    match ctx_tmp.write_single_register(address, value).await {
        Ok(_) => {
            log::info!("Successfully wrote to input register {}", param.0);
            ctx_tmp.disconnect().await.unwrap();
            return Ok(warp::reply::with_status("Wrote register!".to_string(), StatusCode::OK));
        }
        Err(_) => {
            ctx_tmp.disconnect().await.unwrap();
            return Err(warp::reject::custom(CustomErrors::ClientRegisterWriteGenericError));
        }
    }
}

// PUT /clients/{name}/set-coil?{coil name }={value} - set a value for a key in a client
pub async fn write_coil(
    client: String,
    params: HashMap<String, String>,
    clients: Arc<Mutex<Clients::Clients>>,
) -> Result<impl Reply, Rejection> {
    // Get parameter
    let param = params.iter().next().unwrap();
    // Check if the value can be parsed as u16
    let value = match param.1.parse::<bool>() {
        Ok(v) => v,
        Err(_) => {
            return Err(warp::reject::custom(CustomErrors::ValueNotParsableToBool(
                Some(param.1.clone()),
            )))
        }
    };
    // Check if client exist
    if let None = clients.lock().await.clients.get(&client) {
        return Err(warp::reject::custom(CustomErrors::ClientNotFound(Some(
            client.clone(),
        ))));
    };
    // Check if the register to write is a register in the client.registers
    if let None = clients
        .lock()
        .await
        .clients
        .get(&client)
        .unwrap()
        .get_coil_by_name(param.0)
    {
        return Err(warp::reject::custom(CustomErrors::ClientCoilNotFound(
            Some(param.0.clone()),
        )));
    };
    // Check if the coil in writable = is coil
    if !clients
        .lock()
        .await
        .clients
        .get(&client)
        .unwrap()
        .is_coil_input(param.0)
    {
        return Err(warp::reject::custom(CustomErrors::ClientCoilNotInput(
            Some(param.0.clone()),
        )));
    };
    // Get the address of the coil
    let address = clients
        .lock()
        .await
        .clients
        .get(&client)
        .unwrap()
        .get_coil_by_name(param.0)
        .unwrap()
        .address;
    // Context for modbus client
    let mut ctx_tmp = match utils::create_ctx(clients.lock().await.clients.get(&client).unwrap())
        .await
    {
        Ok(ctx) => ctx,
        Err(_) => {
            return Err(warp::reject::custom(CustomErrors::ClientNotAbleToConnect(
                Some(clients.lock().await.clients.get(&client).unwrap().get_ip_address()),
            )));
        }
    };
    // Try to write coil
    match ctx_tmp.write_single_coil(address, value).await {
        Ok(_) => {
            log::info!("Successfully wrote to coil {}", param.0);
            ctx_tmp.disconnect().await.unwrap();
            return Ok(warp::reply::with_status("Wrote coil!".to_string(), StatusCode::OK));
        }
        Err(_) => {
            ctx_tmp.disconnect().await.unwrap();
            return Err(warp::reject::custom(CustomErrors::ClientRegisterWriteGenericError));
        }
    }
}
