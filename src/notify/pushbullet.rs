use async_trait::async_trait;
use serde::{de::DeserializeOwned as DeOwned,
            Deserialize as De,
            Serialize as Ser};

use crate::prelude::*;

#[derive(Debug, Ser, De)]
pub struct Error {
  error: ErrorObject,
}

#[derive(Debug, Ser, De)]
pub struct ErrorObject {
  #[serde(rename = "type")]
  kind: String,
  message: String,
  param: Option<String>,
  cat: String,
}

#[derive(Debug)]
pub struct Pushbullet {
  pub base_url: String,
  pub token: String,
}

impl Pushbullet {
  async fn fail_if_error(r: reqwest::Response)
                         -> Result<reqwest::Response, super::Error> {
    if !r.status().is_success() {
      log::error!("> status {}", r.status());
      let err = r.text()
                 .await
                 .map_err(Into::into)
                 .and_then(Self::parse::<Error>)
                 .map(Into::into)
                 .open();
      log::error!("> err {}", err);
      Err(err)
    } else {
      Ok(r)
    }
  }

  fn parse<T: DeOwned>(json: String) -> Result<T, super::Error> {
    serde_json::from_str(&json).map_err(Into::into)
  }

  async fn get_devices(&self,
                       reqw: &reqwest::Client)
                       -> Result<Vec<Device>, super::Error> {
    let url = format!("{}/devices", self.base_url);
    log::info!("GET {}", url);

    reqw.get(url)
        .header("Access-Token", self.token.as_str())
        .send()
        .await
        .map_err(Into::into)
        .bind_async(Self::fail_if_error)
        .await?
        .text()
        .await
        .map_err(Into::into)
        .and_then(Self::parse::<DeviceResponse>)
        .map(|resp| resp.devices)
        .tap(|ds| log::info!("> OK: {:#?}", ds))
  }

  async fn push(&self,
                reqw: &reqwest::Client,
                push: Push)
                -> Result<(), super::Error> {
    let url = format!("{}/pushes", self.base_url);
    log::info!("POST {}", url);

    reqw.post(url)
        .header("Access-Token", self.token.as_str())
        .json(&push)
        .send()
        .await
        .map_err(Into::into)
        .bind_async(Self::fail_if_error)
        .await
        .tap(|_| log::info!("> OK"))
        .map(|_| ())
  }
}

#[async_trait]
impl super::Notifier for Pushbullet {
  async fn notify(&self,
                  reqw: &reqwest::Client,
                  title: &str,
                  body: &str)
                  -> Result<(), super::Error> {
    let push = Push::Note { title: title.into(),
                            body: body.into() };
    self.push(reqw, push).await.map(|_| ())
  }
}

#[derive(Debug, Ser, De)]
struct DeviceResponse {
  pub devices: Vec<Device>,
}

#[derive(Debug, Ser, De)]
struct Device {
  pub iden: String,
}

#[derive(Ser, De)]
#[serde(tag = "type")]
enum Push {
  #[serde(rename = "note")]
  Note { title: String, body: String },
}
