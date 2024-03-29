use scatter_gather_core::{
    middleware_interface::{
        NodeConfig, 
        Interceptor,
    },
};
use scatter_gather_websockets::WebSocketsMiddleware;
mod source_specs;
use source_specs::{
    Depth,
    bitstamp::BitstampDepthInterceptor,
};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    let config: NodeConfig = NodeConfig {
        url : String::from("wss://ws.bitstamp.net"),
        prefix: String::from(""),
        init_handle: Some(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#.to_string()),
    };
    let mut connection = WebSocketsMiddleware::try_new(config).await?;
    connection.send(r#"{"event": "bts:subscribe","data":{"channel": "diff_order_book_ethbtc"}}"#.to_string()).await?;
    let mut new_stream = connection
        .read
        .map(|result| result.unwrap().into_text().unwrap())
        .map(|msg| BitstampDepthInterceptor::intercept(msg));
    while let Some(data) = new_stream.next().await {
        println!("Summary: {:?}", data.level());
    }
    Ok(())
}