use std::time::Duration;

use async_trait::async_trait;
use ctor::ctor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::session::{Error, Loot};
use crate::utils;
use crate::Options;
use crate::Plugin;

use crate::creds::Credentials;

const CONNECTED_RESPONSE: &[u8] = &[67, 79, 78, 78, 69, 67, 84, 69, 68];

#[ctor]
fn register() {
    crate::plugins::manager::register("stomp", Box::new(STOMP::new()));
}

#[derive(Clone)]
pub(crate) struct STOMP {
    address: String,
}

impl STOMP {
    pub fn new() -> Self {
        STOMP {
            address: String::new(),
        }
    }
}

#[async_trait]
impl Plugin for STOMP {
    fn description(&self) -> &'static str {
        "STOMP password authentication (ActiveMQ, RabbitMQ, HornetQ and OpenMQ)."
    }

    fn setup(&mut self, opts: &Options) -> Result<(), Error> {
        let (host, port) = utils::parse_target(opts.target.as_ref(), 61613)?;
        self.address = format!("{}:{}", host, port);
        Ok(())
    }

    async fn attempt(&self, creds: &Credentials, timeout: Duration) -> Result<Option<Loot>, Error> {
        let mut stream = crate::utils::net::async_tcp_stream(&self.address, timeout, false).await?;

        stream
            .write_all(
                format!(
                    "CONNECT\nlogin:{}\npasscode:{}\n\n\x00\n",
                    &creds.username, &creds.password
                )
                .as_bytes(),
            )
            .await
            .map_err(|e| e.to_string())?;

        let mut buffer = [0_u8; 1024];

        stream.read(&mut buffer).await.map_err(|e| e.to_string())?;

        if buffer.starts_with(CONNECTED_RESPONSE) {
            Ok(Some(Loot::from([
                ("username".to_owned(), creds.username.to_owned()),
                ("password".to_owned(), creds.password.to_owned()),
            ])))
        } else {
            Ok(None)
        }
    }
}
