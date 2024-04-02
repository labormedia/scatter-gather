use ::redis::{Client, aio::MultiplexedConnection};
use scatter_gather_core::{
    connection::ConnectionHandlerOutEvent,
    middleware_interface::NodeConfig
};
use tokio::sync::mpsc;
mod redis;

// Define possible errors.
#[derive(Debug)]
pub enum ConnectionHandlerError {
    Custom
}

impl Error for ConnectionHandlerError {}

// Define a way to debug the errors.
impl fmt::Display for ConnectionHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Custom error")
    }
}

#[derive(Debug)]
pub struct RedisMiddleware {
    pub config: NodeConfig,
    pub write: mpsc::Sender<ConnectionHandlerOutEvent<String>>,
    read: mpsc::Receiver<ConnectionHandlerOutEvent<String>>,
    client: MultiplexedConnection
}

impl RedisMiddleware {
    pub async fn new(config: NodeConfig) -> RedisMiddleware {
        Self::spin_up(config).await.expect("Couldn't build Middleware.")
    }

    pub async fn try_new(config: NodeConfig) -> Result<RedisMiddleware, Box<dyn Error>> {
        Ok(Self::spin_up(config).await?)
    }

    pub async fn spin_up(config: NodeConfig) -> Result<Self, Box<dyn std::error::Error>> {
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