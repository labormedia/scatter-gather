use futures::stream::{SplitSink, SplitStream};
use scatter_gather_core as sgc;
use scatter_gather_core::middleware_specs::{
    ServerConfig
};
use sgc::connection::{
    ConnectionHandlerInEvent,
    ConnectionHandlerOutEvent, ConnectionHandler
};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::{
    connect_async, 
    tungstenite::protocol::Message,
};
use futures::{
    StreamExt, SinkExt
};
use tokio::net::TcpStream;
use std::{
    task::Poll,
    fmt
};

pub struct WebSocketsMiddleware<TInterceptor: ConnectionHandler> {
    pub config: ServerConfig<TInterceptor>,
    // pub ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    // pub response: http::Response<()>
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
}

impl<TInterceptor: ConnectionHandler> WebSocketsMiddleware<TInterceptor> {
    pub async fn new(config: ServerConfig<TInterceptor>) -> Self {
        let (mut write,read) = Self::spin_up(&config).await;
        if let Some(init_handle) = &config.init_handle {
            match write.send(Message::Text(init_handle.to_string())).await {
                Ok(m) => println!("response {:?}", m),
                Err(e) => println!("Initialization Error: {:?}", e)
            };
        };
        Self {
            config,
            write,
            read
        }
    }
    
    async fn spin_up(config: &ServerConfig<TInterceptor>) -> (
        SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, 
        SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
    ) {
        let (a,_b) = connect_async(&config.url).await.expect("Connection to Websocket server failed");
        a.split()
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

impl<TInterceptor: ConnectionHandler> sgc::connection::ConnectionHandler for WebSocketsMiddleware<TInterceptor> {
    type InEvent = ConnectionHandlerInEvent;
    type OutEvent = ConnectionHandlerOutEvent<()>;

    fn inject_event(&mut self, event: Self::InEvent) {
        
    }

    fn poll(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::OutEvent> {
        Poll::Ready(ConnectionHandlerOutEvent::ConnectionEvent(()))
    }
}