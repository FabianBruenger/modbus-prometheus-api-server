use modbus_prometheus_api_server::clients as Clients;
use modbus_prometheus_api_server::errors as Errors;
use modbus_prometheus_api_server::logging as CustomLog;
use modbus_prometheus_api_server::prometheus as Prometheus;
use modbus_prometheus_api_server::routes as Route;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::Method, Filter};

#[tokio::main]
async fn main() {
    env_logger::init();

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

    CustomLog::print_start();
    CustomLog::print_init_clients();

    let clients = Arc::new(Mutex::new(Clients::Clients::new()));
    let prometheus_registry = Arc::new(Mutex::new(Prometheus::PrometheusMetrics::new()));

    clients.lock().await.init().unwrap();
    CustomLog::print_init_prometheus_metrics();
    prometheus_registry
        .lock()
        .await
        .init(clients.clone())
        .await
        .unwrap();

    // Spawn a side thread for reading all modbus clients and set values in the local registers
    tokio::spawn(Clients::read_data::read_data(
        prometheus_registry.clone(),
        clients.clone(),
    ));

    // Filter for Prometheus Registry. That means add the registry to the filter chain so it can be used as funtion parameter
    let prometheus_registry_filter = warp::any().map(move || prometheus_registry.clone());
    let clients_filter = warp::any().map(move || clients.clone());

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

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
