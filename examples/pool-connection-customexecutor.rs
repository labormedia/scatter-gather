use core::task::{
    Context,
    Poll,
};
use scatter_gather_core::{
    middleware_interface::{
        NodeConfig,
    },
    pool::{
        Pool,
        PoolConfig,
        PoolConnectionLimits,
    },
    connection::{
        ConnectionId,
        ConnectionHandler,
        ConnectionHandlerOutEvent,
    },
    executors::CustomExecutor,
};
use futures::{
    executor::ThreadPool,
    StreamExt,
};
use scatter_gather_websockets::WebSocketsMiddleware;
use scatter_gather_grpc::GrpcMiddleware;
mod source_specs;
use source_specs::{
    binance::BinanceDepthInterceptor,
    bitstamp::BitstampDepthInterceptor,
};
use tungstenite::Message;
use std::any::type_name;

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    type WSPool = Pool<WebSocketsMiddleware,Result<Message, tungstenite::Error>>;

    let binance_interceptor = BinanceDepthInterceptor::new();
    let bitstamp_interceptor = BitstampDepthInterceptor::new();

    let config_binance: NodeConfig = NodeConfig {
        url : String::from("wss://stream.binance.com:9443/ws/ethbtc@depth@100ms"),
        prefix: String::from("wss://"),
        init_handle: None,
        // handler: binance_interceptor
    };
    let config_bitstamp: NodeConfig = NodeConfig { 
        url: String::from("wss://ws.bitstamp.net"), 
        prefix: String::from("wss://"), 
        init_handle: Some(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#.to_string()),
        // handler: bitstamp_interceptor
    };

    let pool_config1 = PoolConfig {
        task_event_buffer_size: 1
    };
    let pool_config2 = PoolConfig {
        task_event_buffer_size: 1
    };

    let mut bitstamp_pool: WSPool = Pool::new(0_usize, pool_config1, PoolConnectionLimits::default());
    let mut binance_pool: WSPool = Pool::new(0_usize, pool_config2, PoolConnectionLimits::default());

    let mut ws_pool: WSPool = Pool::new(
        0_usize, // Pool ID
        PoolConfig { // Pool General Configuration
            task_event_buffer_size: 1
        }, 
        PoolConnectionLimits::default() // Pool Limits
    );

    let _ = [ // Slice of WebSocket connection configurations.
        NodeConfig::from(
            "wss://stream.binance.com:9443/ws/ethbtc@depth@100ms", // url
            "wss://",  // prefix
            None,  // init handle
        ),
        NodeConfig::from( 
            "wss://ws.bitstamp.net", // url
            "wss://", // prefix
            Some(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#),  // init handle
        ),
    ]
    .map( |config| {
        ws_pool.inject_connection(WebSocketsMiddleware::new(config.clone()));
    } );
    
    ws_pool.connect().await;

    // bitstamp_pool.with_executor(
    //     Box::new(CustomExecutor {
    //         executor: ThreadPool::new()?,
    //     })
    // );
    // binance_pool.with_executor(
    //     Box::new(CustomExecutor {
    //         executor: ThreadPool::new()?,
    //     })
    // );
    
    // bitstamp_pool.inject_connection(WebSocketsMiddleware::new(config_bitstamp));
    // binance_pool.inject_connection(WebSocketsMiddleware::new(config_binance));

    // let bitstamp_conn = bitstamp_pool.connect().await;
    // let binance_conn = binance_pool.connect().await;

    // if let (Poll::Ready(e1), Poll::Ready(e2)) = (binance_conn,bitstamp_conn) { // the condition will advance if both connections are established
    //     let mut init_binance_ws = e1.state.clone();
    //     init_binance_ws.lock().await.init_handle().await?;
    //     let read_binance_ws = e1.state.clone();
    //     let binance_stream = &mut read_binance_ws.lock().await.read;

    //     let mut init_bitstamp_ws = e2.state.clone();
    //     init_bitstamp_ws.lock().await.init_handle().await?;
    //     let read_bitstamp_ws = e2.state.clone();
    //     let bitstamp_stream = &mut read_bitstamp_ws.lock().await.read;

    //     loop {
    //         println!("Incoming Bitstamp{:?}", bitstamp_stream.next().await);
    //         println!("Incoming Binance{:?}", binance_stream.next().await);
    //     }
    // }

    

    Ok(())
}