use netlify_lambda::{lambda, Context};
use serde_json::Value;

mod app;
mod calendar;
mod handle;
mod integrate;
mod lamb;
mod notify;
mod prelude;

use app::{state,
          state::{Ext, Read}};
use prelude::*;

type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda]
#[tokio::main]
async fn main(event_raw: Value, _: Context) -> Result<Value, AnyError> {
  use lamb::{Event::*, ScheduleKind::*};

  init_logger()?;

  let event = lamb::Event::from_value(event_raw)
        .norm()
        .tap_err(|e| log::error!("Failed to parse event: {:#?}", e))?;
  log::debug!("Received event: {:#?}", event);

  fn s() -> state::S {
    match std::env::var("STATE_MODE").ok()
                                     .as_ref()
                                     .map(String::as_str)
    {
      | Some("FILE") => state::S::File,
      | _ => state::S::InMem,
    }
  }

  let handle_result = match event {
    | Http(req) => handle::http(&s(), req).await,
    | Schedule { kind } => handle::summary(&s(), kind).await,
    | _ => handle::noop(),
  };

  if let Err(err) = handle_result {
    log::error!("{:#?}", err);

    s().notify("error", &format!("{}", err))
       .await
       .tap_err(|e| log::error!("Failed to read app state: {:#?}", e))?;

    handle::noop()
  } else {
    handle_result
  }
}

fn init_logger() -> Result<(), fern::InitError> {
  fern::Dispatch::new().format(|out, message, record| {
                         out.finish(format_args!(
      "{}[{}][{}] {}",
      chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
      record.target(),
      record.level(),
      message
    ))
                       })
                       .level(log::LevelFilter::Info)
                       .chain(std::io::stdout())
                       .apply()?;
  Ok(())
}
