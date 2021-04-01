use netlify_lambda::{lambda, Context};
use serde_json::Value;

mod app;
mod calendar;
mod integrate;
mod lamb;
mod notify;
mod utils;

use app::StaticMutState;
use utils::*;

type AnyError = Box<dyn std::error::Error + Send + Sync>;

#[lambda]
#[tokio::main]
async fn main(event_raw: Value, _: Context) -> Result<Value, AnyError> {
  use lamb::{Event::*, ScheduleKind::*};

  let event = lamb::Event::from_value(event_raw).norm()?;

  match event {
    | Http(req) => handle::http(StaticMutState, req),
    | Schedule { kind: RunJobs } => handle::jobs(StaticMutState).await,
    | _ => handle::noop(),
  }
}

mod handle {
  use serde_json::Value;

  use crate::{app, lamb, utils::*};

  pub fn noop() -> Result<Value, crate::AnyError> {
    serde_json::to_value(()).norm()
  }

  pub async fn jobs(
    state: impl app::ReadState + app::ModifyState,
  ) -> Result<Value, crate::AnyError> {
    state
      .modify_async(|mut s| async {
        s.integrate_ad_auth = s.integrate_ad_auth.authenticate(&s.reqw).await?;
        Ok(s)
      })
      .await?;

    serde_json::to_value(()).norm()
  }

  pub fn http(
    _state: impl app::ReadState,
    req: lamb::HttpRequest,
  ) -> Result<Value, crate::AnyError> {
    let headers = std::collections::HashMap::<String, String>::new();

    let res = lamb::HttpResponse {
      status: 200,
      body: req.body,
      headers: Some(headers),
    };

    serde_json::to_value(res).norm()
  }
}
