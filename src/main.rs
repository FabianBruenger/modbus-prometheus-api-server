use modbus_prometheus_api_server::clients as Clients;
use modbus_prometheus_api_server::configuration as Configuration;
use modbus_prometheus_api_server::errors as Errors;
use modbus_prometheus_api_server::logging as CustomLog;
use modbus_prometheus_api_server::prometheus as Prometheus;
use modbus_prometheus_api_server::routes as Route;

use env_logger::Env;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::Method, Filter};

#[tokio::main]
async fn main() {
    // Global configuration
    let config = Configuration::Args::new();
    // Set up logging
    env_logger::Builder::from_env(Env::default().default_filter_or(config.get_log_level())).init();
    let log_filter = warp::log::custom(|info| {
        log::info!(
            "{} {} {} {:?} from {} with {:?}",
            info.method(),
            info.path(),
            info.status(),
            info.elapsed(),
            info.remote_addr().unwrap(),
            info.request_headers()
        );
    });
    // Strat logging
    CustomLog::print_start(config.get_config_path().to_string());
    // Gloabl clients and prometheus registry
    let clients = Arc::new(Mutex::new(Clients::Clients::new(config.get_config_path())));
    let prometheus_registry = Arc::new(Mutex::new(Prometheus::PrometheusMetrics::new()));
    // Initializing clients and prometheus registry
    if let Err(e) = clients.lock().await.init() {
        log::error!("Error initializing clients: {:?}", e);
    }
    if let Err(e) = prometheus_registry.lock().await.init(clients.clone()).await {
        log::error!("Error initializing prometheus registry: {:?}", e);
    }
    // Spawn a side thread for reading all modbus clients and set values in the local registers
    tokio::spawn(Clients::read_data::read_data(
        prometheus_registry.clone(),
        clients.clone(),
        config.get_read_data_interval_ms() as u64,
    ));
    // Filter for Prometheus Registry. That means add the registry to the filter chain so it can be used as funtion parameter
    let prometheus_registry_filter = warp::any().map(move || prometheus_registry.clone());
    let clients_filter = warp::any().map(move || clients.clone());
    // Service got started
    log::info!("Idle state...");
    /*
    Handle routes:
    - POST /clients
    - GET /clients
    - DELETE /clients
    - GET /metrics
    */
    let metrics_route = warp::get()
        .and(warp::path("metrics"))
        .and(warp::path::end())
        .and(prometheus_registry_filter.clone())
        .and_then(Route::metrics_handler);

    let create_client = warp::post()
        .and(warp::path("clients"))
        .and(warp::path::end())
        .and(prometheus_registry_filter.clone())
        .and(clients_filter.clone())
        .and(warp::body::json())
        .and_then(Route::create_client);

    let get_clients = warp::get()
        .and(warp::path("clients"))
        .and(warp::path::end())
        .and(clients_filter.clone())
        .and_then(Route::get_clients);

    let get_client = warp::get()
        .and(warp::path("clients"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(clients_filter.clone())
        .and_then(Route::get_client);

    let delete_client = warp::delete()
        .and(warp::path("clients"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(clients_filter.clone())
        .and(prometheus_registry_filter.clone())
        .and_then(Route::delete_client);

    let set_reg = warp::put()
        .and(warp::path("clients"))
        .and(warp::path::param::<String>())
        .and(warp::path("set-register"))
        .and(warp::path::end())
        .and(warp::query())
        .and(clients_filter.clone())
        .and_then(Route::write_register);

    let set_coil = warp::put()
        .and(warp::path("clients"))
        .and(warp::path::param::<String>())
        .and(warp::path("set-coil"))
        .and(warp::path::end())
        .and(warp::query())
        .and(clients_filter.clone())
        .and_then(Route::write_coil);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("not-in-the-request")
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let routes = get_clients
        .or(create_client)
        .or(metrics_route)
        .or(get_client)
        .or(delete_client)
        .or(set_reg)
        .or(set_coil)
        .with(cors)
        .with(log_filter)
        .recover(Errors::return_error);

    warp::serve(routes)
        .run(([127, 0, 0, 1], config.get_port()))
        .await;
}
