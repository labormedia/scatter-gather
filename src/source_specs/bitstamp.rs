use super::*;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct BitstampDepth {
    timestamp: String,
    microtimestamp: String,
    bids: Vec<String>,
    asks: Vec<String>,
}