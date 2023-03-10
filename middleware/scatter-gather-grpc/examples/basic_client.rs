
mod orderbook {
    tonic::include_proto!("orderbook"); // The string specified here must match the proto package name
}
const ADDRESS: &str = "http://[::1]:54001";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    client(13).await?;
    Ok(())
}



async fn client(id: i32) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    println!("Starting Client (Feed).");
    let mut channel = orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
        .await?;
    let request = tonic::Request::new( orderbook::Empty {});
    let mut stream = channel.book_summary(request).await?.into_inner();
    #[cfg(debug_assertions)]
    println!("RESPONSE = {:?}", stream);
    while let Ok(item) = stream.message().await {
        match item {
            Some( a) => {
                #[cfg(debug_assertions)]
                println!("\tClient {:?} Item: {:?}", id, a)
            },
            None => { 
                #[cfg(debug_assertions)]
                println!("None") 
            }
        }
    }
    Ok(())
}