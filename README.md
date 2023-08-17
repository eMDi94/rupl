# Label Printer MQTT Connector

Use your favourite MQTT Broker and subscribe to it to receive messages to send to label printers.

The message format will be a JSON with the following structure
```
{
  "host": String,
  "port": u16,
  "message": [u8]
}
```

The message will be sent to the printer with a TCP call.

## Technologies
Rust programming language with actix actors to handle the calls to printers.
