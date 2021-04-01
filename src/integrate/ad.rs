use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use thiserror::Error as DeriveError;

const SCOPES: &'static str = "openid offline_access";

struct Shared<'a> {
  client_id: &'a str,
  login_base_url: &'a str,
  graph_base_url: &'a str,
}

#[derive(Debug)]
pub enum Auth {
  NotAuthed {
    client_id: String,
    login_base_url: String,
    graph_base_url: String,
  },
  WaitForCodeAuth {
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
  pub async fn authenticate(
    self,
    reqw: &reqwest::Client,
  ) -> Result<Self, Error> {
    match self {
      | Auth::NotAuthed { .. } => self.start_code_flow(&reqw).await,
      | Auth::Authed { expires, .. } => {
        if expires > Utc::now() {
          Ok(self)
        } else {
          self.refresh(&reqw).await
        }
      }
      | Auth::WaitForCodeAuth { .. } => self.check_authed(&reqw).await,
    }
  }
}

impl Auth {
  async fn refresh(self, reqw: &reqwest::Client) -> Result<Self, Error> {
    use reqwest::multipart::Form;

    let Shared {
      client_id,
      login_base_url,
      graph_base_url,
    } = self.shared();

    let url = format!("{}/token", self.shared().login_base_url);
    let form = Form::new()
      .text("client_id", client_id.to_string())
      .text("grant_type", "refresh_token")
      .text("scope", SCOPES);

    let refreshed_json = reqw
      .post(url)
      .multipart(form)
      .send()
      .await
      .map_err(Error::Reqwest)?
      .text()
      .await
      .map_err(Error::Reqwest)?;

    let refreshed = serde_json::from_str::<RefreshResponse>(&refreshed_json)
      .map_err(Error::Json)?;

    let refresh = match self {
      | Auth::Authed { ref refresh, .. } => refresh,
      | _ => unreachable!(),
    }
    .to_string();

    Ok(Auth::Authed {
      client_id: client_id.to_string(),
      login_base_url: login_base_url.to_string(),
      graph_base_url: graph_base_url.to_string(),
      refresh,
      expires: Utc::now() + Duration::seconds(i64::from(refreshed.expires_in)),
      token: refreshed.access_token,
      id_token: refreshed.id_token,
    })
  }

  async fn start_code_flow(
    self,
    reqw: &reqwest::Client,
  ) -> Result<Self, Error> {
    use reqwest::multipart::Form;

    let Shared {
      client_id,
      login_base_url,
      graph_base_url,
    } = self.shared();

    let url = format!("{}/devicecode", login_base_url);
    let form = Form::new()
      .text("client_id", client_id.to_string())
      .text("scope", SCOPES);

    let json = reqw
      .post(url)
      .multipart(form)
      .send()
      .await
      .map_err(Error::Reqwest)?
      .text()
      .await
      .map_err(Error::Reqwest)?;

    let code =
      serde_json::from_str::<CodeResponse>(&json).map_err(Error::Json)?;

    Ok(Auth::WaitForCodeAuth {
      client_id: client_id.to_string(),
      login_base_url: login_base_url.to_string(),
      graph_base_url: graph_base_url.to_string(),
      my_code: code.device_code,
      user_code: code.user_code,
      url: code.verification_uri,
    })
  }

  async fn check_authed(self, reqw: &reqwest::Client) -> Result<Self, Error> {
    use reqwest::multipart::Form;

    let my_code = match self {
      | Self::WaitForCodeAuth { ref my_code, .. } => my_code,
      | _ => unreachable!(),
    }
    .to_string();

    let Shared {
      client_id,
      login_base_url: _,
      graph_base_url: _,
    } = self.shared();

    let url = format!("{}/token", self.shared().login_base_url);
    let form = Form::new()
      .text("client_id", client_id.to_string())
      .text("grant_type", "urn:ietf:params:oauth:grant-type:device_code")
      .text("scope", SCOPES)
      .text("device_code", my_code);

    let json = reqw
      .post(url)
      .multipart(form)
      .send()
      .await
      .map_err(Error::Reqwest)?
      .text()
      .await
      .map_err(Error::Reqwest)?;

    let err = serde_json::from_str::<ErrorResponse>(&json).ok();
    if let Some(err) = err {
      return Err(Error::from(err));
    }

    let auth =
      serde_json::from_str::<AuthResponse>(&json).map_err(Error::Json)?;

    let Shared {
      client_id,
      login_base_url,
      graph_base_url,
    } = self.shared();

    Ok(Auth::Authed {
      client_id: client_id.to_string(),
      login_base_url: login_base_url.to_string(),
      graph_base_url: graph_base_url.to_string(),
      token: auth.access_token,
      refresh: auth.refresh_token,
      expires: Utc::now() + Duration::seconds(i64::from(auth.expires_in)),
      id_token: auth.id_token,
    })
  }

  fn shared<'a>(&'a self) -> Shared<'a> {
    match self {
      | Self::NotAuthed {
        client_id,
        login_base_url,
        graph_base_url,
        ..
      }
      | Self::WaitForCodeAuth {
        client_id,
        login_base_url,
        graph_base_url,
        ..
      }
      | Self::Authed {
        client_id,
        login_base_url,
        graph_base_url,
        ..
      } => Shared {
        client_id: &client_id,
        login_base_url: &login_base_url,
        graph_base_url: &graph_base_url,
      },
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

  #[error(
    "Integrate AD: Authentication was declined, sending new code request..."
  )]
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
      | _ => Self::Other(resp.description),
    }
  }
}

#[derive(Deserialize)]
struct ErrorResponse {
  pub error: String,
  pub description: String,
}

#[derive(Deserialize)]
struct AuthResponse {
  pub token_type: String,
  pub scope: String,
  pub expires_in: u32,
  pub access_token: String,
  pub refresh_token: String,
  pub id_token: String,
}

#[derive(Deserialize)]
struct RefreshResponse {
  pub token_type: String,
  pub scope: String,
  pub expires_in: u32,
  pub access_token: String,
  pub id_token: String,
}

#[derive(Deserialize)]
struct CodeResponse {
  pub device_code: String,
  pub user_code: String,
  pub verification_uri: String,
}
