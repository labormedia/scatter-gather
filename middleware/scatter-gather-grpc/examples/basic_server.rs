use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

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
pub struct OrderBook { }

impl OrderBook {
    pub fn new() -> Self {
        Self { }
    }
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderBook {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;
    async fn book_summary(&self, _request: Request<Empty>) -> Result<Response<Self::BookSummaryStream>, Status>{
        let (tx, rx) = mpsc::channel(20);

        tokio::spawn(async move {
            // tx.send(42);
        });

        let response = Response::new(ReceiverStream::new(rx)) ;
        Ok(response)
    }

    async fn book_summary_feed(&self, stream: tonic::Request<tonic::Streaming<Summary>>) -> Result<tonic::Response<orderbook::Empty>, Status>  {
        for i in 5..9 {
            self.tx.clone().send(Ok(Summary 
                { 
                    spread: 0.001*i as f64, 
                    bids: [Level { exchange: String::from("best"), 
                    price: 0.2, amount: 0.4 } ].to_vec(), 
                    asks: [].to_vec()
                })).await;
            
        };
        Ok(Response::new(orderbook::Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: _ = "[::1]:54001".parse().unwrap();
    let order_book_aggregator = OrderBook::new();
    let service = OrderbookAggregatorServer::new(order_book_aggregator);
    Server::builder().add_service(service).serve(addr).await?;
    Ok(())
}