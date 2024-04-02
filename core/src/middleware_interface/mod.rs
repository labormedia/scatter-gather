pub trait Interceptor: Send {
    type Input;
    type Output;

    fn helper(input: Self::Input) -> Self::Output;
    fn intercept(input: Self::Input) -> Self::Output;
}

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub url: String,
    pub prefix: String,
    pub init_handle: Option<String>,
    // pub interceptor: T
    // pub handler: THandler
    // pub handler: &'a THandler,
}