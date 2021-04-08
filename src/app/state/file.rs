use std::fs;

use super::*;
use crate::prelude::*;

const FILE_PATH: &'static str = "./state.json";

static mut CUR: Option<App> = None;

pub struct State;

impl State {
  fn filepath() -> std::path::PathBuf {
    std::env::current_dir().unwrap().join(FILE_PATH)
  }

  fn read_owned() -> Result<App, Error> {
    log::info!("reading file at {:?}", Self::filepath());

    fs::read(Self::filepath()).norm()
                              .map_err(Error::Any)
                              .map(|b| String::from_utf8_lossy(&b).into_owned())
                              .and_then(|json| {
                                serde_json::from_str(&json).norm()
                                                           .map_err(Error::Any)
                              })
  }

  fn file_exists() -> bool {
    fs::File::open(Self::filepath()).is_ok()
  }

  fn init_file() -> Result<(), Error> {
    if Self::file_exists() {
      log::info!("File already exists");
      unsafe { CUR = Some(Self::read_owned()?) };
      return Ok(());
    }

    log::info!("Creating new empty file at {:?}", Self::filepath());
    let state = App::empty()?;

    Self::write(state)?;

    Ok(())
  }

  fn write(s: App) -> Result<(), Error> {
    log::info!("Writing file state");
    let json = serde_json::to_string_pretty(&s).norm()
                                               .map_err(Error::Any)?;

    fs::write(FILE_PATH, json).expect("write should succeed");

    unsafe { CUR = Some(s) };

    Ok(())
  }

  fn init() -> Result<(), Error> {
    let cur = unsafe { CUR.as_ref() };

    match cur {
      | Some(_exists) => {
        Ok(()).tap(|_| log::info!("init called but static already set"))
      },
      | None => Self::init_file().bind(|_| {
                                   let s = unsafe {
                                     CUR.as_mut().expect("should exist")
                                   };

                                   s.init_integrate_ad_auth();
                                   s.add_notifier_pushbullet();
                                   s.add_calendar_integrate();
                                   Ok(())
                                 })
                                 .tap(|_| log::info!("state initialized!")),
    }
  }
}

impl super::Read for State {
  fn read(&self) -> Result<&App, Error> {
    Self::init()?;
    unsafe {
      CUR.as_ref()
         .ok_or(Error::Other("CUR should have been initialized".into()))
    }
  }
}

#[async_trait]
impl super::Modify for State {
  fn modify(&self,
            f: impl FnOnce(App) -> Result<App, AnyError>)
            -> Result<(), Error> {
    Self::init()?;
    unsafe { CUR.take() }.ok_or(Error::Other("CUR not set".to_string()))
                         .bind(|a| f(a).map_err(Error::Any))
                         .bind(Self::write)
  }

  async fn modify_async<'a,
                          R: Send + Future<Output = Result<App, AnyError>>>(
    &'a self,
    f: impl 'a + Send + FnOnce(App) -> R)
    -> Result<(), Error> {
    Self::init()?;
    unsafe { CUR.take() }.ok_or(Error::Other("CUR not set".to_string()))
                         .bind_async(|a| async {
                           f(a).await.map_err(Error::Any)
                         })
                         .await
                         .bind(Self::write)
  }
}
