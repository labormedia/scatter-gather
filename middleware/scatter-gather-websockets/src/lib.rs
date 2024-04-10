use scatter_gather_core as sgc;
use scatter_gather_core::middleware_interface::{
    NodeConfig,
    GetStream,
};
use sgc::connection::{
    Connection,
    ConnectionId,
    ConnectionHandlerInEvent,
    ConnectionHandlerOutEvent, 
    ConnectionHandler
};
pub use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::{
    connect_async, 
    tungstenite::{
            protocol::{
            Message,
        },
        http::Response,
    },
    // tungstenite::error::Error
};
use tokio::net::TcpStream;
use core::{
    task::{
        Poll,
        Context,
    },
    pin::Pin,
};
use futures::{
    Future,
    StreamExt, SinkExt,
    stream::{SplitSink, SplitStream, Next,},
    Stream,
};
use std::{
    fmt,
    error::Error,
};

type WebSocketConnection = Connection<usize>;

// Declares the middleware Factory with an associated generic type. 
#[derive(Debug)]
pub struct WebSocketsMiddleware {
    pub config: NodeConfig,
    // pub ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    // pub response: http::Response<()>
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
}

// Implement custom fucntionality for the middleware.
impl WebSocketsMiddleware {
    pub async fn new(config: NodeConfig) -> WebSocketsMiddleware {
        Self::spin_up(config).await.expect("Couldn't build Middleware.")
    }

    pub async fn connect(config: NodeConfig) -> Pin<Box<dyn Future<Output = Result<(WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Response<()>), tokio_tungstenite::tungstenite::Error>> + Send>>
    {
        Box::pin(connect_async(config.url))
    }

    pub async fn try_new(config: NodeConfig) -> Result<WebSocketsMiddleware, Box<dyn Error>> {
        Ok(Self::spin_up(config).await?)
    }

    pub async fn try_connect(config: NodeConfig) -> Result<Pin<Box<dyn Future<Output = Result<(WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Response<()>), tokio_tungstenite::tungstenite::Error> > + Send>>, Box<dyn Error + Send>> {
        Ok(Box::pin(connect_async(config.url)))
    }

    pub async fn init_handle(&mut self) -> Result<(), Box<dyn Error>>{
        if let Some(init_handle) = &self.config.init_handle {
            self.write.send(Message::Text(init_handle.to_string())).await?;
        };
        Ok(())
    }
    
    pub async fn spin_up(config: NodeConfig) -> Result<Self, tokio_tungstenite::tungstenite::Error>
    {
        let (ws_stream,_b) = connect_async(&config.url).await?;
        // let (ws_stream, _) = (Self::try_connect(config.clone()).await?).await.unwrap();
        let (mut write,read) = ws_stream.split();
        let mut new_ws = Self {
                config,
                write,
                read
            };
        new_ws.init_handle().await.expect("Could not initialize middleware channel.");  // TODO: Handle error.
        Ok(new_ws)
    }

    pub async fn send(&mut self, msg: String) -> Result<(), Box<dyn Error>> {
        let _ = self.write.send(Message::Text(msg)).await?;
        #[cfg(debug_assertions)]
        println!("message sent.");
        Ok(())
    }
}

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

// implement ConnectionHandler for the middleware
// This will facilitate the integration with the other elements of the suite.
impl<'b> ConnectionHandler<'b> for WebSocketsMiddleware {
    type InEvent = ConnectionHandlerInEvent;
    type OutEvent = ConnectionHandlerOutEvent<WebSocketConnection>;

    fn inject_event(&mut self, event: Self::InEvent) -> Result<(), Box<dyn Error>> {
        #[cfg(debug_assertions)]
        println!("Inject debug: InEvent: {:?}", event);
        match event {
            ConnectionHandlerInEvent::Connect => {

            },
            ConnectionHandlerInEvent::Disconnect => {

            },
            ConnectionHandlerInEvent::Intercept => {

            },
        };
        Ok(())
    }

    fn eject_event(&mut self, _event: Self::OutEvent) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn poll(
            self,
            _cx: &mut Context<'_>,
        ) -> Poll<Self::OutEvent> 
    {
        let connection: WebSocketConnection = Connection {
            id : ConnectionId(0),
            source_type: self.config.clone(),
        };        
        let event = ConnectionHandlerOutEvent::ConnectionEvent(connection);
        Poll::Ready(event)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
}

impl GetStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> for WebSocketsMiddleware {
    fn get_stream(self) -> SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> {
        self.read
    }
    fn next<'a>(&'a mut self) -> Next<'a,SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> > {
        self.read.next()
    }
}