use std::env;

use thiserror::Error as DeriveError;

use crate::utils::*;

#[derive(Debug, DeriveError)]
pub enum Error {
  #[error("Required environment variables missing: {0:?}")]
  EnvVarsMissing(Vec<String>),

  #[error("{0}")]
  Other(String),
}

// EXPLAIN MUT STATIC: want persistent app state across / between executions.
static mut APP_STATE: Option<App> = None;

pub(super) struct StaticMutState;

pub struct App {
  pub integrate_ad_client_id: String,
  pub pushbullet_token:       String,
}

pub trait ReadState
  where Self: Sized
{
  fn read(&self) -> Result<&App, Error>;
}

pub trait ModifyState
  where Self: Sized
{
  fn modify(&self, f: impl FnOnce(&App) -> App) -> Result<(), Error>;
}

impl ReadState for StaticMutState {
  fn read(&self) -> Result<&App, Error> {
    Self::init()?;
    unsafe {
      APP_STATE.as_ref().ok_or(Error::Other("`APP_STATE` was not initialized in `App::init`.".into()))
    }
  }
}

impl ModifyState for StaticMutState {
  fn modify(&self, f: impl FnOnce(&App) -> App) -> Result<(), Error> {
    let new_state = f(self.read()?);

    unsafe {
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

    let mut state = App { integrate_ad_client_id: String::new(),
                          pushbullet_token:       String::new(), };

    macro_rules! get_from_env {
      ($k:ident) => {
        Ok("$k").map(str::to_uppercase)
                .and_then(|k| env::var(k))
                .tap_mut(|v| state.$k = v.to_string())
                .map_err(|_| "$k".to_string())
      };
    }

    let results = vec![get_from_env!(integrate_ad_client_id),
                       get_from_env!(pushbullet_token),];

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
