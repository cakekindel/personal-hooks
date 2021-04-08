use async_trait::async_trait;
use chrono::{DateTime, Utc};

pub mod event;

pub use event::*;

#[async_trait]
pub trait Calendar {
  async fn get_events(&self,
                      reqw: &reqwest::Client,
                      after: DateTime<Utc>,
                      before: DateTime<Utc>)
                      -> Result<Vec<Event>, crate::AnyError>;
}
