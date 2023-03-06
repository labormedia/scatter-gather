use std::iter::Sum;
use std::thread::JoinHandle;
use std::{
    time,
    thread,
};
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
    BroadcastStream,
    channel,
    broadcast,
    StreamExt,
    Stream,
    Pin
};
use futures::FutureExt;
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
    pub tx: mpsc::Sender<Result<Summary, Status>>, // Arc<Mutex<Sender<Result<Summary, Status>>>>,
    // pub rx: Arc<Mutex<Receiver<Result<Summary, Status>>>>,
    pub rx: tokio::sync::broadcast::Sender<Result<Summary, Status>>,//Receiver<Result<Summary, Status>>,//Arc<Mutex<Receiver<Result<Summary, Status>>>>,
    // pub collector: Vec<tokio::task::JoinHandle<()>>
    last_state: Summary,
    state_buffer: Arc<Mutex<Vec<Summary>>>
}

impl OrderBook {
    pub fn new(mpsc_sender: mpsc::Sender<Result<Summary, Status>>, broadcaster: Sender<Result<Summary, Status>>) -> Self {
        // let (tx, rx) = channel(20);
        // let another_broadcast_sender = Sender::clone(&tx);
        println!("Injecting channels.");
        Self {
            tx: mpsc_sender, 
            // rx: Arc::new(Mutex::new(rx)),
            rx: broadcaster,
            // collector: vec!()
            last_state: Summary::default(),
            state_buffer: Arc::new(Mutex::new(vec!()))
         }
    }
    pub fn update_collector(mut self, state: Summary) {
        self.last_state = state;
    }
    pub async fn send_state(&mut self, state:Summary) {
        let arc_ref = Arc::clone(&self.state_buffer);
        let mut lock = arc_ref.lock().await;
        // lock.send(Ok(Summary::default())).await;
        lock.push(state);
    }
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderBook {
    // type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;
    type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send + Sync>>;
    async fn book_summary(&self, _request: Request<Empty>) -> Result<Response<Self::BookSummaryStream>, Status>
    {
        println!("Starting book summary response");
        let rx2 = self.rx.subscribe();
        // let lock = &self.state_buffer.lock().await.iter();
        // let handle = tokio::spawn( async move {
        //     println!("End of handle");
        //     // rx2.recv();
        //     // let a = lock.map(|c| {});
        //     let test_msg = Summary 
        //         { 
        //             spread: 0.11 as f64, 
        //             bids: [Level { exchange: String::from("best"), 
        //             price: 0.2, amount: 0.4 } ].to_vec(),   
        //             asks: [].to_vec()
        //         };


        //     return Ok(test_msg)
        // });
        println!("Continue");
        let stream = BroadcastStream::new(rx2)
            .filter_map(|res| { res.ok() });
            // .map(Ok);
        let stream: Self::BookSummaryStream = Box::pin(stream);
        let response = Response::new(stream);
        Ok(response)
    }
    async fn book_summary_feed(&self, stream: tonic::Request<tonic::Streaming<Summary>>) -> Result<tonic::Response<orderbook::Empty>, Status>  {
        println!("book_summary_feed");
        self.tx.send(Ok(Summary 
            { 
                spread: 0.017 as f64, 
                bids: [Level { exchange: String::from("best"), 
                price: 0.2, amount: 0.4 } ].to_vec(), 
                asks: [].to_vec()
            })).await;
        for i in 5..9 {
            self.rx.clone().send(Ok(Summary 
                { 
                    spread: 0.001*i as f64, 
                    bids: [Level { exchange: String::from("best"), 
                    price: 0.2, amount: 0.4 } ].to_vec(), 
                    asks: [].to_vec()
                }));
            
        };
        Ok(Response::new(orderbook::Empty {}))
    }
}

pub async fn server(address: &str, inner: OrderBook) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting service.");
    // let addr: _ = "[::1]:54001".parse()?;
    // let order_book_aggregator = OrderBook::new(sender);
    // let service = OrderbookAggregatorServer::with_interceptor(order_book_aggregator, intercept);
    let service = OrderbookAggregatorServer::new(inner);
    let b = tokio::spawn(async {
        println!("Building server.");
        Server::builder().add_service(service).serve("[::1]:54001".parse().unwrap()).await.unwrap();
        println!("Server builder passed");
    });
    
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