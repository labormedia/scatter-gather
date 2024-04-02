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

    let connection1 = WebSocketsMiddleware::try_new(config_binance);
    let connection2 = WebSocketsMiddleware::try_new(config_bitstamp.clone());

    let pool_config1 = PoolConfig {
        task_event_buffer_size: 1
    };
    let pool_config2 = PoolConfig {
        task_event_buffer_size: 1
    };

    let mut bitstamp_pool: Pool<WebSocketsMiddleware,Result<Message, tungstenite::Error>> = Pool::new(0_usize, pool_config1, PoolConnectionLimits::default());
    let mut binance_pool: Pool<WebSocketsMiddleware,Result<Message, tungstenite::Error>> = Pool::new(0_usize, pool_config2, PoolConnectionLimits::default());

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
    
    bitstamp_pool.inject_connection(WebSocketsMiddleware::new(config_bitstamp));

    loop {
        match bitstamp_pool.connect().await {
            Poll::Ready(event) => {
                println!("Event {:?}", event);
                let mut init_ws = event.state.clone();
                init_ws.lock().await.init_handle().await?;
                let read_ws = event.state.clone();
                let stream = &mut read_ws.lock().await.read;
                while let e = stream.next().await {
                    println!("Incoming {:?}", e);
                };
            },
            Poll::Pending => {
                println!("Pending.")
            }
        }
    } 
    Ok(())
}