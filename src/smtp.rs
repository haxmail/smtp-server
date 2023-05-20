use anyhow::{Context, Ok, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
        tracing::trace!("Received {raw_msg} in state {:?}", self.state);
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
}

impl Server {
    pub async fn new(domain: impl AsRef<str>, stream: tokio::net::TcpStream) -> Result<Server> {
        Ok(Server {
            stream,
            state_machine: StateMachine::new(domain),
        })
    }

    async fn greet(&mut self) -> Result<()> {
        self.stream
            .write_all(StateMachine::HI)
            .await
            .map_err(|e| e.into())
    }

    pub async fn serve(mut self) -> Result<()> {
        self.greet().await?;
        let mut buffer = vec![0; 65536];
        loop {
            let n = self.stream.read(&mut buffer).await?;

            if n == 0 {
                self.state_machine.handle_smtp("quit").ok();
                break;
            }

            let msg = std::str::from_utf8(&buffer[..n])?;
            let response = self.state_machine.handle_smtp(msg)?;
            if response != StateMachine::EMPTY {
                self.stream.write_all(response).await?;
            } else {
            }
            if response == StateMachine::BYE {
                break;
            }

            match self.state_machine.state {
                State::Received(ref mut mail) => {
                    println!("mail: {:?}", mail);
                }
                State::ReceivingData(ref mut mail) => {
                    println!("EOF before end {:?}", mail);
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_flow() {
        let mut sm = StateMachine::new("dummy");
        assert_eq!(sm.state, State::Fresh);
        sm.handle_smtp("HELO localhost").unwrap();
        assert_eq!(sm.state, State::Greeted);
        sm.handle_smtp("MAIL FROM: <local@example.com>").unwrap();
        assert!(matches!(sm.state, State::ReceivingRcpt(_)));
        sm.handle_smtp("RCPT TO: <a@localhost.com>").unwrap();
        assert!(matches!(sm.state, State::ReceivingRcpt(_)));
        sm.handle_smtp("RCPT TO: <b@localhost.com>").unwrap();
        assert!(matches!(sm.state, State::ReceivingRcpt(_)));
        sm.handle_smtp("DATA hello world\n").unwrap();
        assert!(matches!(sm.state, State::ReceivingData(_)));
        sm.handle_smtp("DATA hello world2\n").unwrap();
        assert!(matches!(sm.state, State::ReceivingData(_)));
        sm.handle_smtp("QUIT").unwrap();
        assert!(matches!(sm.state, State::Received(_)));
    }

    #[test]
    fn test_no_greeting() {
        let mut sm = StateMachine::new("dummy");
        assert_eq!(sm.state, State::Fresh);
        for command in [
            "MAIL FROM: <local@example.com>",
            "RCPT TO: <local@example.com>",
            "DATA hey",
            "GARBAGE",
        ] {
            assert!(sm.handle_smtp(command).is_err());
        }
    }
}
