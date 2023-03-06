use std::iter::Sum;
use std::thread::JoinHandle;
use std::{
    time,
    thread,
};
use tokio::
    {    
        runtime::Runtime,
        sync::{
            Mutex,
            mpsc::{
                self, 
                // Sender,
                // Receiver
            },
            broadcast::{
                self,
                channel,
                Receiver,Sender,
            },
        }
    };
use tokio_stream::{
    wrappers::{
        BroadcastStream,
        ReceiverStream
    },
    StreamExt,
    Stream
};
use tonic::{
    transport::Server, 
    Request, 
    Response, 
    Status,
    codegen::{
        Arc,
        Pin
    }
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
    pub rx: tokio::sync::broadcast::Sender<Result<Summary, Status>>,
    last_state: Summary,
    state_buffer: Arc<Mutex<Vec<Summary>>>,
    rt: Runtime
}

impl OrderBook {
    pub fn new(broadcaster: Sender<Result<Summary, Status>>) -> Self {
        #[cfg(debug_assertions)]
        println!("Injecting channels.");
        Self {
            rx: broadcaster,
            last_state: Summary::default(),
            state_buffer: Arc::new(Mutex::new(vec!())),
            rt: Runtime::new().unwrap()
         }
    }
    pub fn update_collector(mut self, state: Summary) {
        self.last_state = state;
    }
    pub async fn send_state(&mut self, state:Summary) {
        let arc_ref = Arc::clone(&self.state_buffer);
        let mut lock = arc_ref.lock().await;
        lock.push(state);
    }
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderBook {
    type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send + Sync>>;
    async fn book_summary(&self, _request: Request<Empty>) -> Result<Response<Self::BookSummaryStream>, Status>
    {
        #[cfg(debug_assertions)]
        println!("Starting book summary response");
        let rx2 = self.rx.subscribe();
        let stream = BroadcastStream::new(rx2)
            .filter_map(|res| { res.ok() });
        let stream: Self::BookSummaryStream = Box::pin(stream);
        let response = Response::new(stream);
        Ok(response)
    }
    async fn book_summary_feed(&self, stream: tonic::Request<tonic::Streaming<Summary>>) -> Result<tonic::Response<orderbook::Empty>, Status>  {
        #[cfg(debug_assertions)]
        println!("Starting book_summary_feed");
        let mut inner  = stream.into_inner();
        while let Some(check) = inner.next().await {
            #[cfg(debug_assertions)]
            println!("Check : {:?}", check);
            self.rx.clone().send(check);
        };
        Ok(Response::new(orderbook::Empty {}))
    }
}



pub fn intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
    #[cfg(debug_assertions)]
    println!("Intercepting request: {:?}", req);

    req.extensions_mut().insert(Extended {
        intercepted_data: "intercepted".to_string(),
    });

    Ok(req)
}

struct Extended {
    intercepted_data: String,
}

pub async fn server(address: &str, inner: OrderBook) -> Result<Runtime, Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;
    #[cfg(debug_assertions)]
    println!("Starting service.");
    let service = OrderbookAggregatorServer::new(inner);
    let b = rt.spawn(async {
        #[cfg(debug_assertions)]
        println!("Building server.");
        Server::builder().add_service(service).serve("[::1]:54001".parse().unwrap()).await.unwrap();
        #[cfg(debug_assertions)]
        println!("Server builder passed");
    });
    
    Ok(rt)
}