use std::sync::{Arc, Mutex};

use anyhow::{Context, Ok, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::database;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mail {
    pub from: String,
    pub to: Vec<String>,
    pub data: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum State {
    Fresh,
    Greeted,
    ReceivingRcpt(Mail),
    ReceivingData(Mail),
    Received(Mail),
}

struct StateMachine {
    state: State,
    ehlo_greeting: String,
}

impl StateMachine {
    const HI: &[u8] = b"220 haxmail\n";
    const OK: &[u8] = b"250 OK\n";
    const AUTH_OK: &[u8] = b"235 OK\n";
    const SEND_DATA: &[u8] = b"354 END DATA WITH <CR><LF>.<CR><LF>\n";
    const BYE: &[u8] = b"221 BYE\n";
    const EMPTY: &[u8] = &[];

    pub fn new(domain: impl AsRef<str>) -> StateMachine {
        let domain = domain.as_ref();
        let ehlo_greeting = format!("250-{domain} Hello {domain}\n250 AUTH PLAIN LOGIN\n");

        StateMachine {
            state: State::Fresh,
            ehlo_greeting,
        }
    }

    pub fn handle_smtp(&mut self, raw_msg: &str) -> Result<&[u8]> {
        println!("Yomama {raw_msg} in state {:?}", self.state);
        let mut msg = raw_msg.split_whitespace();
        let command = msg.next().context("received empty command")?.to_lowercase();
        let state = std::mem::replace(&mut self.state, State::Fresh);
        match (command.as_str(), state) {
            ("ehlo", State::Fresh) => {
                tracing::trace!("Sending AUTH info");
                self.state = State::Greeted;
                Ok(self.ehlo_greeting.as_bytes())
            }
            ("helo", State::Fresh) => {
                self.state = State::Greeted;
                Ok(StateMachine::OK)
            }
            ("noop", _) | ("help", _) | ("info", _) | ("vrfy", _) | ("expn", _) => {
                tracing::trace!("Got {command}");
                Ok(StateMachine::OK)
            }
            ("rset", _) => {
                self.state = State::Fresh;
                Ok(StateMachine::OK)
            }
            ("auth", _) => {
                tracing::trace!("Acknowledging AUTH");
                Ok(StateMachine::AUTH_OK)
            }
            ("mail", State::Greeted) => {
                tracing::trace!("Receiving MAIL");
                let from = msg.next().context("received empty MAIL")?;
                let from = from
                    .strip_prefix("FROM:")
                    .context("received incorrect MAIL")?;
                tracing::debug!("FROM: {from}");
                self.state = State::ReceivingRcpt(Mail {
                    from: from.to_string(),
                    ..Default::default()
                });
                Ok(StateMachine::OK)
            }
            ("rcpt", State::ReceivingRcpt(mut mail)) => {
                tracing::trace!("Receiving rcpt");
                let to = msg.next().context("received empty RCPT")?;
                let to = to.strip_prefix("TO:").context("received incorrect RCPT")?;
                tracing::debug!("TO: {to}");
                mail.to.push(to.to_string());
                self.state = State::ReceivingRcpt(mail);
                Ok(StateMachine::OK)
            }
            ("data", State::ReceivingRcpt(mail)) => {
                tracing::trace!("Receiving data");
                self.state = State::ReceivingData(mail);
                Ok(StateMachine::SEND_DATA)
            }
            ("quit", State::ReceivingData(mail)) => {
                tracing::trace!(
                    "Received data: FROM: {} TO:{} DATA:{}",
                    mail.from,
                    mail.to.join(", "),
                    mail.data
                );
                self.state = State::Received(mail);
                Ok(StateMachine::BYE)
            }
            ("quit", _) => {
                tracing::warn!("Received quit before getting any data");
                Ok(StateMachine::BYE)
            }
            (_, State::ReceivingData(mut mail)) => {
                tracing::trace!("Receiving data");
                let resp = if raw_msg.ends_with("\r\n.\r\n") {
                    StateMachine::OK
                } else {
                    StateMachine::EMPTY
                };
                mail.data += raw_msg;
                self.state = State::ReceivingData(mail);
                Ok(resp)
            }
            _ => anyhow::bail!(
                "Unexpected message received in state {:?}: {raw_msg}",
                self.state
            ),
        }
    }
}

pub struct Server {
    stream: tokio::net::TcpStream,
    state_machine: StateMachine,
    db: Arc<Mutex<database::Client>>,
}

impl Server {
    pub async fn new(domain: impl AsRef<str>, stream: tokio::net::TcpStream) -> Result<Server> {
        Ok(Server {
            stream,
            state_machine: StateMachine::new(domain),
            db: Arc::new(Mutex::new(database::Client::new().await?)),
        })
    }

    async fn greet(&mut self) -> Result<()> {
        self.stream
            .write_all(StateMachine::HI)
            .await
            .map_err(|e| e.into())
    }

    pub async fn serve(mut self) -> Result<()> {
        println!("Serving");
        self.greet().await?;

        let mut buf = vec![0; 65536];
        loop {
            let n = self.stream.read(&mut buf).await?;

            if n == 0 {
                tracing::info!("Received EOF");
                self.state_machine.handle_smtp("quit").ok();
                break;
            }
            let msg = std::str::from_utf8(&buf[0..n])?;
            let response = self.state_machine.handle_smtp(msg)?;
            if response != StateMachine::EMPTY {
                self.stream.write_all(response).await?;
            } else {
                tracing::debug!("Not responding, awaiting more data");
            }
            if response == StateMachine::BYE {
                break;
            }
        }
        match self.state_machine.state {
            State::Received(mail) => {
                println!("Received mail: {:?}", mail);
                tracing::info!("Received mail: {:?}", mail);
                self.db.lock().unwrap().replicate(mail).await?;
            }
            State::ReceivingData(mail) => {
                tracing::info!("Received EOF before receiving QUIT");
                self.db.lock().unwrap().replicate(mail).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_flow() {
        let mut sm = StateMachine::new("localhost");
        assert_eq!(sm.state, State::Fresh);
        sm.handle_smtp("HELO localhost").unwrap();
        assert_eq!(sm.state, State::Greeted);
        sm.handle_smtp("MAIL FROM: <tiger@gmail.com>").unwrap();

        assert!(matches!(sm.state, State::ReceivingRcpt(_)));
        sm.handle_smtp("RCPT TO: <a@localhost>").unwrap();
        assert!(matches!(sm.state, State::ReceivingRcpt(_)));
        sm.handle_smtp("RCPT TO: <b@localhost>").unwrap();
        assert!(matches!(sm.state, State::ReceivingRcpt(_)));
        sm.handle_smtp("DATA hello world\n").unwrap();
        assert!(matches!(sm.state, State::ReceivingData(_)));
        sm.handle_smtp("DATA hello world2\n").unwrap();
        assert!(matches!(sm.state, State::ReceivingData(_)));
        sm.handle_smtp("QUIT").unwrap();
        println!("{:?}", sm.state);
        assert!(matches!(sm.state, State::Received(_)));
    }
}
