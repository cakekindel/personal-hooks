use async_trait::async_trait;
use thiserror::Error as DeriveError;

#[derive(Debug)]
pub struct Pushbullet {
  token: String,
}

#[derive(Debug)]
pub struct Email;

#[derive(Debug)]
pub struct Slack;

#[async_trait]
pub trait Notifier {
  async fn notify(&self, msg: &str) -> Result<(), Error>;
}

#[derive(Debug, DeriveError)]
pub enum Error {}
