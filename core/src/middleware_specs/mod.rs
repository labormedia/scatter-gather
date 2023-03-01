// #[derive(Serialize, Deserialize)]
// struct dummy {
//     i: i32
// }
use super::connection::ConnectionHandler;
pub trait Interceptor: Send + 'static {
    type Input;
    type Output;

    fn intercept(&mut self, input: Self::Input) -> Self::Output;
}

#[derive(Debug)]
pub struct ServerConfig<THandler: ConnectionHandler> {
    pub url: String,
    pub prefix: String,
    pub init_handle: Option<String>,
    pub handler: THandler,
}