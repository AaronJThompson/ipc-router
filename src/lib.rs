mod router;
#[cfg(test)]
mod test;

use std::{io::Error, sync::Arc};

use futures::AsyncRead;
use async_trait::async_trait;
use interprocess::local_socket::tokio::{LocalSocketStream, LocalSocketListener};

use crate::router::Router;

#[async_trait]
pub trait IPCListener<P>
    where P: AsyncRead + Unpin {
    async fn setup(&mut self) -> Result<(), Error>;
    async fn run(&mut self, router: &mut Arc<Router<P>>) -> Result<(), Error>;
}

pub struct IPCRouter<L: IPCListener<P>, P: AsyncRead + Unpin> {
    pub router: Arc<Router<P>>,
    pub listener: L
}

impl<L: IPCListener<P>, P: AsyncRead + Unpin> IPCRouter<L, P> {
    pub async fn run(&mut self) -> Result<(), Error> {
        match self.listener.setup().await {
            Ok(()) => match self.listener.run(&mut self.router).await {
                Ok(()) => Ok(()),
                Err(e) => Err(e)
            }
            Err(e) => Err(e)
        }
    }
}

#[async_trait]
impl IPCListener<LocalSocketStream> for LocalSocketListener {
    async fn setup(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn run(&mut self, router: &mut Arc<Router<LocalSocketStream>>) -> Result<(), Error> {
        loop {
            match self.accept().await {
                Ok(mut stream) => {
                    let local_router = router.clone();
                    tokio::spawn(async move {
                        loop {
                            //TODO: Handle route error gracefully
                            local_router.route(&mut stream).await.unwrap();
                        }
                    });
                }
                Err(e) => return Err(e)
            }
        }
    }
}