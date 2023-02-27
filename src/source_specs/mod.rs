use scatter_gather_core::middleware_specs::Interceptor;
pub mod binance;
pub mod bitstamp;
use serde::{
    Serialize,
    Deserialize,
};
pub trait Depth<T> {
    fn helper(&self, input: String) -> Self;
    fn get_bids(&self) -> &Vec<T>;
    fn get_asks(&self) -> &Vec<T>;
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