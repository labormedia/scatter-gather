use scatter_gather_grpc::schema_specific;
use std::{
    thread,
    time
};


const ADDRESS: &str = "http://[::1]:54001";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _thread = tokio::spawn( async { let _ = schema_specific::server(ADDRESS).await; } );
    let mille_plateaux = time::Duration::from_millis(1000);
    thread::sleep(mille_plateaux);
    let _client = client().await;
    println!("Released");
    Ok(())
}


async fn client() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Client.");

    let mut channel = schema_specific::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
        .await?;

    let request = tonic::Request::new( schema_specific::orderbook::Empty {});
    let mut stream = channel.book_summary(request).await?.into_inner(); 
    while let Ok(item) = stream.message().await {
        match item {
            Some(ref a) => println!("\titem: {:?}", a),
            None => println!("\treceived: {:?}", item)
        }
    }

    println!("RESPONSE={:?}", stream);
    Ok(())
}