// #[derive(Serialize, Deserialize)]
// struct dummy {
//     i: i32
// }

pub trait Interceptor: Send + 'static {
    type Input;
    type Output;

    fn intercept(&mut self, input: Self::Input) -> Self::Output;
}

pub struct ServerConfig<TInterceptor: Interceptor> {
    pub url: String,
    pub prefix: String,
    pub interceptor: TInterceptor,
}