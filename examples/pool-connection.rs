use scatter_gather_core::{
    middleware_specs::{
        ServerConfig,
        Interceptor
    },
    pool::{
        Pool,
        PoolConfig,
        PoolConnectionLimits
    }
};
use scatter_gather_websockets::WebSocketsMiddleware;
use scatter_gather::source_specs::{
    binance::BinanceDepthInterceptor,
    bitstamp::BitstampDepthInterceptor
};
use futures::{StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    let binance_interceptor = BinanceDepthInterceptor::new();
    let bitstamp_interceptor = BitstampDepthInterceptor::new();

    let config_binance: ServerConfig<BinanceDepthInterceptor> = ServerConfig {
        url : String::from("wss://stream.binance.com:9443/ws/ethbtc@depth@100ms"),
        prefix: String::from("wss://"),
        init_handle: None,
        interceptor: binance_interceptor
    };
    let config_bitstamp: ServerConfig<BitstampDepthInterceptor> = ServerConfig { 
        url: String::from("wss://ws.bitstamp.net"), 
        prefix: String::from("wss://"), 
        init_handle: Some(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#.to_string()),
        interceptor: bitstamp_interceptor
    };
    
    let connection1 = WebSocketsMiddleware::new(config_binance);
    let connection2 = WebSocketsMiddleware::new(config_bitstamp);

    let pool_config = PoolConfig {
        task_event_buffer_size: 1
    };
    // type Message = (Option<Result<tungstenite::protocol::message::Message, tungstenite::error::Error>>, tokio_tungstenite::WebSocketStream<tokio_tungstenite::stream::MaybeTlsStream<tokio::net::TcpStream>>);
    let mut new_pool: Pool<_, BinanceDepthInterceptor, tungstenite::error::Error> = Pool::new(0, pool_config, PoolConnectionLimits::default());

    new_pool.collect_streams(Box::pin(connection1.await.read));
    new_pool.collect_streams(Box::pin(connection2.await.read));
    while let Some(a) = new_pool.local_streams.next().await {
        println!("test {:?}", a);
    }
    Ok(())
}