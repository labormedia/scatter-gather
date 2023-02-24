// #[derive(Serialize, Deserialize)]
// struct dummy {
//     i: i32
// }
use std::fmt;
pub trait Interceptor: Send + 'static {
    type Input;
    type InterceptorInEvent: fmt::Debug + Send + 'static;
    type InterceptorOutEvent: fmt::Debug + Send + 'static;
    type InterceptorError: fmt::Debug + Send + 'static;

    fn inject_event(&mut self, event: Self::InterceptorInEvent);
}

pub struct ServerConfig<TInterceptor: Interceptor> {
    pub url: String,
    pub prefix: String,
    pub interceptor: TInterceptor,
}