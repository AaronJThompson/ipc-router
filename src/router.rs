#[macro_export]
macro_rules! router_func {
    ($payload_type: ty, $($n:ident),*) => {
        pub async fn route(&self, route: usize, payload: &mut $payload_type) -> Result<(), Error> {
            match route {
                $(
                    ${index()} => $n(payload).await,
                )*
                _ => panic!()
            }
        } 
    }
}
#[macro_export]
macro_rules! router {
    ($router: ident, $payload_type: ty, $($n:ident),*) => {
        struct $router();

        impl $router {
            pub const routes: [usize; ${count(n, 0)}] = [$(${ignore(n)} ${index()}),*];
            crate::router_func!($payload_type, $($n),*);
            pub fn new() -> $router {
                $router{}
            }
        }
    };
}
