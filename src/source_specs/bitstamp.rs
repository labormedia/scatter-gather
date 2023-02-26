use super::*;

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct BitstampDepth {
    timestamp: String,
    microtimestamp: String,
    bids: Vec<String>,
    asks: Vec<String>,
}