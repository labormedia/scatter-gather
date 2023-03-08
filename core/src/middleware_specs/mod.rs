use futures::{
    Stream,
    stream::{
        SplitSink,
    }
};

// #[derive(Serialize, Deserialize)]
// struct dummy {
//     i: i32
// }
use super::connection::ConnectionHandler;
pub trait Interceptor: Send {
    type Input;
    type Output;

    fn helper(input: Self::Input) -> Self::Output;
    fn intercept(input: Self::Input) -> Self::Output;
}

#[derive(Debug)]
pub struct ServerConfig {
    pub url: String,
    pub prefix: String,
    pub init_handle: Option<String>,
    // pub interceptor: T
    // pub handler: THandler
    // pub handler: &'a THandler,
}