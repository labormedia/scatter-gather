use scatter_gather_core::{
    middleware_specs::{
        ServerConfig, 
        Interceptor,
    }
};
use scatter_gather_websockets::WebSocketsMiddleware;
use scatter_gather::source_specs::{
    Depth,
    bitstamp::BitstampDepthInterceptor as interceptor
};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    let binance_interceptor = interceptor::new();

    let config: ServerConfig<interceptor> = ServerConfig {
        url : String::from("wss://ws.bitstamp.net"),
        prefix: String::from(""),
        init_handle: Some(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#.to_string()),
        interceptor: binance_interceptor
    };
    let mut connection = WebSocketsMiddleware::new(config).await;
    connection.send(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#.to_string()).await;
    while let Some(a) = connection.read.next().await {
        let data: interceptor = connection.config.interceptor.intercept(a?.into_text()?);
        println!("Parsed: {:?}", data);
        println!("Bids: {:?}", data.get_bids());
        println!("Asks: {:?}", data.get_asks());
    }
    Ok(())
}