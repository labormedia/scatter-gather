use scatter_gather_core;
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
        // let (write, read) = a.split();
        a
    }
}


// let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
// println!("WebSocket handshake has been successfully completed");