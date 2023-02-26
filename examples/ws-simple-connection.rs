use scatter_gather_core::{
    middleware_specs::{
        ServerConfig, 
        Interceptor
    }
};
use scatter_gather_websockets::WebSocketsMiddleware;
use scatter_gather::source_specs::binance::BinanceDepthInterceptor;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    let binance_interceptor = BinanceDepthInterceptor::new();

    let config: ServerConfig<BinanceDepthInterceptor> = ServerConfig {
        url : String::from("wss://stream.binance.com:9443/ws/bnbbtc@depth@100ms"),
        prefix: String::from(""),
        interceptor: binance_interceptor
    };
    let mut connection = WebSocketsMiddleware::new(config).await;
    while let Some(a) = connection.read.next().await {
        let data = connection.config.interceptor.intercept(a?.into_text()?);
        println!("Parsed: {:?}", data);
    }
    Ok(())
}