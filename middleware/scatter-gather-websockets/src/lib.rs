use scatter_gather_core as sgc;
use scatter_gather_core::middleware_specs::{
    ServerConfig,
    Interceptor
};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use futures::{
    Future
};
use tokio::net::TcpStream;
use std::{
    task::Poll,
    fmt,
    error::Error
};

pub struct WebSocketsMiddleware<TInterceptor: Interceptor> {
    pub config: ServerConfig<TInterceptor>
}

impl<TInterceptor: Interceptor> WebSocketsMiddleware<TInterceptor> {
    pub fn new(config: ServerConfig<TInterceptor>) -> Self {
        Self {
            config: config
        }
    }
    pub async fn connect(&self) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let url = url::Url::parse(&self.config.url).expect("Expected Websocket Url");
        let (a,_b) = connect_async(url).await.expect("Connection to Websocket server failed");
        a
    }
}

#[derive(Debug)]
pub enum ConnectionHandlerError {
    Custom
}

impl fmt::Display for ConnectionHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Custom error")
    }
}
impl Error for ConnectionHandlerError {}

impl<TInterceptor: Interceptor> sgc::ConnectionHandler for WebSocketsMiddleware<TInterceptor> {
    type InEvent = ();
    type OutEvent = ();
    type Error = ConnectionHandlerError;

    fn inject_event(&mut self, event: Self::InEvent) {
        
    }

    fn poll(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<
            sgc::ConnectionHandlerEvent<Self::OutEvent, Self::Error>
        > {
        Poll::Ready(sgc::ConnectionHandlerEvent::Close(ConnectionHandlerError::Custom))
    }
}

// let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
// println!("WebSocket handshake has been successfully completed");