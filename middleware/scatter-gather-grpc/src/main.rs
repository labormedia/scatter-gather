use scatter_gather_grpc::schema_specific::{self, OrderBook, orderbook::Summary, orderbook::{Level, orderbook_aggregator_server::OrderbookAggregator}};
use tokio::sync::broadcast;
use std::{
    thread,
    time, iter::Sum
};
use tonic::{
    codegen::Arc,
    Status
};
use tokio::sync::mpsc;
use tokio_stream::{
    wrappers::{
        BroadcastStream,
        ReceiverStream
    },
    StreamExt,
    Stream
};

const ADDRESS: &str = "http://[::1]:54001";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (in_channel, out_channel): (mpsc::Sender<Result<Summary, Status>>, mpsc::Receiver<Result<Summary, Status>>) = tokio::sync::mpsc::channel(32);
    let (in_broadcast, mut _out_broadcast): (broadcast::Sender<Result<Summary, Status>>, broadcast::Receiver<Result<Summary, Status>>) = broadcast::channel(32);
    let in_broadcast_clone = broadcast::Sender::clone(&in_broadcast);
    let channels = OrderBook::new(in_broadcast_clone);
    
    schema_specific::server(ADDRESS, channels).await.unwrap();

    let new_state = Summary { spread: 0.001*23 as f64, bids: [Level { exchange: String::from("best"), price: 0.2, amount: 0.4 } ].to_vec(), asks: [].to_vec() };
    println!("Creating orderbook");


    let in_broadcast_clone2 = broadcast::Sender::clone(&in_broadcast); 

    let client_buf = tokio::spawn( async move {
        // let mut mpsc_receiver_stream = ReceiverStream::new(out_channel);
        let _ = client_buf(out_channel).await; 
    } );
    let _client = tokio::spawn( async { let _ = client(1).await; } );
    let _client = tokio::spawn( async { let _ = client(2).await; } );
    let _client = tokio::spawn( async { let _ = client(3).await; } );
    let _streamer = tokio::spawn(async move {
        for i in 66..=168 {
            println!("Sending message from streamer.");
            in_channel.send(
                Ok(                
                    Summary 
                        { 
                            spread: 0.001*i as f64, 
                            bids: [Level { exchange: String::from("very best"), 
                            price: 0.2, amount: 0.4 } ].to_vec(), 
                            asks: [].to_vec()
                        })).await;
        }

    });
    client_buf.await;
    println!("Released");
    Ok(())
}

async fn client_buf(mut buffer: mpsc::Receiver<Result<Summary, Status>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Client (Buffer).");

    let mut channel = schema_specific::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
        .await?;

    while let Some(Ok(msg)) = buffer.recv().await {
        println!("Received while in client_bud: {:?}", msg);
        let input = futures::stream::iter([msg]).take(1);
        let request = tonic::Request::new(input);
        channel.book_summary_feed(request).await;
    }


    Ok(())
}

async fn client(id: i32) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Client (Feed).");

    let mut channel = schema_specific::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient::connect(ADDRESS)
        .await?;
    let request = tonic::Request::new( schema_specific::orderbook::Empty {});
    let mut stream = channel.book_summary(request).await?.into_inner();
    println!("RESPONSE={:?}", stream);
    while let Ok(item) = stream.message().await {
        match item {
            Some( a) => println!("\tClient {:?} Item: {:?}", id, a),
            None => { println!("None") }
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
        orderbook::{
            Level, 
            orderbook_aggregator_server::OrderbookAggregator
        }
    };
    use tokio_stream::StreamExt;
    use tokio::sync::broadcast;
    use tonic::Status;

    #[tokio::test]
    async fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting Client (Buffer).");
        let (in_broadcast, mut _out_broadcast): (broadcast::Sender<Result<Summary, Status>>, broadcast::Receiver<Result<Summary, Status>>) = broadcast::channel(32);
        let in_broadcast_clone = broadcast::Sender::clone(&in_broadcast);
        let channels = OrderBook::new(in_broadcast_clone);
        
        schema_specific::server(ADDRESS, channels).await.unwrap();
        let client_buf = tokio::spawn( async move {
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
    
        channel.book_summary_feed(request).await;
    
    
        Ok(())
    }

}