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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    struct BinanceInterceptor {}

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

    impl Interceptor for BinanceInterceptor {
        type Input = BinanceDepth;

        type InterceptorError = ();
        type InterceptorInEvent = CustomBinanceInEvent;
        type InterceptorOutEvent = ();

        fn inject_event(&mut self, event: Self::InterceptorInEvent) {
            ()
        }
    }

    let binance_interceptor = BinanceInterceptor::new();

    let config: ServerConfig<BinanceInterceptor> = ServerConfig {
        url : String::from("wss://stream.binance.com:9443/ws/bnbbtc@depth@100ms"),
        prefix: String::from(""),
        interceptor: binance_interceptor
    };
    let mut ws_stream = WebSocketsMiddleware::new(config).connect().await;

    let pool_config = PoolConfig {
        task_event_buffer_size: 1
    };
    // type Message = (Option<Result<tungstenite::protocol::message::Message, tungstenite::error::Error>>, tokio_tungstenite::WebSocketStream<tokio_tungstenite::stream::MaybeTlsStream<tokio::net::TcpStream>>);
    let mut new_pool: Pool<_, BinanceInterceptor, <BinanceInterceptor as Interceptor>::InterceptorError> = Pool::new(0, pool_config, ConnectionLimits::default());

    new_pool.collect_streams(Box::pin(ws_stream));
    while let Some(a) = new_pool.local_streams.next().await {
        println!("test {:?}", a);
    }
    Ok(())
}