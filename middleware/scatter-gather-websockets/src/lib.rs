use scatter_gather_core as sgc;
use scatter_gather_core::middleware_specs::ServerConfig;
use sgc::connection::{
    ConnectionHandlerInEvent,
    ConnectionHandlerOutEvent, 
    ConnectionHandler
};
use sgc::middleware_specs::Interceptor;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::{
    connect_async, 
    tungstenite::protocol::Message,
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
pub struct WebSocketsMiddleware<TInterceptor: for<'a> ConnectionHandler<'a>> {
    pub config: ServerConfig<TInterceptor>,
    // pub ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    // pub response: http::Response<()>
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
}

// Implement custom fucntionality for the middleware.
impl<TInterceptor: for<'a> ConnectionHandler<'a>> WebSocketsMiddleware<TInterceptor> {
    pub async fn new(config: ServerConfig<TInterceptor>) -> WebSocketsMiddleware<TInterceptor> {
        let (mut write,read) = Self::spin_up(&config).await;
        if let Some(init_handle) = &config.init_handle {
            match write.send(Message::Text(init_handle.to_string())).await {
                Ok(m) => println!("Connection Response : {:?}", m),
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
        let (ws_stream,_b) = connect_async(&config.url).await.expect("Connection to Websocket server failed");
        ws_stream.split()
    }

    pub async fn send(&mut self, msg: String) {
        match self.write.send(Message::Text(msg)).await {
            Ok(m) => println!("Response {:?}", m),
            Err(e) => println!("Error: {:?}", e)
        };
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
impl<'b, TInterceptor: for<'a> ConnectionHandler<'a> + Interceptor + Sync + fmt::Debug> ConnectionHandler<'b> for WebSocketsMiddleware<TInterceptor> {
    type InEvent = ConnectionHandlerInEvent<Message>;
    type OutEvent = ConnectionHandlerOutEvent<TInterceptor>;

    fn inject_event(&mut self, event: Self::InEvent) {
        println!("Inject debug: InEvent: {:?}", event);
    }

    fn eject_event(&mut self, event: Self::OutEvent) -> ConnectionHandlerOutEvent<TInterceptor> {
        event
    }

    fn poll(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::OutEvent> 
    {
        loop {
            match &self.read.poll_next_unpin(cx) {
                Poll::Ready(None) => {},
                Poll::Ready(a) => {
                    println!("Read message: {:?}", a);
                },
                Poll::Pending => {}
            }
            return Poll::Pending
        } 

    }
}

