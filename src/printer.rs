use std::{net::TcpStream, io::Write};

use actix::prelude::*;
use anyhow::Result;
use log::{info, error};

#[derive(Message, Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub enum PrintMessage {
  Print {
    zpl: Vec<u8>   
  }
}

#[derive(Debug)]
pub struct PrinterActor {
  pub host: String,
  pub port: u16
}

impl PrinterActor {
  pub fn new(host: &str, port: u16) -> Self {
    PrinterActor { host: host.into(), port }
  }

  pub fn host_and_port(&self) -> String {
    let mut host_owned = self.host.clone();
    host_owned.push_str(":");
    host_owned.push_str(&self.port.to_string());
    host_owned
  }
}

impl Actor for PrinterActor {
  type Context = Context<Self>;
}

impl Handler<PrintMessage> for PrinterActor {
  type Result = Result<()>;

  fn handle(&mut self, msg: PrintMessage, _ctx: &mut Self::Context) -> Self::Result {
    match msg {
      PrintMessage::Print { zpl } => {
        match TcpStream::connect(self.host_and_port()) {
          Ok(mut stream) => {
            if let Err(err) = stream.write_all(&zpl) {
              error!("Error while writing content on the socket. {:?}", err);
              Err(anyhow::Error::msg(err.to_string()))
            } else {
              info!("Label sent to printer {:?}", self.host_and_port());
              Ok(())
            }
          },
          Err(err) => {
            error!("Error while connecting to tcp socket. {:?}", err);
            Err(anyhow::Error::msg(err.to_string()))
          }
        }
      }
    }
  }
}