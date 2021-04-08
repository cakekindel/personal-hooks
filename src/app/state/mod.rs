use std::future::Future;
use futures::stream::StreamExt;

use chrono::{DateTime, Utc};
use async_trait::async_trait;

mod file;
mod in_mem;

use super::{App, Error};
use crate::{prelude::*, AnyError};

pub enum S {
  File,
  InMem,
}

#[async_trait]
pub trait Ext: Read + Modify {
  async fn authenticate_integrate(&self) -> Result<(), AnyError> {
    self.modify_async(|mut s| async {
      log::info!("Authenticating against Integrate AD...");

      s.integrate_ad_auth =
        s.integrate_ad_auth.authenticate(&s.reqw).await?;

      let auth = &s.integrate_ad_auth;

      match (auth.wait_msg(), auth.user_code()) {
        | (Some(code_msg), Some(user_code)) => {
          log::info!("Need to authenticate with code");

          self.notify("Auth needed", code_msg).await?;

          self.notify("Code", user_code).await?;
        },
        | _ => (),
      };

      Ok(s)
    })
    .await
    .norm()
  }

  async fn get_events(&self,
                      after: DateTime<Utc>,
                      before: DateTime<Utc>)
                      -> Result<Vec<crate::calendar::Event>, crate::AnyError> {
    let app = self.read().norm()?;
    log::debug!("get_events ({} calendars) between {} and {}",
                app.calendars.len(),
                after,
                before);

    futures::stream::iter(&app.calendars)
      .then(|n| async move {
        n.get_events(&app.reqw, after, before)
         .await
         .tap(|_| log::info!("> calendar ok"))
         .tap_err(|e| log::error!("> calendar error: {:#?}", e))
         .map_err(Box::from)
      })
      .collect::<Vec<_>>()
      .await
      .into_iter()
      .collect_results::<Vec<_>, Vec<_>>()
      .map_err(super::Error::Many)
      .map(|nested| nested.into_iter().flatten().collect::<Vec<_>>())
      .norm()
  }

  async fn notify(&self, title: &str, body: &str) -> Result<(), AnyError> {
    let app = self.read().norm()?;
    log::debug!("notify_all ({} notifiers) - {}\n{}",
                app.notifiers.len(),
                title,
                body);

    futures::stream::iter(&app.notifiers)
      .then(|n| async move {
        n.notify(&app.reqw, title, body)
          .await
          .tap(|_| log::info!("> notify success"))
          .tap_err(|e| log::error!("> notify error: {:#?}", e))
          .map_err(Box::from)
      })
      .collect::<Vec<_>>()
      .await
      .into_iter()
      .collect_results::<(), Vec<_>>()
      .map_err(super::Error::Many)
      .norm()
  }
}

impl<T: Read + Modify> Ext for T {}

pub trait Read {
  fn read(&self) -> Result<&App, Error>;
}

#[async_trait]
pub trait Modify {
  fn modify(&self,
            f: impl FnOnce(App) -> Result<App, AnyError>)
            -> Result<(), Error>;

  /// this is a monstrosity
  async fn modify_async<'a, R: Send + Future<Output = Result<App, AnyError>>>(
    &'a self,
    f: impl 'a + Send + FnOnce(App) -> R)
    -> Result<(), Error>;
}

impl Read for S {
  fn read(&self) -> Result<&App, Error> {
    match self {
      | S::File => file::State.read(),
      | S::InMem => in_mem::State.read(),
    }
  }
}

#[async_trait]
impl Modify for S {
  fn modify(&self,
            f: impl FnOnce(App) -> Result<App, AnyError>)
            -> Result<(), Error> {
    match self {
      | S::File => file::State.modify(f),
      | S::InMem => in_mem::State.modify(f),
    }
  }

  async fn modify_async<'a,
                          R: Send + Future<Output = Result<App, AnyError>>>(
    &'a self,
    f: impl 'a + Send + FnOnce(App) -> R)
    -> Result<(), Error> {
    match self {
      | S::File => file::State.modify_async(f).await,
      | S::InMem => in_mem::State.modify_async(f).await,
    }
  }
}
