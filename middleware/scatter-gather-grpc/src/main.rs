use scatter_gather_grpc::schema_specific::{self, OrderBook, orderbook::Summary, orderbook::Level};
use tokio::sync::broadcast;
use std::{
    thread,
    time
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
    let handle = tokio::spawn(async move {
        let mut mpsc_receiver_stream = ReceiverStream::new(out_channel);
        let channels = OrderBook::new(in_channel, in_broadcast_clone);
        schema_specific::server(ADDRESS, channels).await.unwrap();
        // we test the broadcast a priori
        in_broadcast
            .send(Ok(Summary::default())).expect("sender: it should voice to receiver sent successfully");
        println!("stream tested");
        // We block the task to the channel bundling
        loop {
            match mpsc_receiver_stream.next().await {
                Some(data) => {
                        println!("got data: {:?}", data);
                        in_broadcast
                            .send(data)
                            .expect("sender: it should voice to receiver sent successfully");
                },
                None => {
                    println!("debugging on None");
                    in_broadcast
                    .send(Ok(Summary 
                        { 
                            spread: 0.001*7 as f64, 
                            bids: [Level { exchange: String::from("dummy data"), 
                            price: 0.2, amount: 0.4 } ].to_vec(), 
                            asks: [].to_vec() 
                        }))
                    .expect("sender: it should voice to receiver sent successfully");
                }
            }
        }
        // while let Some(data) = mpsc_receiver_stream.next().await {
        //     println!("got data: {:?}", data);
        //     in_broadcast
        //         .send(Ok(Summary 
        //             { 
        //                 spread: 0.001*5 as f64, 
        //                 bids: [Level { exchange: String::from("best"), 
        //                 price: 0.2, amount: 0.4 } ].to_vec(), 
        //                 asks: [].to_vec() 
        //             }))
        //         .expect("sender: it should voice to receiver sent successfully");
        // }
    });
    let new_state = Summary { spread: 0.001*2 as f64, bids: [Level { exchange: String::from("best"), price: 0.2, amount: 0.4 } ].to_vec(), asks: [].to_vec() };
    println!("Creating orderbook");

    // channels.tx.send(Ok(new_state.clone()));
    // channels.tx.send(Ok(new_state.clone()));
    let _client = tokio::spawn( async { let _ = client().await; } );
    let _ = client().await;
    handle.await?;
    

    // let _check = client().await;
    println!("Released");
    Ok(())
}


async fn client() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Client (Feed).");

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