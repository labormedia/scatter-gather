use ::redis::{Client, aio::MultiplexedConnection};
use scatter_gather_core::{
    connection::ConnectionHandlerOutEvent,
    middleware_specs::ServerConfig
};
use tokio::sync::mpsc;
mod redis;

#[derive(Debug)]
pub struct RedisMiddleware {
    pub config: ServerConfig,
    pub write: mpsc::Sender<ConnectionHandlerOutEvent<String>>,
    read: mpsc::Receiver<ConnectionHandlerOutEvent<String>>,
    client: MultiplexedConnection
}

impl RedisMiddleware {
    pub async fn new(config: ServerConfig) -> RedisMiddleware {
        Self::spin_up(config).await.expect("Couldn't build Middleware.")
    }

    pub async fn spin_up(config: ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let (write, read): (mpsc::Sender<ConnectionHandlerOutEvent<String>>, mpsc::Receiver<ConnectionHandlerOutEvent<String>>) = mpsc::channel(32);
        let client = Client::open(config.url.clone())?;  // "redis://172.17.0.2/"
        let con = client.get_multiplexed_tokio_connection().await?;
        Ok(
            Self {
                config,
                write,
                read,
                client: con
            }
        )
    }

    pub fn conn_clone(self) -> MultiplexedConnection {
        self.client.clone()
    }
}