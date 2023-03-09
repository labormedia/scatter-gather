use super::{
    Depth,
    Interceptor,
    Level
};
use scatter_gather_core::connection::{ConnectionHandler, self, ConnectionHandlerOutEvent, ConnectionHandlerInEvent};
use serde_json;
use serde::{
    Serialize,
    Deserialize,
};
use futures::Future;
use tungstenite::Message;
use std::task::Poll;

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct BitstampDepthInterceptor {
    event: String,
    channel: String,
    data: Data
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Data {
    timestamp: String,
    microtimestamp: String,
    bids: Vec<Level>,
    asks: Vec<Level>,
}

#[derive(Debug)]
enum CustomDepthInEvent {
    Message(String),
    Error(Box<dyn std::error::Error + Send>)
}

impl BitstampDepthInterceptor {
    pub fn new() -> Self {
        Self::default()
    }
    
}

fn interceptor_to_pb(data: BitstampDepthInterceptor) -> (String, Vec<Level>, Vec<Level>) {
    let mut a = Vec::new();
    a.clone_from_slice(&data.data.asks);
    let mut b = Vec::new();
    b.clone_from_slice(&data.data.bids);
    let exchange = data.exchange().clone();

    (exchange, b, a)
}

impl Depth<Level> for BitstampDepthInterceptor {
    fn level(self) -> (String, Vec<Level>, Vec<Level>) {
        let mut a = Vec::new();
        let _ = a.extend_from_slice(&self.data.asks);
        let mut b = Vec::new();
        let _ = b.extend_from_slice(&self.data.bids);

        (self.exchange(), b, a)
    }
    fn exchange(self) -> String { String::from("Bitstamp") }
    fn get_bids(self) -> Vec<Level> {
        self.data.bids
    }

    fn get_asks(self) -> Vec<Level> {
        self.data.asks
    }
}

impl Interceptor for BitstampDepthInterceptor {
    type Input = String;
    type Output = BitstampDepthInterceptor;
    fn helper(input: String) -> Self {
        // #[cfg(debug_assertions)]
        // println!("Input: {:?}", input);
        match serde_json::from_str(&input){
            Ok(a) => {
                // #[cfg(debug_assertions)]
                // println!("Input: {:?}", a);
                a
            },
            Err(e) => {
                #[cfg(debug_assertions)]
                println!("Dropping failed parsing: {:?}", e);
                Self::default()
            }
        }
    }
    fn intercept(input: Self::Input) -> BitstampDepthInterceptor {
        let a = Self::helper(input);
        a
    }
}

impl ConnectionHandler<'_> for BitstampDepthInterceptor {
    type InEvent = connection::ConnectionHandlerInEvent;
    type OutEvent = connection::ConnectionHandlerOutEvent<Message>;

    fn poll(
        mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::OutEvent>
    {
        // Poll::Ready(connection::ConnectionHandlerOutEvent::ConnectionClosed(Message::Text("hello".to_string())))
        Poll::Pending
    }
    fn inject_event(&mut self, event: Self::InEvent) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn eject_event(&mut self, event: Self::OutEvent) -> Result<(), tokio::sync::mpsc::error::SendError<Self::OutEvent>> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
}

impl Future for BitstampDepthInterceptor {
    type Output = &'static dyn Depth<Level>;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.inject_event(ConnectionHandlerInEvent::Connect);
        Poll::Pending
    }
}