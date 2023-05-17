use prometheus::{Registry};
use std::{collections::HashMap, sync::{Arc}};
use tokio::sync::Mutex;
use crate::clients::{Clients, Client};
use crate::errors::impls::ErrorRuntime;

// The struct hold the registry and the metrics. Metrics are stored in a vector and can be added or reduced.
#[derive(Clone, Debug)]
pub struct PrometheusMetrics {
    pub registry: Registry,
    pub counters: HashMap<String, prometheus::Gauge>,
}
impl PrometheusMetrics {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            counters: HashMap::new(),
        }
    }

    pub async fn init(&mut self, clients: Arc<Mutex<Clients>>) -> Result<(), ErrorRuntime> {
        // check if clients are initialized
        let clients = clients.lock().await;
        if clients.clients.is_empty() {
            log::warn!("No clients initialized. Please initialize clients first.");
            return Ok(());
        } else {
            for (client_name, client) in clients.clients.iter() {
                for register in client.registers.iter() {
                    let tmp_name = format!("{}_{}", client_name, register.name);
                    let tmp_help = format!("{} {}", register.datatype, register.objecttype);
                    let tmp_gauge = match prometheus::Gauge::new(&tmp_name, &tmp_help){
                        Ok(gauge) => gauge,
                        Err(_) => return Err(ErrorRuntime::PrometheusErrorGaugeNew),
                    };
                    if let Err(_) = self.registry.register(Box::new(tmp_gauge.clone())){
                        return Err(ErrorRuntime::PrometheusErrorRegistryRegister);
                    }
                    self.counters.insert(tmp_name, tmp_gauge);
                }
                for coil in client.coils.iter() {
                    let tmp_name = format!("{}_{}", client.name, coil.name);
                    let tmp_help = format!("{}", coil.objecttype);
                    let tmp_gauge = match prometheus::Gauge::new(&tmp_name, &tmp_help){
                        Ok(gauge) => gauge,
                        Err(_) => return Err(ErrorRuntime::PrometheusErrorGaugeNew),
                    };
                    if let Err(_) = self.registry.register(Box::new(tmp_gauge.clone())){
                        return Err(ErrorRuntime::PrometheusErrorRegistryRegister);
                    }
                    self.counters.insert(tmp_name, tmp_gauge);
                }
            }
        }
        Ok(())
    }

    pub fn register_client(&mut self, client: &Client) -> Result<(), ErrorRuntime>{
        // add all registers to the registry
        for register in client.registers.iter() {
            let tmp_name = format!("{}_{}", client.name, register.name);
            let tmp_help = format!("{} {}", register.datatype, register.objecttype);
            let tmp_gauge = match prometheus::Gauge::new(&tmp_name, &tmp_help){
                Ok(gauge) => gauge,
                Err(_) => return Err(ErrorRuntime::PrometheusErrorGaugeNew),
            };
            if let Err(_) = self.registry.register(Box::new(tmp_gauge.clone())){
                return Err(ErrorRuntime::PrometheusErrorRegistryRegister);
            }
            self.counters.insert(tmp_name, tmp_gauge);
        }
        // register all coils to the registry
        for coil in client.coils.iter() {
            let tmp_name = format!("{}_{}", client.name, coil.name);
            let tmp_help = format!("{}", coil.objecttype);
            let tmp_gauge = match prometheus::Gauge::new(&tmp_name, &tmp_help){
                Ok(gauge) => gauge,
                Err(_) => return Err(ErrorRuntime::PrometheusErrorGaugeNew),
            };
            if let Err(_) = self.registry.register(Box::new(tmp_gauge.clone())){
                return Err(ErrorRuntime::PrometheusErrorRegistryRegister);
            }
            self.counters.insert(tmp_name, tmp_gauge);
        }

        log::debug!("New registry: {:?}", self.registry);

        Ok(())
    }

    pub fn unregister_client(&mut self, client: &Client) -> Result<(), ErrorRuntime>{
        // Delete all registers from the registry
        for register in client.registers.iter() {
            let tmp_name = format!("{}_{}", client.name, register.name);
            let tmp_gauge = match self.counters.remove(&tmp_name){
                Some(gauge) => gauge,
                None => return Err(ErrorRuntime::PrometheusErrorGaugeRemove),
            };
            if let Err(_) = self.registry.unregister(Box::new(tmp_gauge.clone())){
                return Err(ErrorRuntime::PrometheusErrorRegistryUnregister);
            }
        }
        Ok(())
    }

    pub fn update_gauge(&mut self, name: &str, value: f64) {
        // write value to hash map entry
        if let Some(metric) = self.counters.get_mut(name) {
            metric.set(value);
        }
    }
}
