use core::task::{
    Context,
    Poll,
};
use scatter_gather_core::{
    Executor,
    middleware_interface::{
        NodeConfig,
        GetStream,
    },
    pool::{
        Pool,
        PoolConfig,
        PoolConnectionLimits,
        PoolConnection,
        EstablishedConnection,

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
    stream::SplitStream,
};
use scatter_gather_websockets::{
    WebSocketsMiddleware,
    WebSocketStream,
    MaybeTlsStream,
};
use scatter_gather_grpc::GrpcMiddleware;
mod source_specs;
use source_specs::{
    binance::BinanceDepthInterceptor,
    bitstamp::BitstampDepthInterceptor,
};
use tungstenite::Message;
use std::{
    sync::mpsc::sync_channel,
    any::type_name,
};

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    type WSPool = Pool<WebSocketsMiddleware,Result<Message, tungstenite::Error>, usize>;

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

    const MAX_QUEUED_TASKS: usize = 10;
    let (executor, receiver) = sync_channel(MAX_QUEUED_TASKS);
    let custom_executor = CustomExecutor::with_sender(executor);

    ws_pool.with_executor(
        Box::new(custom_executor)
    );
    
    match ws_pool.connect().await {
        Poll::Ready(connections) => {
            connections.iter()
            .map(|conn| async {
                let mut conn_lock = ws_pool.get_established_connection(conn).expect("Could not connect.").conn.lock().await;
                conn_lock
            })
            .map( |conn| {
                conn
            });
        },
        _ => {},
    }

    // let Some(executor): &Option<Box<dyn Executor<WebSocketsMiddleware> + std::marker::Send >> = ws_pool.get_executor()
    // else { todo!() };



    // if let (Poll::Ready(e1), Poll::Ready(e2)) = (binance_conn,bitstamp_conn) { // the condition will advance if both connections are established
    //     let mut init_binance_ws = e1.conn.clone();
    //     init_binance_ws.lock().await.init_handle().await?;
    //     let read_binance_ws = e1.conn.clone();
    //     let binance_stream = &mut read_binance_ws.lock().await.read;

    //     let mut init_bitstamp_ws = e2.conn.clone();
    //     init_bitstamp_ws.lock().await.init_handle().await?;
    //     let read_bitstamp_ws = e2.conn.clone();
    //     let bitstamp_stream = &mut read_bitstamp_ws.lock().await.read;

    //     loop {
    //         println!("Incoming Bitstamp{:?}", bitstamp_stream.next().await);
    //         println!("Incoming Binance{:?}", binance_stream.next().await);
    //     }
    // }

    

    Ok(())
}