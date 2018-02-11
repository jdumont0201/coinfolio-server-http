
use ws::{listen, connect, Handshake, Handler, Sender,Error, Result as wsResult, Message, CloseCode};

pub struct Server {
  pub   out: Sender,

}

impl Handler for Server {

    fn on_open(&mut self, _: Handshake) -> wsResult<()> {
        // We have a new connection, so we increment the connection counter
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> wsResult<()> {
        
        self.out.send(msg)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away   => println!("The client is leaving the site."),
            CloseCode::Abnormal => println!(
                "Closing handshake failed! Unable to obtain closing status from client."),
            _ => println!("The client encountered an error: {}", reason),
        }

        // The connection is going down, so we need to decrement the count

    }

    fn on_error(&mut self, err: Error) {
        println!("The server encountered an error: {:?}", err);
    }

}