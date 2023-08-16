use std::{time::Duration, env::VarError};
use std::env;

use rumqttc::{MqttOptions, AsyncClient, EventLoop};
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub fn create_mqtt_client_and_eventloop() -> Result<(AsyncClient, EventLoop)> {
  let client_name = env::var("MQTT_CLIENT_NAME").expect("MQTT_CLIENT_NAME need to be set");
  let mqtt_broker_host = env::var("MQTT_BROKER_HOST").expect("MQTT_BROKER_HOST must be set");
  let mqtt_broker_port = env::var("MQTT_BROKER_PORT")
    .and_then(|port| port.parse::<u16>().map_err(|_| VarError::NotPresent))
    .expect("MQTT_BROKER_PORT must be set");
  let mqtt_keep_alive = env::var("MQTT_KEEP_ALIVE")
    .and_then(|keep| keep.parse::<u64>().map_err(|_| VarError::NotPresent))
    .unwrap_or(5);
  let mqtt_channel_capacity = env::var("MQTT_CHANNEL_CAPACITY")
    .and_then(|keep| keep.parse::<usize>().map_err(|_| VarError::NotPresent))
    .unwrap_or(10);


  let mut options = MqttOptions::new(client_name, mqtt_broker_host, mqtt_broker_port);
  options.set_keep_alive(Duration::from_secs(mqtt_keep_alive));

  Ok(AsyncClient::new(options, mqtt_channel_capacity))
}


#[derive(Debug, Deserialize, Serialize)]
pub struct MqttPrintMessage {
  pub host: String,
  pub port: u16,
  pub message: Vec<u8>
}

impl MqttPrintMessage {
  pub fn host_and_port(&self) -> String {
    let mut host_owned = self.host.clone();
    host_owned.push_str(":");
    host_owned.push_str(&self.port.to_string());
    host_owned
  }
}

impl TryFrom<&[u8]> for MqttPrintMessage {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let string_message = String::from_utf8(value.to_vec());
        match string_message {
          Ok(value) => serde_json::from_str(&value).map_err(|err| anyhow::Error::new(err)),
          Err(err) => Err(anyhow::Error::new(err))
        }
    }
}
