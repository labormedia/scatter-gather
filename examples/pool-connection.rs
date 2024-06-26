use scatter_gather_core::{
    middleware_interface::{
        NodeConfig,
        GetStream,
    },
    pool::{
        Pool,
        PoolConfig,
        PoolConnectionLimits, 
    },
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
    let connection2 = WebSocketsMiddleware::try_new(config_bitstamp);

    let pool_config1 = PoolConfig {
        task_event_buffer_size: 1
    };
    let pool_config2 = PoolConfig {
        task_event_buffer_size: 1
    };

    let mut bitstamp_pool: Pool<WebSocketsMiddleware,Result<Message, tungstenite::Error>, usize> = Pool::new(0_usize, pool_config1, PoolConnectionLimits::default());
    let mut binance_pool: Pool<WebSocketsMiddleware,Result<Message, tungstenite::Error>, usize> = Pool::new(0_usize, pool_config2, PoolConnectionLimits::default());

    bitstamp_pool.collect_streams(Box::pin(connection2.await?.get_stream()));
    binance_pool.collect_streams(Box::pin(connection1.await?.get_stream()));

    loop {
        match bitstamp_pool.next().await {
            None => { },
            Some(Ok(Message::Text(a))) => println!("Accesing Bitstamp: {:?}", a),
            _ => {}
        }
        match binance_pool.next().await {
            None => { },
            Some(Ok(Message::Text(a))) => println!("Accesing Binance: {:?}", a),
            _ => {}
        }
    };
    Ok(())
}