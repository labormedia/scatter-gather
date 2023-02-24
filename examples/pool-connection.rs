use scatter_gather_core::{
    middleware_specs::{
        ServerConfig,
        Interceptor
    },
    Pool,
    PoolConfig,
    ConnectionLimits
};
use scatter_gather_websockets::WebSocketsMiddleware;
use scatter_gather::source_specs::binance::BinanceDepth;
use futures::{StreamExt, FutureExt};
use tungstenite::Message;
use futures::SinkExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    struct BinanceInterceptor {}
    struct BitstampInterceptor {}

    #[derive(Debug)]
    enum CustomBinanceInEvent {
        Message(String),
        Error(Box<dyn std::error::Error + Send>)
    }

    impl BinanceInterceptor {
        fn new() -> Self {
            Self {}
        }
    }

    impl BitstampInterceptor {
        fn new() -> Self {
            Self {}
        }
    }

    impl Interceptor for BinanceInterceptor {
        type Input = BinanceDepth;

        type InterceptorError = ();
        type InterceptorInEvent = CustomBinanceInEvent;
        type InterceptorOutEvent = ();

        fn inject_event(&mut self, event: Self::InterceptorInEvent) {
            ()
        }
    }

    impl Interceptor for BitstampInterceptor {
        type Input = BinanceDepth;

        type InterceptorError = ();
        type InterceptorInEvent = CustomBinanceInEvent;
        type InterceptorOutEvent = ();

        fn inject_event(&mut self, event: Self::InterceptorInEvent) {
            ()
        }
    }

    let binance_interceptor = BinanceInterceptor::new();
    let bitstamp_interceptor = BitstampInterceptor::new();

    let config_binance: ServerConfig<BinanceInterceptor> = ServerConfig {
        url : String::from("wss://stream.binance.com:9443/ws/ethbtc@depth@100ms"),
        prefix: String::from("wss://"),
        interceptor: binance_interceptor
    };
    let config_bitstamp: ServerConfig<BitstampInterceptor> = ServerConfig { 
        url: String::from("wss://ws.bitstamp.net"), 
        prefix: String::from("wss://"), 
        interceptor: bitstamp_interceptor
    };
    let connection1 = WebSocketsMiddleware::new(config_binance).await;
    let mut connection2 = WebSocketsMiddleware::new(config_bitstamp).await;

    connection2.send(r#"{"event": "bts:subscribe","data":{"channel": "live_orders_ethbtc"}}"#.to_string()).await;

    let pool_config = PoolConfig {
        task_event_buffer_size: 1
    };
    // type Message = (Option<Result<tungstenite::protocol::message::Message, tungstenite::error::Error>>, tokio_tungstenite::WebSocketStream<tokio_tungstenite::stream::MaybeTlsStream<tokio::net::TcpStream>>);
    let mut new_pool: Pool<_, BinanceInterceptor, <BinanceInterceptor as Interceptor>::InterceptorError> = Pool::new(0, pool_config, ConnectionLimits::default());

    new_pool.collect_streams(Box::pin(connection1.read));
    new_pool.collect_streams(Box::pin(connection2.read));
    while let Some(a) = new_pool.local_streams.next().await {
        println!("test {:?}", a);
    }
    Ok(())
}