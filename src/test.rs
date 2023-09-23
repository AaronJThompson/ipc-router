#[cfg(test)]
mod tests {
    use std::io::ErrorKind;
    use std::time::Duration;
    use std::{sync::Arc, io::Error};

    use futures::{AsyncReadExt, AsyncWriteExt};
    use futures::io::BufWriter;
    use interprocess::local_socket::tokio::{LocalSocketListener, LocalSocketStream};
    use interprocess::local_socket::NameTypeSupport;

    use crate::router;
    router!(MainRouter, LocalSocketStream, test_handler);

    pub const fn get_pipe_name() -> &'static str {
        use NameTypeSupport::*;
        match NameTypeSupport::ALWAYS_AVAILABLE {
            OnlyPaths => "./ipc_router.sock",
            OnlyNamespaced | Both => "@ipc_router.sock",
        }
    }

    #[tokio::test]
    async fn can_route_socket() {
        let res = route_socket().await;
        assert!(res.is_err())
    }

    async fn route_socket() -> Result<(), Error> {
        let listener = match LocalSocketListener::bind(get_pipe_name()) {
			Ok(listener) => listener,
			Err(e) => {
				panic!("{:?}", e);
			}
		};
        let router = Arc::new(MainRouter{});
        loop {
            tokio::spawn(async move  {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                write_string("Hello, world!".to_string()).await;
            });
            match listener.accept().await {
                Ok(mut stream) => {
                    let local_router = router.clone();
                    tokio::spawn(async move {
                        loop {
                            async fn loop_impl(router: &MainRouter, stream: &mut LocalSocketStream) -> Result<(), Error> {
                                let mut single_buffer: [u8; 1] = [0; 1];
                                let res = stream.read_exact(&mut single_buffer).await;
                                match res {
                                Ok(()) => {
                                        let route_idx = single_buffer[0] as usize;
                                        if route_idx >= MainRouter::routes.len() {
                                            return Err(Error::new(ErrorKind::InvalidData, "Route out of range"));
                                        }
                                        router.route(route_idx, stream).await?;
                                        Ok(())
                                    }
                                    Err(err) => match err.kind() {
                                        ErrorKind::WouldBlock => Ok(()),
                                        _ => Err(err)
                                    }
                                }
                            }

                            match loop_impl(local_router.as_ref(), &mut stream).await {
                                //TODO: Handle route error gracefully
                                Err(_) => break,
                                _ => {},
                            }
                            
                        }
                    });
                }
                Err(e) => return Err(e)
            }
        }
    }

    async fn test_handler(stream: &mut LocalSocketStream) -> Result<(), Error> {
        let mut read_buf = String::new();
        let bytes = stream.read_to_string(&mut read_buf).await?;
        println!("Bytes: {}, Output: {}", bytes, read_buf);
        Ok(())
    }

    async fn write_string(str: String) -> Result<(), Error> {
        let stream: LocalSocketStream = match LocalSocketStream::connect(get_pipe_name()).await {
			Ok(stream) => stream,
			Err(e) => {
				panic!("{:?}", e);
			}
		};
        let mut writer = BufWriter::new(stream);
        let to_write = [[0 as u8].as_mut(), str.as_bytes()].concat();
        writer.write_all(to_write.as_slice()).await?;
        Ok(())
    }
}