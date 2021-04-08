use std::collections::HashMap;

use serde::{Deserialize, Serialize};

type Object = HashMap<String, String>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Event {
  Http(HttpRequest),
  Schedule { kind: ScheduleKind },
}

impl Event {
  pub fn from_value(val: serde_json::Value) -> serde_json::Result<Self> {
    serde_json::from_value::<Self>(val)
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ScheduleKind {
  KeepWarm,
  SummaryToday,
  SummaryTomorrow,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRequest {
  pub path: String,

  #[serde(rename = "httpMethod")]
  pub method: HttpMethod,

  pub headers: Option<Object>,

  #[serde(rename = "queryStringParameters")]
  pub query: Option<Object>,

  pub body: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpResponse {
  #[serde(rename = "statusCode")]
  pub status: i32,
  pub headers: Option<Object>,
  pub body: Option<String>,
}

impl HttpResponse {
  pub fn new() -> Self {
    Self { status: 200,
           headers: None,
           body: None }
  }

  pub fn status(mut self, status: impl Into<i32>) -> Self {
    self.status = status.into();
    self
  }

  pub fn body_json(mut self,
                   body: impl Serialize)
                   -> Result<Self, serde_json::Error> {
    serde_json::to_string(&body).map(|json| {
                                  self.body(json).header("content-type",
                                                         "application/json")
                                })
  }

  pub fn body(mut self, body: impl ToString) -> Self {
    self.body = Some(body.to_string());
    self
  }

  pub fn headers(mut self, headers: impl Into<Object>) -> Self {
    self.headers = Some(headers.into());
    self
  }

  pub fn header(mut self, k: impl ToString, v: impl ToString) -> Self {
    if let None = self.headers.as_ref() {
      self = self.headers(HashMap::new());
    }

    self.headers
        .as_mut()
        .unwrap()
        .insert(k.to_string(), v.to_string());

    self
  }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HttpMethod {
  Get,
  Post,
  Options,

  #[serde(other)]
  Other,
}
