use super::*;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct InData {
    timestamp: String,
    microtimestamp: String,
    bids: Vec<String>,
    asks: Vec<String>,
}