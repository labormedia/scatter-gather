#[cfg(test)]
mod tests {
    use scatter_gather_grpc::schema_specific::orderbook::{
        self,
        orderbook_aggregator_client::OrderbookAggregatorClient,
    };
    use std::time::Instant;
    const ADDRESS: &str = "http://[::1]:54001";
    #[tokio::test]
    async fn throughput() -> Result<(), Box<dyn std::error::Error + 'static>> {
        let mut channel = OrderbookAggregatorClient::connect(ADDRESS)
            .await?;
        let request = tonic::Request::new( orderbook::Empty {});
        let mut stream = channel.book_summary(request).await?.into_inner();
        let now = Instant::now();
        const N: usize = 100;
        let mut buffer= [0_f64;N];
        let mut i = 0;
        while let Ok(Some(_a)) =  stream.message().await {
            if i == N { break };
            buffer[i] = now.elapsed().as_millis() as f64;
            i +=1;
        };
        println!("Buffer: {:?}", buffer );
        Ok(())
    }
}

fn main() {
    
}