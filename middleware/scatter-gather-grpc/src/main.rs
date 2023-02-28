use scatter_gather_grpc::*;
use tonic::transport::Endpoint;
use std::{
    thread,
    time
};


const ADDRESS: &str = "http://[::1]:54001";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _thread = tokio::spawn( async { let _ = scatter_gather_grpc::server().await; } );
    let mille_plateaux = time::Duration::from_millis(1000);
    thread::sleep(mille_plateaux);
    let _client = client().await;
    println!("Released");
    Ok(())
}


async fn client() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Client.");
    // let channel = Endpoint::from_static(ADDRESS)
    //     .connect()
    //     .await?;

    let mut channel = orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
        .await?;

    // let mut client = orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::with_interceptor(channel, intercept);

    let request = tonic::Request::new(orderbook::Empty {});

    let mut stream = channel.book_summary(request).await.unwrap().into_inner();// .book_summary(request).await?.into_inner() ;

    while let Ok(item) = stream.message().await {
        match item {
            Some(ref a) => println!("\treceived: {:?}", item),
            None => println!("\treceived: {:?}", item)
        }
        
    }

    println!("RESPONSE={:?}", stream);

    Ok(())
}