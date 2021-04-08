use std::env;

use super::*;
use crate::prelude::*;

// EXPLAIN MUT STATIC: want persistent app state across / between executions.
static mut STATE: Option<App> = None;

pub struct State;

impl State {
  fn already_init() -> bool {
    unsafe { STATE.is_some() }
  }

  fn init() -> Result<(), Error> {
    if Self::already_init() {
      log::debug!("Already init");
      return Ok(());
    }

    let mut state = App::empty()?;

    macro_rules! set_from_env {
      ($k:ident) => {
        Ok(std::stringify!($k)).map(str::to_uppercase)
                               .and_then(|k| env::var(k))
                               .tap_mut(|v| state.$k = v.to_string())
                               .map_err(|_| {
                                 std::stringify!($k).to_uppercase().to_string()
                               })
      };
    }

    let results = vec![set_from_env!(integrate_ad_client_id),
                       set_from_env!(pushbullet_token),
                       set_from_env!(pushbullet_base_url),
                       set_from_env!(integrate_ad_login_base_url),];
    log::debug!("Pulled environment variables: {:#?}", results);

    let errs = results.into_iter()
                      .filter_map(Result::err)
                      .collect::<Vec<_>>();

    if errs.len() > 0 {
      log::error!("App state errors: {:#?}", errs);
      Err(Error::EnvVarsMissing(errs))
    } else {
      state.init_integrate_ad_auth();
      state.add_notifier_pushbullet();
      state.add_calendar_integrate();

      log::debug!("Initialized: {:#?}", state);
      unsafe { STATE = Some(state) }

      Ok(())
    }
  }
}

#[async_trait]
impl super::Modify for State {
  fn modify(&self,
            f: impl FnOnce(App) -> Result<App, AnyError>)
            -> Result<(), Error> {
    Self::init()?;

    unsafe {
      let new_state =
                f(STATE.take().expect("STATE should be initialized")).map_err(Error::Any)?;
      STATE = Some(new_state);
    }

    Ok(())
  }

  async fn modify_async<'a,
                          R: Send + Future<Output = Result<App, AnyError>>>(
    &'a self,
    f: impl 'a + Send + FnOnce(App) -> R)
    -> Result<(), Error> {
    log::debug!("ensuring STATE set");
    Self::init()?;

    let state = unsafe { STATE.take().expect("STATE should be initialized") };
    let new_state =
      f(state).await
              .tap_err(|e| log::error!("Callback errored: {:#?}", e))
              .map_err(Error::Any)?;

    unsafe {
      STATE = Some(new_state);
    }

    Ok(())
  }
}

impl super::Read for State {
  fn read(&self) -> Result<&App, Error> {
    Self::init()?;
    unsafe {
      STATE.as_ref().ok_or(Error::Other(
                "`STATE` was not initialized in `App::init`.".into(),
            ))
    }
  }
}
