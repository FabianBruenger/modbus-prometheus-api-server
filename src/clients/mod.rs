use crate::errors::impls::ErrorRuntime;
use crate::routes::helpers;
// use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

pub mod read_data;

#[derive(Debug, Serialize, Deserialize)]
pub struct Clients {
    pub clients: HashMap<String, Client>,
}
impl Clients {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
    // read local JSON configs and initialize all known clients
    pub fn init(&mut self) -> Result<(), ErrorRuntime> {
        // get all local config files
        if let Ok(config_files) = helpers::get_local_config_files_full_path() {
            if config_files.len() > 0 {
                for config_file in config_files {
                    let json_string = match fs::read_to_string(config_file) {
                        Ok(json_string) => json_string,
                        Err(_) => return Err(ErrorRuntime::FSReadToStringError),
                    };
                    let client = Client::new(json_string)?;
                    self.clients.insert(client.name.to_owned(), client);
                }
            }
        }
        Ok(())
    }
    pub fn add_client(&mut self, name: String, client: Client) {
        self.clients.insert(name, client);
    }
    pub fn delete_client(&mut self, name: &str) {
        self.clients.remove(name);
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Client {
    pub name: String,
    pub ip_address: String,
    pub port: u16,
    pub protocol: String,
    pub registers: Vec<Register>,
    pub coils: Vec<Coil>,
}
impl Client {
    pub fn new(json_string: String) -> Result<Self, ErrorRuntime> {
        let client = match serde_json::from_str::<Client>(&json_string) {
            Ok(client) => client,
            Err(_) => return Err(ErrorRuntime::ClientJsonParseError),
        };
        if let Err(e) = client.verify() {
            return Err(e);
        }
        log::info!("Client: {} successfully created", &client.name);
        log::debug!("Client {}:\n {:?} ", &client.name, &client);
        Ok(client)
    }
    /// Verify the client configuration
    ///
    /// # Arguments
    ///
    /// * `self` - The Client struct
    ///
    /// # Returns
    ///
    /// * `Result<(), ErrorRuntime>` - The result of the verification
    fn verify(&self) -> Result<(), ErrorRuntime> {
        let re = regex::Regex::new(r"^[a-z0-9_]+$").unwrap();
        if !re.is_match(&self.name) {
            return Err(ErrorRuntime::RegexError);
        }
        // Check if protocol is supported
        match self.protocol.as_str() {
            "tcp" => {}
            _ => {
                return Err(ErrorRuntime::ClientProtocolNotSupported);
            }
        }
        // Check if the names of the registers follow the naming convention
        for register in &self.registers {
            if !re.is_match(&register.name) {
                return Err(ErrorRuntime::RegexError);
            }
            match register.datatype.as_str() {
                "int16" => {}
                "uint16" => {}
                _ => {
                    return Err(ErrorRuntime::ClientRegisterDatatypeNotSupported);
                }
            }
            match register.objecttype.as_str() {
                "holding" => {}
                "input" => {}
                _ => {
                    return Err(ErrorRuntime::ClientRegisterObjecttypeNotSupported);
                }
            }
        }
        for coil in &self.coils {
            if !re.is_match(&coil.name) {
                return Err(ErrorRuntime::RegexError);
            }
            match coil.objecttype.as_str() {
                "coil" => {}
                "discrete" => {}
                _ => {
                    return Err(ErrorRuntime::ClientRegisterObjecttypeNotSupported);
                }
            }
        }
        Ok(())
    }
    pub fn get_register_by_name(&self, name: &str) -> Option<&Register> {
        for register in &self.registers {
            if register.name == name {
                return Some(register);
            }
        }
        None
    }
    // get coil by name 
    pub fn get_coil_by_name(&self, name: &str) -> Option<&Coil> {
        for coil in &self.coils {
            if coil.name == name {
                return Some(coil);
            }
        }
        None
    }
    // Check if the register is a input register
    pub fn is_register_input(&self, name: &str) -> bool {
        for register in &self.registers {
            if register.name == name {
                if register.objecttype == "input" {
                    return true;
                }
            }
        }
        false
    }
    // Check if the coil is a coil 
    pub fn is_coil_input(&self, name: &str) -> bool {
        for coil in &self.coils {
            if coil.name == name {
                if coil.objecttype == "coil" {
                    return true;
                }
            }
        }
        false
    }
    // Get ip address of client
    pub fn get_ip_address(&self) -> String {
        self.ip_address.to_owned()
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Register {
    pub name: String,
    pub objecttype: String,
    pub address: u16,
    length: u16,
    pub datatype: String,
    pub factor: i8,
    pub value: u16,
}
impl Register {
    /// Calculate the final value for the prometheus registry
    ///
    /// The value is calculated by the following formula:
    /// self.vale * 10 ^ self.factor
    ///
    /// # Arguments
    ///
    /// * `self` - The Register struct
    ///
    /// # Returns
    ///
    /// * `Result<f64, std::io::Error>` - The final value for the prometheus registry
    fn calc_final_value_for_registry(&self) -> Result<f64, std::io::Error> {
        match self.datatype.as_str() {
            "int16" => {
                let value_int16 = self.value as i16;
                let return_value = (value_int16 as f64) * (10_f64.powf(self.factor as f64));
                Ok(return_value)
            }
            "uint16" => {
                let value_uint16 = self.value as u16;
                let return_value = (value_uint16 as f64) * (10_f64.powf(self.factor as f64));
                Ok(return_value)
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unknown datatype",
                ));
            }
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Coil {
    pub name: String,
    pub objecttype: String,
    pub address: u16,
    pub value: bool,
}
// ----------------- TESTS -----------------
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_client_ok() {
        let client = Client::new(crate::JSON_STRING_OK.to_string());
        assert_eq!(client.is_ok(), true);
        println!("Client: {:?}", client);
    }
    #[test]
    fn test_client_not_ok() {
        let client = Client::new(crate::JSON_STRING_NOT_OK.to_string());
        assert_eq!(client.is_err(), true);
        println!("Client error: {:?}", client);
    }
    #[test]
    fn test_calc_final_value_for_registry_int16_minus128_factor0_ok() {
        let register = Register {
            name: "test".to_string(),
            objecttype: "holding".to_string(),
            address: 0,
            length: 1,
            datatype: "int16".to_string(),
            factor: 0,
            value: 65408,
        };
        let result = register.calc_final_value_for_registry();
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), -128.0);
    }
    #[test]
    fn test_calc_final_value_for_registry_int16_minus128_factor1_ok() {
        let register = Register {
            name: "test".to_string(),
            objecttype: "holding".to_string(),
            address: 0,
            length: 1,
            datatype: "int16".to_string(),
            factor: 1,
            value: 65408,
        };
        let result = register.calc_final_value_for_registry();
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), -1280.0);
    }
    #[test]
    fn test_calc_final_value_for_registry_int16_minus128_factorminus1_ok() {
        let register = Register {
            name: "test".to_string(),
            objecttype: "holding".to_string(),
            address: 0,
            length: 1,
            datatype: "int16".to_string(),
            factor: -1,
            value: 65408,
        };
        let result = register.calc_final_value_for_registry();
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), -12.80);
    }
    #[test]
    fn test_calc_final_value_for_registry_int16_minus128_factorminus126_ok() {
        let register = Register {
            name: "test".to_string(),
            objecttype: "holding".to_string(),
            address: 0,
            length: 1,
            datatype: "int16".to_string(),
            factor: -126,
            value: 65408,
        };
        let result = register.calc_final_value_for_registry();
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), -1.28e-124);
    }
    #[test]
    fn test_calc_final_value_for_registry_uint16_128_factor0_ok() {
        let register = Register {
            name: "test".to_string(),
            objecttype: "holding".to_string(),
            address: 0,
            length: 1,
            datatype: "uint16".to_string(),
            factor: 0,
            value: 128,
        };
        let result = register.calc_final_value_for_registry();
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), 128.0);
    }
    #[test]
    fn test_calc_final_value_for_registry_uint16_128_factor1_ok() {
        let register = Register {
            name: "test".to_string(),
            objecttype: "holding".to_string(),
            address: 0,
            length: 1,
            datatype: "uint16".to_string(),
            factor: 1,
            value: 128,
        };
        let result = register.calc_final_value_for_registry();
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), 1280.0);
    }
    #[test]
    fn test_calc_final_value_for_registry_uint16_128_factorminus1_ok() {
        let register = Register {
            name: "test".to_string(),
            objecttype: "holding".to_string(),
            address: 0,
            length: 1,
            datatype: "uint16".to_string(),
            factor: -1,
            value: 128,
        };
        let result = register.calc_final_value_for_registry();
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), 12.80);
    }
    #[test]
    fn test_calc_final_value_for_registry_uint16_128_factorminus126_ok() {
        let register = Register {
            name: "test".to_string(),
            objecttype: "holding".to_string(),
            address: 0,
            length: 1,
            datatype: "uint16".to_string(),
            factor: -126,
            value: 128,
        };
        let result = register.calc_final_value_for_registry();
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), 1.28e-124);
    }
    #[test]
    fn test_client_new_ok() {
        let test_json_config_ok = r#"{
            "name": "test_client",
            "ip_address": "127.0.0.1",
            "port": 8081,
            "protocol": "tcp",
            "registers": [
              {
                "name": "test_register_1",
                "objecttype": "holding",
                "address": 0,
                "length": 1,
                "datatype": "int16",
                "factor": 2,
                "value": 0
              },
              {
                "name": "test_register_2",
                "objecttype": "input",
                "address": 1,
                "length": 1,
                "datatype": "int16",
                "factor": -3,
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
                "address": 1,
                "value": false
              }
            ]
          }"#;
        let client = Client::new(test_json_config_ok.to_string());
        println!("{:?}", client);
        assert_eq!(client.is_ok(), true);
    }
    #[test]
    fn test_client_new_not_ok() {
        let test_json_config_not_ok = r#"{
            "name": "test_client",
            "ip_address": "127.0.0.1",
            "port": 8081,
            "protocol": "tcp",
            "registers": [
              {
                "name": "test_register_1",
                "objecttype": "holding",
                "address": 0,
                "length": 1,
                "datatype": "int16",
                "factor": 2,
                "value": 0
              },
              {
                "name": "test_register_2",
                "objecttype": "input",
                "address": 1,
                "length": 1,
                "datatype": "int16",
                "factor": -3,
                "value": 0
              }
            ],
            "coils_wrong": [
              {
                "name": "test_coil_1",
                "objecttype": "coil",
                "address": 0,
                "value": false
              },
              {
                "name": "test_coil_2",
                "objecttype": "discrete",
                "address": 1,
                "value": false
              }
            ]
          }"#;
        let client = Client::new(test_json_config_not_ok.to_string());
        println!("{:?}", client);
        assert_eq!(client.is_ok(), false);
    }
    #[test]
    fn test_client_verify_ok() {
        let test_json_config_ok = r#"{
            "name": "test_client",
            "ip_address": "127.0.0.1",
            "port": 8081,
            "protocol": "tcp",
            "registers": [
              {
                "name": "test_register_1",
                "address": 0,
                "length": 1,
                "datatype": "int16",
                "factor": 2,
                "read": true,
                "write": false,
                "value": 0
              },
              {
                "name": "test_register_2",
                "address": 1,
                "length": 1,
                "datatype": "int16",
                "factor": -3,
                "read": true,
                "write": false,
                "value": 0
              }
            ]
          }"#;

        let client = Client::new(test_json_config_ok.to_string());
        assert_eq!(client.unwrap().verify().is_ok(), true);
    }
    #[test]
    fn test_client_verify_not_ok_wrong_name() {
        let test_json_config_not_ok = r#"{
            "name": "test_client wrong",
            "ip_address": "127.0.0.1",
            "port": 8081,
            "protocol": "tcp",
            "registers": [
              {
                "name": "test_register_1",
                "address": 0,
                "length": 1,
                "datatype": "int16",
                "factor": 2,
                "read": true,
                "write": false,
                "value": 0
              },
              {
                "name": "test_register_2",
                "address": 1,
                "length": 1,
                "datatype": "int16",
                "factor": -3,
                "read": true,
                "write": false,
                "value": 0
              }
            ]
          }"#;

        let client = Client::new(test_json_config_not_ok.to_string());
        assert_eq!(client.unwrap().verify().is_err(), true);
    }
    #[test]
    fn test_client_verify_not_ok_wrong_protocol() {
        let test_json_config_not_ok = r#"{
            "name": "test_client",
            "ip_address": "127.0.0.1",
            "port": 8081,
            "protocol": "other than tcp",
            "registers": [
              {
                "name": "test_register_1",
                "address": 0,
                "length": 1,
                "datatype": "int16",
                "factor": 2,
                "read": true,
                "write": false,
                "value": 0
              },
              {
                "name": "test_register_2",
                "address": 1,
                "length": 1,
                "datatype": "int16",
                "factor": -3,
                "read": true,
                "write": false,
                "value": 0
              }
            ]
          }"#;

        let client = Client::new(test_json_config_not_ok.to_string());
        assert_eq!(client.unwrap().verify().is_err(), true);
    }
    #[test]
    fn test_client_verify_not_ok_wrong_register_name() {
        let test_json_config_not_ok = r#"{
            "name": "test_client",
            "ip_address": "127.0.0.1",
            "port": 8081,
            "protocol": "tcp",
            "registers": [
              {
                "name": "test_register 1",
                "address": 0,
                "length": 1,
                "datatype": "int16",
                "factor": 2,
                "read": true,
                "write": false,
                "value": 0
              },
              {
                "name": "test_register_2",
                "address": 1,
                "length": 1,
                "datatype": "int16",
                "factor": -3,
                "read": true,
                "write": false,
                "value": 0
              }
            ]
          }"#;

        let client = Client::new(test_json_config_not_ok.to_string());
        assert_eq!(client.unwrap().verify().is_err(), true);
    }
    #[test]
    fn test_client_verify_not_ok_wrong_register_datatype() {
        let test_json_config_not_ok = r#"{
            "name": "test_client",
            "ip_address": "127.0.0.1",
            "port": 8081,
            "protocol": "tcp",
            "registers": [
              {
                "name": "test_register_1",
                "address": 0,
                "length": 1,
                "datatype": "wrong",
                "factor": 2,
                "read": true,
                "write": false,
                "value": 0
              },
              {
                "name": "test_register_2",
                "address": 1,
                "length": 1,
                "datatype": "int16",
                "factor": -3,
                "read": true,
                "write": false,
                "value": 0
              }
            ]
          }"#;

        let client = Client::new(test_json_config_not_ok.to_string());
        assert_eq!(client.unwrap().verify().is_err(), true);
    }
}
