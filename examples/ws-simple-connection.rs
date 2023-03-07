use scatter_gather_core::{
    middleware_specs::{
        ServerConfig, 
        Interceptor,
    },
};
use scatter_gather_websockets::WebSocketsMiddleware;
use scatter_gather::source_specs::{
    Depth,
    bitstamp::BitstampDepthInterceptor,
};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    let bitstamp_interceptor = BitstampDepthInterceptor::new();

    let config: ServerConfig<BitstampDepthInterceptor> = ServerConfig {
        url : String::from("wss://ws.bitstamp.net"),
        prefix: String::from(""),
        init_handle: Some(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#.to_string()),
        handler: bitstamp_interceptor
    };
    let mut connection = WebSocketsMiddleware::new(config).await;
    connection.send(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#.to_string()).await;
    let mut new_stream = connection
        .read
        .map(|result| result.unwrap().into_text().unwrap())
        .map(|msg| BitstampDepthInterceptor::intercept(msg));
    while let Some(data) = new_stream.next().await {
        // let data: interceptor = connection.config.handler.intercept(a?.into_text()?);
        println!("Parsed: {:?}", data);
        println!("Exchange: {:?}", data.exchange());
        println!("Bids: {:?}", data.get_bids());
        println!("Asks: {:?}", data.get_asks());
    }
    Ok(())
}