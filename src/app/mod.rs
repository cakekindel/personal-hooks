use serde::{Deserialize as De, Serialize as Ser};
use thiserror::Error as DeriveError;

use crate::{calendar::Calendar,
            integrate,
            notify,
            notify::Notifier,
            prelude::*,
            AnyError};

pub mod state;

#[derive(Debug, DeriveError)]
pub enum Error {
  #[error("Required environment variables missing: {0:?}")]
  EnvVarsMissing(Vec<String>),

  #[error("reqwest error: {0:#?}")]
  Reqwest(reqwest::Error),

  #[error("{0}")]
  Other(String),

  #[error("{0}")]
  Any(AnyError),

  #[error("{0:?}")]
  Many(Vec<AnyError>),
}

pub trait DebugNotifier: Notifier + std::fmt::Debug + Sync + Send {}
impl<T: Notifier + std::fmt::Debug + Sync + Send> DebugNotifier for T {}

pub trait DebugCalendar: Calendar + std::fmt::Debug + Sync + Send {}
impl<T: Calendar + std::fmt::Debug + Sync + Send> DebugCalendar for T {}

#[derive(Debug, Ser, De)]
pub struct App {
  #[serde(skip, default = "App::init_reqw_panic")]
  pub reqw: reqwest::Client,

  #[serde(skip, default)]
  pub notifiers: Vec<Box<dyn DebugNotifier>>,

  #[serde(skip, default)]
  pub calendars: Vec<Box<dyn DebugCalendar>>,

  // pushbullet
  pub pushbullet_token: String,
  pub pushbullet_base_url: String,

  // integrate
  pub integrate_ad_client_id: String,
  pub integrate_ad_auth: integrate::ad::Auth,
  pub integrate_ad_login_base_url: String,

  // ms
  pub ms_graph_base_url: String,
}

impl App {
  fn empty() -> Result<Self, Error> {
    let auth_empty =
      integrate::ad::Auth::NotAuthed { client_id: String::new(),
                                       login_base_url: String::new(),
                                       graph_base_url: String::new() };

    let app = Self { reqw: Self::init_reqw()?,
                     calendars: vec![],
                     notifiers: vec![],
                     integrate_ad_client_id: String::new(),
                     integrate_ad_login_base_url: String::new(),
                     ms_graph_base_url: String::new(),
                     pushbullet_base_url: String::new(),
                     pushbullet_token: String::new(),
                     integrate_ad_auth: auth_empty };

    Ok(app)
  }

  fn init_integrate_ad_auth(&mut self) -> () {
    match self.integrate_ad_auth {
      | integrate::ad::Auth::NotAuthed { .. } => {
        let client_id = self.integrate_ad_client_id.clone();
        let login_base_url = self.integrate_ad_login_base_url.clone();
        let graph_base_url = self.ms_graph_base_url.clone();

        self.integrate_ad_auth = integrate::ad::Auth::NotAuthed { client_id,
                                                                  login_base_url,
                                                                  graph_base_url };
      },
      | _ => (),
    }
  }

  fn add_calendar_integrate(&mut self) -> () {
    let outlook = integrate::Outlook::new(self.integrate_ad_auth.clone());

    self.calendars
        .push(Box::from(outlook) as Box<dyn DebugCalendar>);
  }

  fn add_notifier_pushbullet(&mut self) -> () {
    let pb_notifier =
      notify::Pushbullet { token: self.pushbullet_token.clone(),
                           base_url: self.pushbullet_base_url.clone() };

    self.notifiers
        .push(Box::from(pb_notifier) as Box<dyn DebugNotifier>);
  }

  fn init_reqw() -> Result<reqwest::Client, Error> {
    reqwest::Client::builder().use_rustls_tls()
                              .build()
                              .map_err(Error::Reqwest)
  }

  fn init_reqw_panic() -> reqwest::Client {
    reqwest::Client::builder().use_rustls_tls()
                              .build()
                              .map_err(Error::Reqwest)
                              .tap_err(|e| log::error!("{:#?}", e))
                              .unwrap()
  }
}
