
use scatter_gather_core::{
    connection::{
        Connection,
        ConnectionId,
        ConnectionHandler,
        ConnectionHandlerInEvent,
        ConnectionHandlerOutEvent,
    },
    middleware_specs::{
        ServerConfig,
        Interceptor
    },
};
use std::{
    task::Poll,
    fmt,
    
};
use futures::{
    Future,
    stream::{
        SplitSink, 
        SplitStream
    }, 
    io::Empty
};
use schema_specific::orderbook::{Summary, self, orderbook_aggregator_server::OrderbookAggregator};
use tonic::{
    transport::Server, 
    Request, 
    Response, 
    Status,
    codegen::Arc
};
// use tokio_stream::wrappers::ReceiverStream;
use tokio::
    {
        sync::{
        Mutex,
        mpsc::{
            self, 
            // Sender,
            // Receiver
        },
        broadcast::{
            self,
            channel,
            Receiver,Sender,
        },
    }, 
};
use tokio_stream::{
    wrappers::{
        BroadcastStream,
        ReceiverStream
    },
    StreamExt,
    Stream
};
use tonic::codegen::Pin;

pub mod schema_specific;
const ADDRESS: &str = "http://[::1]:54001";

#[derive(Debug)]
pub struct GrpcMiddleware<TInterceptor: for <'a> ConnectionHandler<'a>> {
    pub config: ServerConfig<TInterceptor>,
    pub write: mpsc::Sender<Result<Summary, Status>>,
    read: mpsc::Receiver<Result<Summary, Status>>
}


impl<TInterceptor: for <'a> ConnectionHandler<'a>> GrpcMiddleware<TInterceptor> {

    pub async fn new(config: ServerConfig<TInterceptor>) -> GrpcMiddleware<TInterceptor> {
        let (write, read): (mpsc::Sender<Result<Summary, Status>>, mpsc::Receiver<Result<Summary, Status>>) = mpsc::channel(32);
        let (in_broadcast, mut _out_broadcast): (broadcast::Sender<Result<Summary, Status>>, broadcast::Receiver<Result<Summary, Status>>) = broadcast::channel(32);
        let in_broadcast_clone = broadcast::Sender::clone(&in_broadcast);
        let channels = schema_specific::OrderBook::new(in_broadcast_clone);
        schema_specific::server(ADDRESS, channels).expect("Couldn't start server.");
        Self {
            config,
            write,
            read
        }

    }

    async fn client_buf(&mut self, mut _buffer: mpsc::Receiver<Result<Summary, Status>>) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(debug_assertions)]
        println!("Starting Client (Buffer).");
        // Creates a new client for accesing the gRPC service.
        let mut channel = schema_specific::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
            .await?;
    
        while let Some(Ok(msg)) = self.read.recv().await {
            #[cfg(debug_assertions)]
            println!("Received while in client_buf: {:?}", msg);
            let input = futures::stream::iter([msg]).take(1);
            let request = tonic::Request::new(input);
            channel.book_summary_feed(request).await?;
        }
        Ok(())
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

// Implement ConnectionHandler for the middleware.

impl<'b, T: for<'a> ConnectionHandler<'a> + Interceptor + Sync + fmt::Debug> ConnectionHandler<'b> for GrpcMiddleware<T> {
    type InEvent = ConnectionHandlerInEvent<Result<Summary, Status>>;
    type OutEvent = ConnectionHandlerOutEvent<Connection<GrpcMiddleware<T>>>;

    fn inject_event(&mut self, event: Self::InEvent) {
        #[cfg(debug_assertions)]
        println!("Injecting event on GrpcMiddleware. {:?}", event);
    }
    fn eject_event(& mut self, event: Self::OutEvent) -> Self::OutEvent {
        event
    }

    fn poll(
        self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::OutEvent> 
    {
        // Poll::Ready(ConnectionHandlerOutEvent::ConnectionEvent(()))
        let connection = Connection {
            id : ConnectionId::new(0),
            source_type: ServerConfig {
                url: self.config.url.clone(),
                prefix: self.config.prefix.clone(),
                init_handle: self.config.init_handle.clone(),
                handler: self
            }
        };
        Poll::Ready(Self::OutEvent::ConnectionEvent(connection))
        // Poll::Pending
    }
}