#![feature(macro_metavar_expr)]
#[cfg(test)]
mod test;
mod router;

use std::io::Error;

use async_trait::async_trait;
use futures::AsyncRead;
use interprocess::local_socket::tokio::LocalSocketStream;


async fn test(stream:&mut LocalSocketStream) -> Result<(), Error> {
    Ok(())
}

async fn test2(steam: &mut LocalSocketStream) -> Result<(), Error> {
    Ok(())
}
router!(LibRouter, LocalSocketStream, test, test2);