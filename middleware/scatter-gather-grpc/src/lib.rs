
use scatter_gather_core::{
    connection::ConnectionHandler,
    middleware_specs::ServerConfig
};
use futures::{stream::{
    SplitSink, 
    SplitStream
}, io::Empty};
use schema_specific::orderbook::{Summary, self};
use tonic::{
    transport::Server, 
    Request, 
    Response, 
    Status,
    codegen::Arc
};
// use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::{
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
    pub write: Sender<Result<Summary,Status>>,
    pub read: Arc<Mutex<Receiver<Result<Summary, Status>>>> 
}


impl<TInterceptor: ConnectionHandler> GrpcMiddleware<TInterceptor> {

    pub async fn new(config: ServerConfig<TInterceptor>) -> Self {
        let (mut write,read) = Self::spin_up(&config).await;

        // There's no init handle in this case (Empty).

        // if let Some(init_handle) = &config.init_handle {
        //     match write.send(Some(orderbook::Empty {})).await {
        //         Ok(m) => println!("Connection Response : {:?}", m),
        //         Err(e) => println!("Initialization Error: {:?}", e)
        //     };
        // };
        Self {
            config,
            write,
            read: Arc::new(Mutex::new(read))
        }
    }
    async fn spin_up(config: &ServerConfig<TInterceptor>) -> (
        Sender<Result<Summary,Status>>, 
        Receiver<Result<Summary, Status>>
    ) {
        let mut channel = orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
        .await.expect("Unable to build service.");
        broadcast::channel(10)
    }

}