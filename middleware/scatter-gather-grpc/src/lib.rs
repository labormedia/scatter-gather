
use scatter_gather_core::{
    connection::{
        Connection,
        ConnectionId,
        ConnectionHandler,
        ConnectionHandlerInEvent,
        ConnectionHandlerOutEvent,
    },
    middleware_interface::{
        NodeConfig,
        Interceptor
    },
};
use core::task::{
    Poll,
    Context
};
use std::{
    fmt,
    thread,
    time,
    error::Error,
};
use schema_specific::orderbook::{
    Summary, orderbook_aggregator_client::OrderbookAggregatorClient,
};
use tonic::{
    Status,
    transport::Channel
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
use std::collections::VecDeque;

pub mod schema_specific;
const ADDRESS: &str = "http://[::1]:54001";

type GrpcConnection = Connection<usize>;

#[derive(Debug)]
pub struct GrpcMiddleware {
    pub config: NodeConfig,
    pub write: mpsc::Sender<ConnectionHandlerOutEvent<Result<Summary, Status>>>,
    pub read: mpsc::Receiver<ConnectionHandlerOutEvent<Result<Summary, Status>>>,
    client: OrderbookAggregatorClient<Channel>,
    pub state: VecDeque<Summary>
}


impl GrpcMiddleware {
    pub async fn new(config: NodeConfig) -> GrpcMiddleware {
        Self::spin_up(config).await.expect("Couldn't build Middleware.")
    }

    pub async fn try_new(config: NodeConfig) -> Result<GrpcMiddleware, Box<dyn Error>> {
        Ok(Self::spin_up(config).await?)
    }

    pub async fn spin_up(config: NodeConfig) -> Result<Self, Box<dyn std::error::Error>> {
      
        let (write, read): (mpsc::Sender<ConnectionHandlerOutEvent<Result<Summary, Status>>>, mpsc::Receiver<ConnectionHandlerOutEvent<Result<Summary, Status>>>) = mpsc::channel(32);
        let (in_broadcast, mut _out_broadcast): (broadcast::Sender<Result<Summary, Status>>, broadcast::Receiver<Result<Summary, Status>>) = broadcast::channel(32);
        let in_broadcast_clone = broadcast::Sender::clone(&in_broadcast);
        let channels = schema_specific::OrderBook::new(in_broadcast_clone);        
        schema_specific::server(ADDRESS, channels)?;
        thread::sleep(time::Duration::from_secs(5));
        // Creates a new client for accesing the gRPC service.
        let client_channel = schema_specific::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
        .await?;

        let i = Self {
            config,
            write,
            read,
            client: client_channel,
            state: VecDeque::with_capacity(10)
        };
        Ok(i)
    }

    pub async fn client_buf(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // #[cfg(debug_assertions)]
        // println!("Starting Client (Buffer).");

        if let Some(ConnectionHandlerOutEvent::ConnectionEvent(Ok(msg))) = self.read.recv().await {
            // #[cfg(debug_assertions)]
            // println!("Received while in client_buf: {:?}", msg);
            let input = futures::stream::iter([msg]).take(1);
            let request = tonic::Request::new(input);
            self.client.book_summary_feed(request).await?;
        };
        // #[cfg(debug_assertions)]
        // println!("Leaving client_buf.");
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

// Implement ConnectionHandler for the middleware.

impl<'b> ConnectionHandler<'b> for GrpcMiddleware {
    type InEvent = ConnectionHandlerOutEvent<Result<Summary, Status>>;
    type OutEvent = ConnectionHandlerOutEvent<GrpcConnection>;

    fn inject_event(&mut self, event: Self::InEvent) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(debug_assertions)]
        println!("Injecting event on GrpcMiddleware. {:?}", event);
        Ok(())
    }
    fn eject_event(& mut self, event: Self::OutEvent) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(debug_assertions)]
        println!("Ejecting event within GrpcMiddleware. {:?}", event);
        Ok(())
    }

    fn poll(
        self,
        _cx: &mut Context<'_>,
    ) -> Poll<Self::OutEvent> 
    {
        // Poll::Ready(ConnectionHandlerOutEvent::ConnectionEvent(()))
        let connection: GrpcConnection = Connection {
            id : ConnectionId(1),
            source_type: NodeConfig {
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