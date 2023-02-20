use std::env;
use futures_util::{StreamExt};
use tokio::io::{AsyncReadExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use redis::{
    aio::MultiplexedConnection,
};
