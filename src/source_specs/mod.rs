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
use futures::{
    task::Poll
};
use std::fmt::Debug;

use self::{binance::BinanceDepthInterceptor, bitstamp::BitstampDepthInterceptor};

pub trait Depth<T>: Send + Sync {
    fn get_bids(&self) -> &Vec<T>;
    fn get_asks(&self) -> &Vec<T>;
}

impl<T: Send + 'static + Default> Debug for Box<dyn Depth<T>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Depth: {:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Level {
    #[serde(deserialize_with = "helpers::quantity_from_str")]
    left: f64,
    #[serde(deserialize_with = "helpers::quantity_from_str")]
    right: f64
}

pub mod helpers {
    use serde::{
        Deserialize,
        Deserializer,
        de
    };
    pub fn quantity_from_str<'a, D>(input: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'a>,
    {
        let str_val = String::deserialize(input)?;
        str_val.parse::<f64>().map_err(de::Error::custom)
    }
    pub fn check_json<'a, D, T>(input: D, source_type: T) -> Result<String, D::Error>
    where
        D: Deserializer<'a>
    {
        let str_val = String::deserialize(input)?;
        str_val.parse::<String>().map_err(de::Error::custom)
    }
    pub fn parse<'a, D, T>(input: &str )
    where
        D: Deserializer<'a>
    {
        let _: f64 = match serde_json::from_str(input){
            Ok(a) => {
                #[cfg(debug_assertions)]
                println!("Input: {:?}", a);
                a
            },
            Err(e) => {
                #[cfg(debug_assertions)]
                println!("Dropping failed parsing: {:?}", e);
                0.0
            }
        };
    }
}

#[derive(Debug)]
pub enum Interceptors {
    Binance(BinanceDepthInterceptor),
    Bitstamp(BitstampDepthInterceptor),
    Depth
}

impl ConnectionHandler<'_> for Interceptors {
    type InEvent = String;
    type OutEvent = Self;

    fn poll(
        self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::OutEvent> 
    {
        // Poll::Ready(connection::ConnectionHandlerOutEvent::ConnectionClosed(Message::Text("hello".to_string())))
        Poll::Pending
    }
    fn inject_event(&mut self, event: Self::InEvent) -> Result<(), Box<dyn std::error::Error>> {
        println!("Hello Future! InEvent: {:?}", event);
        Ok(())
    }
    fn eject_event(&mut self, event: Self::OutEvent) -> Result<(), tokio::sync::mpsc::error::SendError<Self::OutEvent>> {
        println!("Hello Future! OutEvent: {:?}", event);
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
}

impl Interceptor for Interceptors {
    type Input = Self;
    type Output = Self;

    fn helper(input: Self::Input) -> Self::Output {
        match input {
            Self::Binance(binance) => Self::Binance(binance),
            Self::Bitstamp(bitstamp) => Self::Bitstamp(bitstamp),
            Self::Depth => Self::Depth
        }
    }

    fn intercept(input: Self::Input) -> Self::Output {
        Self::helper(input)
    }
}