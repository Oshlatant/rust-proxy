use config::{Config, File, Value};
use std::collections::HashMap;

pub fn init_config() -> Config {
    let config_file = File::with_name("./Config.toml");
    let mut config = Config::default();
    if let Err(e) = config.merge(config_file) {
        panic!("You need to make a Config.toml, error: {}", e);
    }

    config
}

pub fn server_config() -> HashMap<String, Value> {
    let config = init_config();
    let server = config
        .get_table("server")
        .unwrap_or_else(|e| panic!("Error: {}", e));

    server
}

pub fn get_caddr(config: &HashMap<String, Value>) -> String {
    let ip = config.get("ip").cloned().unwrap();
    let port = config.get("port").cloned().unwrap();

    format!("{}:{}", ip, port)
}

pub fn get_ip_whitelist(config: &HashMap<String, Value>) -> Vec<String> {
    let ip_whitelist = config.get("allowed_ip").cloned();

    let ip_whitelist = match ip_whitelist {
        Some(v) => v.into_array().unwrap(),
        None => panic!("Incorrect Config.toml"),
    };

    ip_whitelist
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
}
