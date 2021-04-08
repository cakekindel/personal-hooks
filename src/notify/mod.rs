use async_trait::async_trait;
use thiserror::Error as DeriveError;

pub mod pushbullet;
pub use pushbullet::Pushbullet;

#[derive(Debug)]
pub struct Slack;

#[async_trait]
pub trait Notifier {
  async fn notify(&self,
                  reqw: &reqwest::Client,
                  title: &str,
                  body: &str)
                  -> Result<(), Error>;
}

#[derive(Debug, DeriveError)]
pub enum Error {
  #[error("{0:#?}")]
  Many(Vec<Self>),

  #[error("Pushbullet error: {0:#?}")]
  Pushbullet(pushbullet::Error),

  #[error("reqwest error: {0:#?}")]
  Reqwest(reqwest::Error),

  #[error("serde_json error: {0:#?}")]
  Json(serde_json::Error),
}

impl From<reqwest::Error> for Error {
  fn from(e: reqwest::Error) -> Self {
    Self::Reqwest(e)
  }
}

impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Self {
    Self::Json(e)
  }
}

impl From<pushbullet::Error> for Error {
  fn from(e: pushbullet::Error) -> Self {
    Self::Pushbullet(e)
  }
}
