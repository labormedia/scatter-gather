use futures::stream::{SplitSink, SplitStream};
use scatter_gather_core as sgc;
use scatter_gather_core::middleware_specs::{
    ServerConfig,
    Interceptor
};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use futures::{
    Future, StreamExt, SinkExt
};
use tokio::net::TcpStream;
use std::{
    task::Poll,
    fmt,
    error::Error
};

pub struct WebSocketsMiddleware<TInterceptor: Interceptor> {
    pub config: ServerConfig<TInterceptor>,
    // pub ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    // pub response: http::Response<()>
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
}

impl<TInterceptor: Interceptor> WebSocketsMiddleware<TInterceptor> {
    pub async fn new(config: ServerConfig<TInterceptor>) -> Self {
        let (a,b) = connect_async(&config.url).await.expect("Connection to Websocket server failed");
        let (c,d) = a.split();
        Self {
            config: config,
            // ws_stream: a,
            write: c,
            read: d,
        }
    }
    pub async fn connect(&self) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let url = url::Url::parse(&self.config.url).expect("Expected Websocket Url");
        let (a,_b) = connect_async(url).await.expect("Connection to Websocket server failed");
        a
    }

    pub async fn send(&mut self, msg: String) {
        match self.write.send(Message::Text(msg)).await {
            Ok(m) => println!("response {:?}", m),
            Err(e) => println!("Error: {:?}", e)
        };
        println!("message sent");
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