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

type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda]
#[tokio::main]
async fn main(event_raw: Value, _: Context) -> Result<Value, AnyError> {
  use lamb::{Event::*, ScheduleKind::*};

  let event = lamb::Event::from_value(event_raw).norm()?;

  let handle_result = match event {
    | Http(req) => handle::http(&StaticMutState, req),
    | Schedule { kind: RunJobs } => handle::jobs(&StaticMutState).await,
    | _ => handle::noop(),
  };

  if let Err(err) = handle_result {
    notify_all(&StaticMutState, &format!("{}", err)).await.and_then(|_| handle::noop())
  } else {
    handle_result
  }
}

pub async fn notify_all(state: &impl app::ReadState,
                        msg: &str)
                        -> Result<(), AnyError> {
  use futures::stream;
  use stream::StreamExt;

  let notifiers = &state.read().norm()?.notifiers;

  let aggregate_errors = |acc: Result<(), app::Error>,
                          res: Result<(), notify::Error>|
   -> Result<(), app::Error> {
    match (acc, res) {
      | (Ok(_), Ok(_)) => Ok(()),
      | (Err(app::Error::Many(errs)), Err(err)) => {
        Err(errs).tap_err_mut(|errs| errs.push(Box::from(err)))
      },
      | (Ok(_), Err(err)) => Err(vec![Box::from(err)]),
      | (Err(_), _) => unreachable!(),
    }.map_err(app::Error::Many)
  };

  futures::stream::iter(notifiers)
       .then(|ns| async move { ns.notify(msg).await })
       .fold(Ok(()), |acc, res| async { aggregate_errors(acc, res) })
       .await
       .norm()
}

mod handle {
  use serde_json::Value;

  use crate::{app, lamb, utils::*};

  pub fn noop() -> Result<Value, crate::AnyError> {
    serde_json::to_value(()).norm()
  }

  pub async fn jobs(state: &(impl app::ReadState + app::ModifyState))
                    -> Result<Value, crate::AnyError> {
    state.modify_async(|mut s| async {
           s.integrate_ad_auth =
             s.integrate_ad_auth.authenticate(&s.reqw).await?;
           Ok(s)
         })
         .await?;

    let app = state.read()?;

    match app.integrate_ad_auth.wait_msg() {
      | Some(code_msg) => {
        super::notify_all(state, code_msg).await?;

        Err(app::Error::Other("you're authenticated!".to_string())).norm()
      },
      | None => todo!(),
    }
  }

  pub fn http(_state: &impl app::ReadState,
              req: lamb::HttpRequest)
              -> Result<Value, crate::AnyError> {
    let headers = std::collections::HashMap::<String, String>::new();

    let res = lamb::HttpResponse { status:  200,
                                   body:    req.body,
                                   headers: Some(headers), };

    serde_json::to_value(res).norm()
  }
}
