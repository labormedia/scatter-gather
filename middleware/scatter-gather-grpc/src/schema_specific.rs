use super::{
    Sender,
    Receiver,
    Status,
    ReceiverStream,
    Request,
    Response,
    Server,
    Arc,
    Mutex,
    mpsc,
};
use orderbook::orderbook_aggregator_server::{
    OrderbookAggregator,
    OrderbookAggregatorServer,
};
use orderbook::{
    Empty,
    Summary,
    Level,
};

pub mod orderbook {
    tonic::include_proto!("orderbook"); // The string specified here must match the proto package name
}

#[derive(Debug)]
pub struct OrderBook
{
    tx: Sender<Result<Summary, Status>>,
    rx: Arc<Mutex<Receiver<Result<Summary, Status>>>>
}

impl OrderBook {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(20);
        Self {
            tx, 
            rx: Arc::new(Mutex::new(rx))
         }
    }

}

#[tonic::async_trait]
impl OrderbookAggregator for OrderBook {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;
    async fn book_summary(&self, _request: Request<Empty>) -> Result<Response<Self::BookSummaryStream>, Status>
    {
        let (tx, rx) = mpsc::channel(20);
        let response = Response::new(ReceiverStream::new(rx)) ;
        Ok(response)
    }
}

pub async fn server(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting service.");
    let addr: _ = "[::1]:54001".parse()?;
    let order_book_aggregator = OrderBook::new();
    let service = orderbook::orderbook_aggregator_server::OrderbookAggregatorServer::with_interceptor(order_book_aggregator, intercept);
    Server::builder().add_service(service).serve(addr).await?;
    Ok(())
}

pub fn intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
    println!("Intercepting request: {:?}", req);

    req.extensions_mut().insert(Extended {
        intercepted_data: "intercepted".to_string(),
    });

    Ok(req)
}

struct Extended {
    intercepted_data: String,
}