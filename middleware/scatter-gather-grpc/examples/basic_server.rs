use tonic::{transport::Server, Request, Response, Status};

use orderbook::orderbook_aggregator_server::{
    OrderbookAggregator,
    OrderbookAggregatorServer
};
use orderbook::{
    Summary,
    Level
};

pub mod orderbook {
    tonic::include_proto!("orderbook"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct OrderBook {}



#[tokio::main]
async fn main() {
    ()
}