use std::{env, future::Future, marker::Send};

use async_trait::async_trait;
use thiserror::Error as DeriveError;

use crate::{integrate, utils::*};

#[derive(Debug, DeriveError)]
pub enum Error {
  #[error("Required environment variables missing: {0:?}")]
  EnvVarsMissing(Vec<String>),

  #[error("{0}")]
  Other(String),

  #[error("{0}")]
  Any(crate::AnyError),
}

// EXPLAIN MUT STATIC: want persistent app state across / between executions.
static mut APP_STATE: Option<App> = None;

pub(super) struct StaticMutState;

#[derive(Debug)]
pub struct App {
  pub reqw:                   reqwest::Client,
  pub integrate_ad_client_id: String,
  pub pushbullet_token:       String,
  pub integrate_ad_auth:      integrate::ad::Auth,
}

pub trait ReadState
  where Self: Sized
{
  fn read(&self) -> Result<&App, Error>;
}

#[async_trait]
pub trait ModifyState
  where Self: Sized
{
  fn modify(&self,
            f: impl FnOnce(App) -> Result<App, crate::AnyError>)
            -> Result<(), Error>;

  /// this is a monstrosity
  async fn modify_async<'a,
                          R: Send + Future<Output = Result<App, crate::AnyError>>>(
    &'a self,
    f: impl 'a + Send + FnOnce(App) -> R)
    -> Result<(), Error>;
}

impl ReadState for StaticMutState {
  fn read(&self) -> Result<&App, Error> {
    Self::init()?;
    unsafe {
      APP_STATE.as_ref().ok_or(Error::Other(
        "`APP_STATE` was not initialized in `App::init`.".into(),
      ))
    }
  }
}

#[async_trait]
impl ModifyState for StaticMutState {
  fn modify(&self,
            f: impl FnOnce(App) -> Result<App, crate::AnyError>)
            -> Result<(), Error> {
    Self::init()?;

    unsafe {
      let new_state =
        f(APP_STATE.take().expect("APP_STATE should be initialized"))
          .map_err(Error::Any)?;
      APP_STATE = Some(new_state);
    }

    Ok(())
  }

  async fn modify_async<'a,
                          R: Send
                            + Future<Output = Result<App, crate::AnyError>>>(
    &'a self,
    f: impl 'a + Send + FnOnce(App) -> R)
    -> Result<(), Error> {
    Self::init()?;

    unsafe {
      let new_state =
        f(APP_STATE.take().expect("APP_STATE should be initialized"))
          .await
          .map_err(Error::Any)?;
      APP_STATE = Some(new_state);
    }

    Ok(())
  }
}

impl StaticMutState {
  fn already_init() -> bool {
    unsafe { APP_STATE.is_some() }
  }

  pub fn init() -> Result<(), Error> {
    if Self::already_init() {
      return Ok(());
    }

    let auth_empty =
      integrate::ad::Auth::NotAuthed { client_id:      String::new(),
                                       login_base_url: String::new(),
                                       graph_base_url: String::new(), };

    let mut state = App { reqw:                   reqwest::Client::new(),
                          integrate_ad_client_id: String::new(),
                          pushbullet_token:       String::new(),
                          integrate_ad_auth:      auth_empty, };

    macro_rules! set_from_env {
      ($k:ident) => {
        Ok("$k").map(str::to_uppercase)
                .and_then(|k| env::var(k))
                .tap_mut(|v| state.$k = v.to_string())
                .map_err(|_| "$k".to_string())
      };
    }

    let results = vec![set_from_env!(integrate_ad_client_id),
                       set_from_env!(pushbullet_token),];

    state.integrate_ad_auth = integrate::ad::Auth::NotAuthed {
      client_id: state.integrate_ad_client_id.clone(),
      login_base_url:
        "https://login.microsoftonline.com/organizations/oauth2/v2.0".into(),
      graph_base_url: "".into(),
    };

    let errs = results.into_iter()
                      .filter_map(Result::err)
                      .collect::<Vec<_>>();

    if errs.len() > 0 {
      Err(Error::EnvVarsMissing(errs))
    } else {
      unsafe { APP_STATE = Some(state) }
      Ok(())
    }
  }
}