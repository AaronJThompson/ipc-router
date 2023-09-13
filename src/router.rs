
use std::{pin::Pin, io::Error, io::ErrorKind};
use futures::{Future, AsyncRead, AsyncReadExt};

pub struct Handler<P>
    where P : AsyncRead + Unpin {
    func: Box<dyn Fn(&mut P) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync + 'static>
}

impl<P: AsyncRead + Unpin> Handler<P> {
    pub fn new<H, R>(func: H) -> Handler<P>
        where
        R : Future<Output = Result<(), Error>> + Send + 'static,
        H : Fn(&mut P) -> R + Send + Sync + 'static
    {
        Handler {
            func: Box::new(move |stream| Box::pin(func(stream)))
        }
    }
}

pub struct Router<P>
    where P : AsyncRead + Unpin {
    routes: Vec<Handler<P>>
}

impl<P: AsyncRead + Unpin> Router<P> {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn get(&self, index: usize) -> &Handler<P> {
        &self.routes[index]
    }

    pub fn add<H, R>(&mut self, func: H) -> &Router<P>
    where 
        R : Future<Output = Result<(), Error>> + Send + 'static,
        H: Fn(&mut P) -> R + Send + Sync + 'static
    {
        self.routes.push(Handler::new(func));
        self
    }

    pub async fn route(&self, sock: &mut P) -> Result<(), Error> {
        let mut single_buffer: [u8; 1] = [0; 1];
        let res = sock.read_exact(&mut single_buffer).await;
        match res {
         Ok(()) => {
                let route_idx = single_buffer[0] as usize;
                if route_idx >= self.routes.len() {
                    return Err(Error::new(ErrorKind::InvalidData, "Route out of range"));
                }
                let route = self.get(route_idx);
                (*route.func)(sock).await?;
                Ok(())
            }
            Err(err) => match err.kind() {
                ErrorKind::WouldBlock => Ok(()),
                _ => Err(err)
            }
        }
    }

}