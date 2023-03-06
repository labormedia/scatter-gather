use scatter_gather_core::{
    middleware_specs::{
        ServerConfig,
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
    binance::BinanceDepthInterceptor,
    bitstamp::BitstampDepthInterceptor,
};
use tungstenite::Message;
use std::{any::type_name};

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

    let connection1 = WebSocketsMiddleware::new(config_binance);
    let connection2 = WebSocketsMiddleware::new(config_bitstamp);

    let pool_config1 = PoolConfig {
        task_event_buffer_size: 1
    };
    // let pool_config2 = PoolConfig {
    //     task_event_buffer_size: 1
    // };
    // type Message = (Option<Result<tungstenite::protocol::message::Message, tungstenite::error::Error>>, tokio_tungstenite::WebSocketStream<tokio_tungstenite::stream::MaybeTlsStream<tokio::net::TcpStream>>);
    // let mut new_pool: Pool<WebSocketsMiddleware<_> = Pool::new(0, pool_config, PoolConnectionLimits::default());

    let mut desired_pool: Pool<WebSocketsMiddleware<BitstampDepthInterceptor>,Result<Message, tungstenite::Error>> = Pool::new(0_usize, pool_config1, PoolConnectionLimits::default());
    // let mut binance_pool: Pool<WebSocketsMiddleware<BinanceDepthInterceptor>,Result<Message, tungstenite::Error>> = Pool::new(0_usize, pool_config2, PoolConnectionLimits::default());

    // new_pool.inject_connection(connection2);
    desired_pool.collect_streams(Box::pin(connection2.await.get_stream()));
    desired_pool.collect_streams(Box::pin(connection1.await.get_stream()));
    // new_pool.intercept_stream().await;

    loop {
        match desired_pool.next().await {
            None => { },
            Some(Ok(Message::Text(a))) => println!("Accesing Pool: {:?}", a),
            _ => { return Ok(())}
        }
        // match binance_pool.next().await {
        //     None => { },
        //     Some(Ok(Message::Text(a))) => println!("Accesing Binance: {:?}", a),
        //     _ => {}
        // }
    };
    
}