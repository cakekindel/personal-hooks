use maplit::hashmap;
use serde_json::Value;
use state::Ext;
use chrono::{Utc, Timelike, DateTime, FixedOffset};

use crate::{app, app::state, lamb, prelude::*};

pub fn noop() -> Result<Value, crate::AnyError> {
  log::info!("noop called, exiting app");
  serde_json::to_value(()).norm()
}

pub async fn jobs(state: &(impl state::Read + state::Modify + Sync))
                  -> Result<Value, crate::AnyError> {
                      noop()
                  }

pub async fn summary(state: &(impl state::Read + state::Modify + Sync), kind: lamb::ScheduleKind)
                  -> Result<Value, crate::AnyError> {
  use lamb::ScheduleKind::*;
  use chrono::Duration as Dur;

  state.authenticate_integrate()
       .await
       .tap_err(|e| log::error!("Error authenticating to AD: {:#?}", e))?;

  let last_midnight = Utc::now().with_hour(0).unwrap().with_minute(0).unwrap();
  let this_midnight = last_midnight + Dur::days(1);

  let begin = match kind {
    KeepWarm => unreachable!(),
    SummaryToday => last_midnight,
    SummaryTomorrow => this_midnight,
  };

  let end = begin + Dur::days(1);

  let mut events = state.get_events(begin, end)
                    .await?;

  // sort by start date ascending
  events.sort_by(|a, b| {
    std::cmp::Ord::cmp(&a.time_start, &b.time_start)
  });

  let msg = events.into_iter()
                  .fold(String::new(), |msg, event| {
                    let fmt_time = |dt: DateTime::<Utc>| {
                      let hours = 3600;
                      let mst = FixedOffset::west(7 * hours);
                      dt.with_timezone(&mst).format("%I:%M%p")
                    };

                    let event_msg = format!("\"{}\" ({})\n{} - {}", event.title, event.cat, fmt_time(event.time_start), fmt_time(event.time_end));

                    msg + &event_msg + "\n\n"
                  });

  state.notify("Today's Events", &msg)
       .await?;

  noop()
}

pub async fn http(state: &(impl state::Read + state::Modify + Sync),
                  req: lamb::HttpRequest)
                  -> Result<Value, crate::AnyError> {
  use lamb::HttpMethod::*;

  let error_response = move |e: super::AnyError| {
    lamb::HttpResponse::new()
            .status(500)
            .body_json(hashmap! {
              "success" => serde_json::to_value(false).unwrap(),
              "errors" => serde_json::to_value(format!("{:#?}", e).as_str()).unwrap(),
            })
            .expect("error should serialize")
  };

  let execute = || async {
    jobs(state).await
               .bind(|_| {
                 lamb::HttpResponse::new().body_json(hashmap! {
                              "success" => serde_json::to_value(true).unwrap()
                            })
                 .norm()
               })
               .tap(|r| log::info!("Responding: {:#?}", r))
               .map_err(error_response)
               .tap_err(|e| log::error!("Responding: {:#?}", e))
               .open()
  };

  let response = match (req.method, req.path.as_str()) {
    | (Post, "/execute") => execute().await,
    | (_, "/execute") => lamb::HttpResponse::new().status(405),
    | (_, _) => todo!(),
  };

  serde_json::to_value(&response)
        .norm()
        .tap_err(|e| log::error!("Failed to serialize HTTP response: {:#?}", e))
}
