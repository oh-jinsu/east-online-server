use east_online_core::model::Vector3;
use futures::future::select_all;
use std::{collections::HashMap, error::Error};
use tokio::net::TcpStream;

#[async_trait::async_trait]
pub trait Waitings<T> {
    async fn wait_for_readable(&self) -> Result<T, Box<dyn Error>>;
}

#[async_trait::async_trait]
impl Waitings<usize> for Vec<TcpStream> {
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

#[async_trait::async_trait]
impl Waitings<String> for HashMap<String, (TcpStream, Vector3)> {
    async fn wait_for_readable(&self) -> Result<String, Box<dyn Error>> {
        if self.is_empty() {
            return Err("no waitings".into());
        }

        match select_all(self.iter().map(|(key, (stream, _))| {
            Box::pin(async move {
                stream.readable().await?;

                Ok::<&str, Box<dyn Error>>(key)
            })
        }))
        .await
        {
            (Ok(key), _, _) => Ok(key.to_string()),
            (Err(e), _, _) => Err(e),
        }
    }
}
