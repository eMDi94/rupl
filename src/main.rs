mod printer;
mod mqtt;

use std::env;
use std::collections::HashMap;

use actix::prelude::*;
use printer::*;
use mqtt::*;
use dotenv::dotenv;
use rumqttc::{QoS, Incoming, Event};
use log::{info, error};

type PrinterMap = HashMap<String, Addr<PrinterActor>>;

#[actix::main]
async fn main() {
  dotenv().ok();
  log4rs::init_file("config/log4rs.yml", Default::default()).expect("Cannot initialize logger");

  info!("Starting the application...");

  let mut printer_map = PrinterMap::new();
  let printers = load_printers();
  for printer in printers {
    let host_and_port = printer.host_and_port();
    let addr = printer.start();
    printer_map.insert(host_and_port, addr);
  }

  let topic = env::var("MQTT_TOPIC").expect("MQTT_TOPIC must be set");
  let (client, mut eventloop) = create_mqtt_client_and_eventloop().expect("Cannot start mqtt client");
  let _ = client.subscribe(&topic, QoS::ExactlyOnce).await;

  info!("Subscribed to {:?}. Start listening...", &topic);

  while let Ok(event) = eventloop.poll().await {
    if let Event::Incoming(Incoming::Publish(packet)) = event {
      let mqtt_print_message = MqttPrintMessage::try_from(packet.payload.as_ref());
      match mqtt_print_message {
        Ok(message) => {
          info!("Received mqtt print message");
          select_printer_and_send_message(&printer_map, message).await
        },
        Err(err) => error!("Received an error. {:?}", err)
      }
    }
  }

  info!("Exiting...");
}

fn load_printers() -> Vec<PrinterActor> {
  let printers_str = env::var("PRINTERS").expect("PRINTERS must be set");
  let printers = printers_str.split(",").map(|printer| printer.trim());

  printers.map(|printer| {
    let printer = String::from(printer);
    let mut printer_host_port = printer.split(":");
    let host = printer_host_port.nth(0).unwrap().to_owned();
    let port = match printer_host_port.nth(1) {
      Some(port_string) => port_string.parse::<u16>().expect("Port must be parsable in an u16"),
      None => 9001
    };

    PrinterActor::new(host.as_str(), port)
  }).collect()
}

async fn select_printer_and_send_message(printer_map: &PrinterMap, message: MqttPrintMessage) {
  let host_and_port = message.host_and_port();
  let printer_addr = printer_map.get(&host_and_port);
  match printer_addr {
    Some(printer_actor) => {
      let print_message = PrintMessage::Print { zpl: message.message };
      let _ = printer_actor.send(print_message).await;
    },
    None => {
      error!("No printer actor found for {:?}", host_and_port);
    }
  }
}
