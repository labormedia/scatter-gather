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

    fn intercept(&mut self, input: Self::Input) -> Self::Output;
}

#[derive(Debug)]
pub struct ServerConfig<THandler: for <'a> ConnectionHandler<'a> > {
    pub url: String,
    pub prefix: String,
    pub init_handle: Option<String>,
    pub handler: THandler
    // pub handler: &'a THandler,
}