use scatter_gather_core as sgc;
use scatter_gather_core::middleware_specs::ServerConfig;
use sgc::connection::{
    Connection,
    ConnectionId,
    ConnectionHandlerInEvent,
    ConnectionHandlerOutEvent, 
    ConnectionHandler
};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::{
    connect_async, 
    tungstenite::protocol::{
        Message,
    },
    tungstenite::error::Error
};
use tokio::net::TcpStream;
use futures::{
    StreamExt, SinkExt,
    stream::{SplitSink, SplitStream},
    task::Poll,
};
use std::fmt;


// Declares the middleware Factory with an associated generic type. 
#[derive(Debug)]
pub struct WebSocketsMiddleware {
    pub config: ServerConfig,
    // pub ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    // pub response: http::Response<()>
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
}

// Implement custom fucntionality for the middleware.
impl WebSocketsMiddleware {
    pub async fn new(config: ServerConfig) -> WebSocketsMiddleware {
        Self::spin_up(config).await
    }
    
    async fn spin_up(config: ServerConfig) -> 
        Self
    {
        let (ws_stream,_b) = connect_async(&config.url).await.expect("Connection to Websocket server failed");
        let (mut write,read) = ws_stream.split();
        if let Some(init_handle) = &config.init_handle {
            match write.send(Message::Text(init_handle.to_string())).await {
                Ok(m) => {
                    #[cfg(debug_assertions)]
                    println!("Connection Response : {:?}", m)
                },
                Err(e) => {
                    #[cfg(debug_assertions)]
                    println!("Initialization Error: {:?}", e)
                }
            };
        };
        Self {
            config,
            write,
            read
        }
    }

    pub async fn send(&mut self, msg: String) {
        match self.write.send(Message::Text(msg)).await {
            Ok(m) => {
                #[cfg(debug_assertions)]
                println!("Response {:?}", m)
            },
            Err(e) => {
                #[cfg(debug_assertions)]
                println!("Error: {:?}", e)
            }
        };
        #[cfg(debug_assertions)]
        println!("message sent.");
    }

    pub fn get_stream(self) -> SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        self.read
    }
}

// Define possible errors.
#[derive(Debug)]
pub enum ConnectionHandlerError {
    Custom
}

// Define a way to debug the errors.
impl fmt::Display for ConnectionHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Custom error")
    }
}

// implement ConnectionHandler for the middleware
// This will facilitate the integration with the other elements of the suite.
impl<'b> ConnectionHandler<'b> for WebSocketsMiddleware {
    type InEvent = ConnectionHandlerInEvent;
    type OutEvent = ConnectionHandlerOutEvent<Connection>;

    fn inject_event(&mut self, event: Self::InEvent) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(debug_assertions)]
        println!("Inject debug: InEvent: {:?}", event);
        Ok(())
    }

    fn eject_event(&mut self, _event: Self::OutEvent) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn poll(
            self,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::OutEvent> 
    {
        let connection: Connection = Connection {
            id : ConnectionId::new(0),
            source_type: ServerConfig {
                url: self.config.url.clone(),
                prefix: self.config.prefix.clone(),
                init_handle: self.config.init_handle,
            },
        };        
        let event = ConnectionHandlerOutEvent::ConnectionEvent(connection);
        Poll::Ready(event)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
}

