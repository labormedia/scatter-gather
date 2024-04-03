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

impl NodeConfig {
    pub fn from<T: ToString>(
        url: T,
        prefix: T,
        init_handle: Option<T>,
    ) -> Self {
        NodeConfig {
            url: url.to_string(),
            prefix: prefix.to_string(),
            init_handle: match init_handle {
                Some(handle) => {
                    Some(handle.to_string())
                },
                None => { None },
            }
        }
    }
}