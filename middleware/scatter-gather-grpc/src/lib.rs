
use scatter_gather_core::{
    connection::{
        ConnectionHandler,
        ConnectionHandlerInEvent,
        ConnectionHandlerOutEvent
    },
    middleware_specs::{
        ServerConfig,
        Interceptor
    },
};
use std::{
    task::Poll,
    fmt
};
use futures::{
    Future,
    stream::{
        SplitSink, 
        SplitStream
    }, 
    io::Empty
};
use schema_specific::orderbook::{Summary, self};
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
    }
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
pub struct GrpcMiddleware<TInterceptor: ConnectionHandler> {
    pub config: ServerConfig<TInterceptor>,
    // pub ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    // pub response: http::Response<()>
    // Will the channel need additional smart pointers ? we'll figure it out.
    pub write: mpsc::Sender<Result<Summary,Status>>,
    pub read: broadcast::Sender<Result<Summary, Status>> //Arc<Mutex<Receiver<Result<Summary, Status>>>> 
}


impl<TInterceptor: ConnectionHandler> GrpcMiddleware<TInterceptor> {

    pub async fn new(config: ServerConfig<TInterceptor>) -> Self {
        let (in_channel, out_channel): (mpsc::Sender<Result<Summary, Status>>, mpsc::Receiver<Result<Summary, Status>>) = tokio::sync::mpsc::channel(32);
        let (in_broadcast, mut _out_broadcast): (broadcast::Sender<Result<Summary, Status>>, broadcast::Receiver<Result<Summary, Status>>) = broadcast::channel(32);
        let in_broadcast_clone = broadcast::Sender::clone(&in_broadcast);
        let mut mpsc_receiver_stream = ReceiverStream::new(out_channel);
        let channels = schema_specific::OrderBook::new(in_channel, in_broadcast);

        Self {
            config,
            write: channels.tx,
            read: channels.rx // Arc::new(Mutex::new(read))
        }

    }

    // async fn spin_up(config: &ServerConfig<TInterceptor>) -> (
    //     mpsc::Sender<Result<Summary,Status>>, 
    //     broadcast::Sender<Result<Summary, Status>>
    // ) {


    //     // generic blocking channel
    //     (in_channel, in_broadcast_clone)
    // }

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

impl<T: ConnectionHandler + Interceptor> ConnectionHandler for GrpcMiddleware<T> {
    type InEvent = ConnectionHandlerInEvent<()>;
    type OutEvent = ConnectionHandlerOutEvent<()>;

    fn inject_event(&mut self, event: Self::InEvent) {}
    fn eject_event(&mut self, event: Self::OutEvent) {}

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::OutEvent> 
    {
        Poll::Pending
    }
}