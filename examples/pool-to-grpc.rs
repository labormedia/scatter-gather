use futures::StreamExt;
use scatter_gather_core::{
    middleware_specs::{
        ServerConfig, Interceptor,
    },
    pool::{
        Pool,
        PoolConfig,
        PoolConnectionLimits, 
    },
};
use scatter_gather_websockets::WebSocketsMiddleware;
use scatter_gather_grpc::{
    schema_specific::{
        self, 
        OrderBook, 
        orderbook::{
            Summary, 
            Level
        }
    }
};
use scatter_gather::source_specs::{
    Interceptors,
    binance::BinanceDepthInterceptor,
    bitstamp::BitstampDepthInterceptor, self,
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

    let config_binance: ServerConfig<BinanceDepthInterceptor> = ServerConfig {
        url : String::from("wss://stream.binance.com:9443/ws/ethbtc@depth@100ms"),
        prefix: String::from("wss://"),
        init_handle: None,
        handler: binance_interceptor
    };
    let config_bitstamp: ServerConfig<BitstampDepthInterceptor> = ServerConfig { 
        url: String::from("wss://ws.bitstamp.net"), 
        prefix: String::from("wss://"), 
        init_handle: Some(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#.to_string()),
        handler: bitstamp_interceptor
    };

    let mut binance = WebSocketsMiddleware::new(config_binance).await;
    let mut bitstamp = WebSocketsMiddleware::new(config_bitstamp).await;

    let binance_intercepted = 
        binance
            .read
            .map(|result| result.unwrap().into_text().unwrap())
            .map(|text| Interceptors::Binance(BinanceDepthInterceptor::intercept(text)) );
    let bitstamp_intercepted =
        bitstamp
            .read
            .map(|result| result.unwrap().into_text().unwrap())
            .map(|text| Interceptors::Bitstamp(BitstampDepthInterceptor::intercept(text)) );

    let pool_config1 = PoolConfig {
        task_event_buffer_size: 1
    };
    let mut desired_pool: Pool<WebSocketsMiddleware<Interceptors>,Interceptors> = Pool::new(0_usize, pool_config1, PoolConnectionLimits::default());

    desired_pool.collect_streams(Box::pin(binance_intercepted));
    desired_pool.collect_streams(Box::pin(bitstamp_intercepted));

    loop {
        match desired_pool.next().await {
            None => { },
            Some(a) => println!("Accesing Pool: {:?}", a),
        }
    };
    Ok(())
    
}