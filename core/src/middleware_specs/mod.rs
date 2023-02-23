// #[derive(Serialize, Deserialize)]
// struct dummy {
//     i: i32
// }
pub trait Interceptor {
    type InterceptorInEvent;
    type InterceptorOutEvent;
    type InterceptorError;

    fn inject_event(&mut self, event: Self::InterceptorInEvent);
}

pub struct ServerConfig<TInterceptor: Interceptor> {
    pub url: String,
    pub prefix: String,
    pub protocol: TInterceptor,
}