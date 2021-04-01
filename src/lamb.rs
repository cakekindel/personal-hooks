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
  RunJobs,
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

#[derive(Debug, Deserialize, Serialize)]
pub enum HttpMethod {
  Get,
  Post,
  Options,

  #[serde(other)]
  Other,
}
