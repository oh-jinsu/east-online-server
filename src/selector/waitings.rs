use futures::future::select_all;
use std::error::Error;
use tokio::net::TcpStream;

#[async_trait::async_trait]
pub trait Waitings {
    async fn wait_for_readable(&self) -> Result<usize, Box<dyn Error>>;
}

#[async_trait::async_trait]
impl Waitings for Vec<TcpStream> {
    async fn wait_for_readable(&self) -> Result<usize, Box<dyn Error>> {
        if self.is_empty() {
            return Err("no waitings".into());
        }

        match select_all(self.iter().enumerate().map(|(index, stream)| {
            Box::pin(async move {
                stream.readable().await?;

                Ok::<usize, Box<dyn Error>>(index)
            })
        }))
        .await
        {
            (Ok(index), _, _) => Ok(index),
            (Err(e), _, _) => Err(e),
        }
    }
}
