#[cfg(test)]
mod tests {
    use std::{sync::Arc, io::Error};

    use interprocess::local_socket::tokio::{LocalSocketListener, LocalSocketStream};
    use interprocess::local_socket::NameTypeSupport;

    use crate::{IPCRouter, router::Router};

    pub const fn get_pipe_name() -> &'static str {
        use NameTypeSupport::*;
        match NameTypeSupport::ALWAYS_AVAILABLE {
            OnlyPaths => "./ipc_router.sock",
            OnlyNamespaced | Both => "@ipc_router.sock",
        }
    }

    #[tokio::test]
    async fn can_route_socket() {
        let listener = match LocalSocketListener::bind(get_pipe_name()) {
			Ok(listener) => listener,
			Err(e) => {
				panic!("{:?}", e);
			}
		};
        let router = Router::new();
        let ipc_router: IPCRouter<LocalSocketListener, LocalSocketStream> = IPCRouter{ listener: listener, router: Arc::new(router) };
        router.add(test_handler);
    }

    async fn test_handler(stream: &mut LocalSocketStream) -> Result<(), Error> {
        Ok(())
    }
}