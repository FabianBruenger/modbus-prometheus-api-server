pub mod clients;
pub mod errors;
pub mod routes;
pub mod prometheus;
pub mod logging;
pub mod utils;

const CLIENT_CONFIG_PATH: &str = "/Users/fabianbrunger/Library/Mobile Documents/com~apple~CloudDocs/Programming/EMS/ems-backend/config";
const DATA_COLLECTOR_TIME: u16 = 1000; // in ms

#[allow(dead_code)]
#[allow(unused_variables)]
// Test JSON GET string
const JSON_STRING_OK: &str = r#"{
    "name": "test_client",
    "ip_address": "123.23.54.678",
    "registers": [
        {
            "name": "test_register_1",
            "address": 1,
            "datatype": "int16",
            "read": true,
            "write": false,
            "value": 0
        },
        {
            "name": "test_register_2",
            "address": 2,
            "datatype": "int16",
            "read": true,
            "write": false,
            "value": 0
        }
    ]
}"#;

#[allow(dead_code)]
#[allow(unused_variables)]
const JSON_STRING_NOT_OK: &str = r#"{
    "name: "Test Client",
    "ip_address": "123.23.54.678"
    "registers": [
        {
            "name": "Test Register",
            "address": 1,
            "datatype": "int16",
            "read": true,
            "write": false
        },
        {
            "name": "Test Register 2",
            "address": 2,
            "datatype": "int16",
            "read": true,
            "write": false
        }
    ]
}"#;
#[allow(dead_code)]
#[allow(unused_variables)]
const JSON_STRING_NOT_OK_STRING: &str = r#"{
    "name": "test_client 1",
    "ip_address": "123.23.54.678",
    "registers": [
        {
            "name": "test_register_1",
            "address": 1,
            "datatype": "int16",
            "read": true,
            "write": false,
            "value": 0
        },
        {
            "name": "test_register_2",
            "address": 2,
            "datatype": "int16",
            "read": true,
            "write": false,
            "value": 0
        }
    ]
}"#;