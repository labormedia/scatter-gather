use scatter_gather_grpc::schema_specific::{
    self, 
    OrderBook, 
    orderbook::{
        Summary, 
        Level
    }
};
use tokio::{
    runtime::Runtime,
    sync::{
        broadcast,
        mpsc
    }
};
use tonic::Status;
use tokio_stream::StreamExt;

const ADDRESS: &str = "http://[::1]:54001";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (in_channel, out_channel): (mpsc::Sender<Result<Summary, Status>>, mpsc::Receiver<Result<Summary, Status>>) = tokio::sync::mpsc::channel(32);
    let (in_broadcast, mut _out_broadcast): (broadcast::Sender<Result<Summary, Status>>, broadcast::Receiver<Result<Summary, Status>>) = broadcast::channel(32);
    let in_broadcast_clone = broadcast::Sender::clone(&in_broadcast);
    let channels = OrderBook::new(in_broadcast_clone);
    let rt = Runtime::new()?;
    schema_specific::server(ADDRESS, channels, rt).await?; 

    let client_buf = tokio::spawn( async move {
        let _ = client_buf(out_channel).await; 
    } );
    let _client = tokio::spawn( async { let _ = client(1).await; } );
    let _client = tokio::spawn( async { let _ = client(2).await; } );
    let _client = tokio::spawn( async { let _ = client(3).await; } );
    let _streamer = tokio::spawn(async move {
        for i in 66..=168 {
            #[cfg(debug_assertions)]
            println!("Sending message from streamer.");
            match in_channel.send(
                Ok(                
                    Summary 
                        { 
                            spread: 0.001*i as f64, 
                            bids: [Level { exchange: String::from("very best"), 
                            price: 0.2, amount: 0.4 } ].to_vec(), 
                            asks: [].to_vec()
                        }
                )
            ).await
                {
                    Ok(msg) => {
                        #[cfg(debug_assertions)]
                        println!("Send succesful, {:?}", msg);
                    },
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        println!("Received error while sending messages : {:?}", e);
                    }
                };
        }
    });
    client_buf.await?;
    #[cfg(debug_assertions)]
    println!("Released");
    Ok(())
}

async fn client_buf(mut buffer: mpsc::Receiver<Result<Summary, Status>>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    println!("Starting Client (Buffer).");
    // Creates a new client for accesing the gRPC service.
    let mut channel = schema_specific::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
        .await?;

    while let Some(Ok(msg)) = buffer.recv().await {
        #[cfg(debug_assertions)]
        println!("Received while in client_buf: {:?}", msg);
        let input = futures::stream::iter([msg]).take(1);
        let request = tonic::Request::new(input);
        channel.book_summary_feed(request).await?;
    }
    Ok(())
}

async fn client(id: i32) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    println!("Starting Client (Feed).");

    let mut channel = schema_specific::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
        .await?;
    let request = tonic::Request::new( schema_specific::orderbook::Empty {});
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

#[cfg(test)]
mod tests {
    const ADDRESS: &str = "http://[::1]:54001";
    use scatter_gather_grpc::schema_specific::{
        self, 
        OrderBook, 
        orderbook::Summary, 
        orderbook::Level, 
    };
    use tokio_stream::StreamExt;
    use tokio::{
        sync::broadcast,
        runtime::Runtime
    };
    use tonic::Status;

    #[tokio::test]
    async fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting Client (Buffer).");
        let (in_broadcast, mut _out_broadcast): (broadcast::Sender<Result<Summary, Status>>, broadcast::Receiver<Result<Summary, Status>>) = broadcast::channel(32);
        let in_broadcast_clone = broadcast::Sender::clone(&in_broadcast);
        let channels = OrderBook::new(in_broadcast_clone);
        let rt = Runtime::new()?;
        schema_specific::server(ADDRESS, channels, rt).await.unwrap();
        let _client_buf = tokio::spawn( async move {
            // let mut mpsc_receiver_stream = ReceiverStream::new(out_channel);
            let _ = client_buf_test().await; 
        } );
        Ok(())
    }

    async fn client_buf_test() -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting Client (Buffer).");
    
        let mut channel = schema_specific::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
            .await?;
    
        let point_count = 72;
        
        let mut points = vec![];
        for i in 51..=point_count {
            points.push(
                Summary 
                    { 
                        spread: 0.001*i as f64, 
                        bids: [Level { exchange: String::from("best"), 
                        price: 0.2, amount: 0.4 } ].to_vec(), 
                        asks: [].to_vec()
                    }
            );
    
        }
        let input = futures::stream::iter(points).take(15);
        let request = tonic::Request::new(input);
    
        channel.book_summary_feed(request).await?;
    
    
        Ok(())
    }

}