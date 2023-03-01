use scatter_gather_grpc::schema_specific::{self, OrderBook, orderbook::Summary};
use std::{
    thread,
    time
};


const ADDRESS: &str = "http://[::1]:54001";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channels = OrderBook::new();
    let _thread = tokio::spawn( async { let _ = schema_specific::server(ADDRESS, channels).await; } );
    let _client = tokio::spawn( async { let _ = client().await; } );
    let _check = client().await;
    println!("Released");
    Ok(())
}


async fn client() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Client.");

    let mut channel = schema_specific::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
        .await?;

    let request = tonic::Request::new( schema_specific::orderbook::Empty {});
    let mut stream = channel.book_summary(request).await?.into_inner(); 
    println!("RESPONSE={:?}", stream);
    while let Ok(item) = stream.message().await {
        match item {
            Some(ref a) => println!("\titem: {:?}", a),
            None => {  }
        }
    }


    Ok(())
}