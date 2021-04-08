use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize as De, Serialize as Ser};
use serde::de::{DeserializeOwned as DeOwned};
use thiserror::Error as DeriveError;

use crate::prelude::*;

const SCOPES: &'static str = "openid offline_access Calendars.Read";

pub struct Shared<'a> {
  pub client_id: &'a str,
  pub login_base_url: &'a str,
  pub graph_base_url: &'a str,
}

#[derive(Clone, Debug, Ser, De)]
pub enum Auth {
  NotAuthed {
    client_id: String,
    login_base_url: String,
    graph_base_url: String,
  },
  WaitForCodeAuth {
    message: String,
    my_code: String,
    user_code: String,
    url: String,
    client_id: String,
    login_base_url: String,
    graph_base_url: String,
  },
  Authed {
    client_id: String,
    login_base_url: String,
    graph_base_url: String,
    expires: DateTime<Utc>,
    token: String,
    refresh: String,
    id_token: String,
  },
}

impl Auth {
  pub fn dbg_label(&self) -> String {
    match self {
      Self::NotAuthed {..} => "NotAuthed",
      Self::WaitForCodeAuth {..} => "WaitForCodeAuth",
      Self::Authed {..} => "Authed",
    }.into()
  }

  pub fn token(&self) -> Option<&str> {
    match self {
      | Self::Authed { token, .. } => Some(token),
      | _ => None,
    }
  }

  pub fn wait_msg(&self) -> Option<&str> {
    match self {
      | Self::WaitForCodeAuth { message, .. } => Some(message),
      | _ => None,
    }
  }

  pub fn user_code(&self) -> Option<&str> {
    match self {
      | Self::WaitForCodeAuth { user_code, .. } => Some(user_code),
      | _ => None,
    }
  }

  pub async fn authenticate(self,
                            reqw: &reqwest::Client)
                            -> Result<Self, Error> {
    log::info!("Authenticating...");

    match self {
      | Auth::NotAuthed { .. } => self.start_code_flow(&reqw).await,
      | Auth::Authed { expires, .. } => {
          self.refresh(&reqw).await
      },
      | Auth::WaitForCodeAuth { .. } => self.check_authed(&reqw).await,
    }
  }
}

impl Auth {
  fn try_parse<T: DeOwned>(json: String) -> Result<T, Error> {
    serde_json::from_str::<ErrorResponse>(&json)
      .ok()
      .tap(|err| log::error!("> Error: {:#?}", err))
      .map(Error::from)
      .map(Err)
      .unwrap_or(serde_json::from_str::<T>(&json).map_err(Error::Json))
  }

  async fn refresh(self, reqw: &reqwest::Client) -> Result<Self, Error> {
    use reqwest::multipart::Form;

    let Shared { client_id,
                 login_base_url,
                 graph_base_url, } = self.shared();

    let refresh = match self {
                    | Auth::Authed { ref refresh, .. } => refresh,
                    | _ => unreachable!(),
                  }.to_string();

    let url = format!("{}/token", self.shared().login_base_url);
    let form = Form::new().text("client_id", client_id.to_string())
                          .text("grant_type", "refresh_token")
                          .text("refresh_token", refresh.to_string())
                          .text("scope", SCOPES);

    let refreshed = reqw.post(url)
                        .multipart(form)
                        .send()
                        .await
                        .map_err(Error::Reqwest)?
                        .text()
                        .await
                        .map_err(Error::Reqwest)
                        .bind(Self::try_parse::<RefreshResponse>)?;

    let expires = Utc::now()
                        + Duration::seconds(i64::from(refreshed.expires_in));

    Ok(Auth::Authed { client_id: client_id.to_string(),
                      login_base_url: login_base_url.to_string(),
                      graph_base_url: graph_base_url.to_string(),
                      refresh,
                      expires,
                      token: refreshed.access_token,
                      id_token: refreshed.id_token })
  }

  async fn start_code_flow(self,
                           reqw: &reqwest::Client)
                           -> Result<Self, Error> {
    use reqwest::multipart::Form;

    log::info!("Not authed, starting code flow");

    let Shared { client_id,
                 login_base_url,
                 graph_base_url, } = self.shared();

    let url = format!("{}/devicecode", login_base_url);
    let form = Form::new().text("client_id", client_id.to_string())
                          .text("scope", SCOPES);

    log::info!("POST {}", url);
    let code = reqw.post(url)
                   .multipart(form)
                   .send()
                   .await
                   .map_err(Error::Reqwest)
                   .tap_err(|e| log::error!("> {}", e))?
                   .text()
                   .await
                   .map_err(Error::Reqwest)
                   .bind(Self::try_parse::<CodeResponse>)?;

    log::info!("> OK {:#?}", code);

    Ok(Auth::WaitForCodeAuth { client_id: client_id.to_string(),
                               login_base_url: login_base_url.to_string(),
                               graph_base_url: graph_base_url.to_string(),
                               my_code: code.device_code,
                               message: code.message,
                               user_code: code.user_code,
                               url: code.verification_uri })
  }

  async fn check_authed(self, reqw: &reqwest::Client) -> Result<Self, Error> {
    log::info!("Checking if code has been authenticated...");
    use reqwest::multipart::Form;

    let my_code = match self {
                    | Self::WaitForCodeAuth { ref my_code, .. } => my_code,
                    | _ => unreachable!(),
                  }.to_string();

    let Shared { client_id,
                 login_base_url: _,
                 graph_base_url: _, } = self.shared();

    let url = format!("{}/token", self.shared().login_base_url);
    let form = Form::new().text("client_id", client_id.to_string())
                          .text("grant_type",
                                "urn:ietf:params:oauth:grant-type:device_code")
                          .text("device_code", my_code);

    log::info!("POST {}", url);
    log::info!("> {:#?}", form);

    let auth = reqw.post(url)
                   .multipart(form)
                   .send()
                   .await
                   .map_err(Error::Reqwest)?
                   .text()
                   .await
                   .map_err(Error::Reqwest)
                   .bind(Self::try_parse::<AuthResponse>)?;

    let Shared { client_id,
                 login_base_url,
                 graph_base_url, } = self.shared();

    let expires =
                        Utc::now()
                        + Duration::seconds(i64::from(auth.expires_in));

    Ok(Auth::Authed { client_id: client_id.to_string(),
                      login_base_url: login_base_url.to_string(),
                      graph_base_url: graph_base_url.to_string(),
                      token: auth.access_token,
                      refresh: auth.refresh_token,
                      expires,
                      id_token: auth.id_token })
  }

  pub fn shared<'a>(&'a self) -> Shared<'a> {
    match self {
      | Self::NotAuthed { client_id,
                          login_base_url,
                          graph_base_url,
                          .. }
      | Self::WaitForCodeAuth { client_id,
                                login_base_url,
                                graph_base_url,
                                .. }
      | Self::Authed { client_id,
                       login_base_url,
                       graph_base_url,
                       .. } => Shared { client_id: &client_id,
                                        login_base_url: &login_base_url,
                                        graph_base_url: &graph_base_url },
    }
  }
}

#[derive(Debug, DeriveError)]
pub enum Error {
  #[error("Integrate AD: Error {0}")]
  Other(String),

  #[error("Integrate AD: Error making request: {0:#?}")]
  Reqwest(reqwest::Error),

  #[error("Integrate AD: Error deserializing JSON response: {0:#?}")]
  Json(serde_json::Error),

  #[error("Integrate AD: Authentication pending...")]
  CodePending,

  #[error("Integrate AD: Authentication was declined, sending new code request...")]
  CodeDeclined,

  #[error("Integrate AD: Code entered was bad")]
  CodeBad,

  #[error("Integrate AD: Code expired, sending new code request...")]
  CodeExpired,
}

impl From<ErrorResponse> for Error {
  fn from(resp: ErrorResponse) -> Self {
    match resp.error.as_str() {
      | "authorization_pending" => Self::CodePending,
      | "authorization_declined" => Self::CodeDeclined,
      | "bad_verification_code" => Self::CodeBad,
      | "expired_token" => Self::CodeExpired,
      | _ => Self::Other(resp.error_description),
    }
  }
}

#[derive(Debug, Ser, De)]
struct ErrorResponse {
  pub error: String,
  pub error_description: String,
}

#[derive(Debug, Ser, De)]
struct AuthResponse {
  pub token_type: String,
  pub scope: String,
  pub expires_in: u32,
  pub access_token: String,
  pub refresh_token: String,
  pub id_token: String,
}

#[derive(Debug, Ser, De)]
struct RefreshResponse {
  pub token_type: String,
  pub scope: String,
  pub expires_in: u32,
  pub access_token: String,
  pub id_token: String,
}

#[derive(Debug, Ser, De)]
struct CodeResponse {
  pub message: String,
  pub device_code: String,
  pub user_code: String,
  pub verification_uri: String,
}
