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
    // pub tx: Sender<Result<Summary, Status>>,
    // pub rx: Arc<Mutex<Receiver<Result<Summary, Status>>>>,
    // pub rx: Receiver<Result<Summary, Status>>,
    // pub collector: Vec<tokio::task::JoinHandle<()>>
    last_state: Summary
}

impl OrderBook {
    pub fn new() -> Self {
        // let (tx, rx) = mpsc::channel(20);
        println!("Injecting channels.");
        Self {
            // tx, 
            // rx: Arc::new(Mutex::new(rx)),
            // rx
            // collector: vec!()
            last_state: Summary::default()
         }
    }
    pub fn update_collector(mut self, state: Summary) {
        self.last_state = state;
    }
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderBook {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;
    async fn book_summary(&self, _request: Request<Empty>) -> Result<Response<Self::BookSummaryStream>, Status>
    {
        println!("Starting response");
        let (tx, rx) = mpsc::channel(100);
        let state_clone = self.last_state.clone();


        let handle = tokio::spawn( async move {
            let test = Arc::new(Mutex::new(state_clone));

            let tx_clone = tx.clone();
            let arc_ref = Arc::clone(&test);

            for _ in 0..6 {
                let locked = arc_ref.lock().await.clone();
                match tx_clone.send(Ok(locked)).await {
                    
                    Ok(a) => {
                        println!("cool {:?}", a);
                        
                    },
                    Err(e) => println!("not cool {:?}", e),
                };
            }

            println!("End of handle");
        });
        println!("Continue");

        let response = Response::new(ReceiverStream::new(rx));
        let mille_plateaux = time::Duration::from_millis(1000);
        thread::sleep(mille_plateaux);
        match handle.await {
                    
            Ok(a) => {
                // println!("Handle {:?}", a);
                
            },
            Err(e) => println!("Erronous handle {:?}", e),
        };
        println!("ending response");

        Ok(response)
    }
}

pub async fn server(address: &str, inner: OrderBook) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting service.");
    let addr: _ = "[::1]:54001".parse()?;
    let order_book_aggregator = OrderBook::new();
    let service = OrderbookAggregatorServer::with_interceptor(order_book_aggregator, intercept);
    // let service = OrderbookAggregatorServer::new(inner);
    Server::builder().add_service(service).serve(addr).await?;
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