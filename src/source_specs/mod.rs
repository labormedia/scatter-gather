use scatter_gather_core::{
    middleware_specs::Interceptor, 
    connection::{
        ConnectionHandler,
        ConnectionHandlerInEvent,
        ConnectionHandlerOutEvent
    }
};
pub mod binance;
pub mod bitstamp;
use serde::{
    Serialize,
    Deserialize,
};
use tungstenite::Message;
use std::task::Poll;

pub trait Depth<T>: Send + Sync {
    // fn helper(&self, input: String) -> Self;
    fn get_bids(&self) -> &Vec<T>;
    fn get_asks(&self) -> &Vec<T>;
}

impl<T: Send + 'static> ConnectionHandler for &'static dyn Depth<T> {
    type InEvent = ConnectionHandlerInEvent;
    type OutEvent = ConnectionHandlerOutEvent<Message>;

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::OutEvent> 
    {
        Poll::Ready(ConnectionHandlerOutEvent::ConnectionClosed(Message::Text("Hello".to_string())))
    }
    fn inject_event(&mut self, event: Self::InEvent) {
        
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Level {
    #[serde(deserialize_with = "helpers::quantity_from_str")]
    left: f32,
    #[serde(deserialize_with = "helpers::quantity_from_str")]
    right: f32
}

pub mod helpers {
    use serde::{
        Deserialize,
        Deserializer,
        de
    };
    pub fn quantity_from_str<'a, D>(input: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'a>,
    {
        let str_val = String::deserialize(input)?;
        str_val.parse::<f32>().map_err(de::Error::custom)
    }
    pub fn temporal_value<'a, D, T>(input: D, source_type: T) -> Result<String, D::Error>
    where
        D: Deserializer<'a>
    {
        let str_val = String::deserialize(input)?;
        str_val.parse::<String>().map_err(de::Error::custom)
    }
}