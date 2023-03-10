
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
    thread,
    time
};
use futures::{
    Future,
    stream::{
        SplitSink, 
        SplitStream, FuturesUnordered
    }, 
    io::Empty
};
use schema_specific::orderbook::{
    Summary,
};
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
            error::SendError
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

pub mod schema_specific;
const ADDRESS: &str = "http://[::1]:54001";

#[derive(Debug)]
pub struct GrpcMiddleware {
    pub config: ServerConfig,
    pub write: mpsc::Sender<ConnectionHandlerOutEvent<Result<Summary, Status>>>,
    read: mpsc::Receiver<ConnectionHandlerOutEvent<Result<Summary, Status>>>,
}


impl GrpcMiddleware {
    pub async fn new(config: ServerConfig) -> GrpcMiddleware {
        Self::spin_up(config).await.expect("Couldn't build Middleware.")
    }

    pub async fn spin_up(config: ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
      
        let (write, read): (mpsc::Sender<ConnectionHandlerOutEvent<Result<Summary, Status>>>, mpsc::Receiver<ConnectionHandlerOutEvent<Result<Summary, Status>>>) = mpsc::channel(32);
        let (in_broadcast, mut _out_broadcast): (broadcast::Sender<Result<Summary, Status>>, broadcast::Receiver<Result<Summary, Status>>) = broadcast::channel(32);
        let in_broadcast_clone = broadcast::Sender::clone(&in_broadcast);
        let channels = schema_specific::OrderBook::new(in_broadcast_clone);
        schema_specific::server(ADDRESS, channels).expect("Couldn't start server.");
        let mut i = Self {
            config,
            write,
            read,
        };
        thread::sleep(time::Duration::from_secs(5));
        Ok(i)
    }

    pub async fn client_buf(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(debug_assertions)]
        println!("Starting Client (Buffer).");
        // Creates a new client for accesing the gRPC service.
        #[cfg(debug_assertions)]
        println!("Reached.");
        let mut channel = schema_specific::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
            .await.expect("Cannot start gRPC client.");
        // self.write.send(ConnectionHandlerOutEvent::ConnectionEvent(Ok(Summary::default()))).await?;

        if let Some(ConnectionHandlerOutEvent::ConnectionEvent(Ok(msg))) = self.read.recv().await {
            #[cfg(debug_assertions)]
            println!("Received while in client_buf: {:?}", msg);
            let input = futures::stream::iter([msg]).take(10);
            let request = tonic::Request::new(input);
            channel.book_summary_feed(request).await.expect("Cannot buffer book_summary_feed.");
        };
        #[cfg(debug_assertions)]
        println!("Leaving client_buf.");
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

impl<'b> ConnectionHandler<'b> for GrpcMiddleware {
    type InEvent = ConnectionHandlerOutEvent<Result<Summary, Status>>;
    type OutEvent = ConnectionHandlerOutEvent<Connection>;

    fn inject_event(&mut self, event: Self::InEvent) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(debug_assertions)]
        println!("Injecting event on GrpcMiddleware. {:?}", event);
        Ok(())
    }
    fn eject_event(& mut self, event: Self::OutEvent) -> Result<(), SendError<ConnectionHandlerOutEvent<Connection>>> {
        #[cfg(debug_assertions)]
        println!("Ejecting event within GrpcMiddleware. {:?}", event);
        Ok(())
    }

    fn poll(
        self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<ConnectionHandlerOutEvent<Connection>> 
    {
        // Poll::Ready(ConnectionHandlerOutEvent::ConnectionEvent(()))
        let connection: Connection = Connection {
            id : ConnectionId::new(0),
            source_type: ServerConfig {
                url: self.config.url.clone(),
                prefix: self.config.prefix.clone(),
                init_handle: self.config.init_handle.clone(),
            },
        };
        let event = ConnectionHandlerOutEvent::ConnectionEvent(connection);
        Poll::Ready(event)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
}