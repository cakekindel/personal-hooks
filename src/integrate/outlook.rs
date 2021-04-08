use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize as Ser, Deserialize as De};
use serde::de::{DeserializeOwned as DeOwned};

use crate::{calendar::*, prelude::*};

#[derive(Debug)]
pub struct Outlook(super::ad::Auth);

impl Outlook {
  pub fn new(auth: super::ad::Auth) -> Self {
    match auth {
      super::ad::Auth::Authed {..} => {},
      _ => log::warn!("Outlook was constructed with integrate::ad::Auth::{}, will not authenticate.", auth.dbg_label()),
    };

    Outlook(auth)
  }
}

#[async_trait]
impl Calendar for Outlook {
  async fn get_events(&self,
                      reqw: &reqwest::Client,
                      after: DateTime<Utc>,
                      before: DateTime<Utc>)
                      -> Result<Vec<Event>, crate::AnyError> {
    let token = self.0.token().map(String::from).unwrap_or_default();

    let url = format!("{}/me/calendar/calendarView", self.0.shared().graph_base_url);
    log::info!("GET {}", url);

    let query = [("startDateTime", &after.to_rfc3339()),
                 ("endDateTime", &before.to_rfc3339())];
    log::info!("> query: {:#?}", query);

    reqw.get(url)
        .query(&query)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .norm()?
        .text()
        .await
        .norm()
        .bind(|json| serde_json::from_str::<ResponseWrapper<CalendarViewResponse>>(&json)
                       .map(|r| r.value)
                       .tap_err(|_| log::error!("> Failed to parse CalendarViewResponse: {}", json))
                       .map(|es| es.into_iter().map(Into::<Event>::into).collect::<Vec<_>>())
                       .tap(|events| log::info!("> Got {} events from outlook", events.len()))
                       .norm()
        )
  }
}

#[derive(Ser, De, Debug)]
struct ResponseWrapper<T: Ser + DeOwned + std::fmt::Debug> {
  #[serde(bound(deserialize = "T: DeOwned"))]
  value: Vec<T>
}

#[derive(Ser, De, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CalendarViewResponse {
  subject: String,
  start: DateWrapper,
  end: DateWrapper,
  location: Location,
}

impl Into<Event> for CalendarViewResponse {
  fn into(self) -> Event {
    Event {
      cat: Cat::Work,
      title: self.subject,
      time_start: self.start.into(),
      time_end: self.end.into(),
      location: Some(self.location.display_name),
    }
  }
}

#[derive(Ser, De, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Location {
  display_name: String,
}

#[derive(Ser, De, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DateWrapper {
  date_time: String,
  time_zone: String,
}

impl Into<DateTime<Utc>> for DateWrapper {
  fn into(self) -> DateTime<Utc> {
    use chrono::{NaiveDateTime};

    if &self.time_zone != "UTC" { log::error!("Timezone is not UTC: {}", self.time_zone) }

    NaiveDateTime::parse_from_str(&self.date_time, "%Y-%m-%dT%H:%M:%S.%f")
      .map(|n| DateTime::<Utc>::from_utc(n, Utc)).expect("should be parseable")
  }
}

mod tests {
  use super::*;

  #[test]
  pub fn calendar_view_response_should_deserialize() {
    // ARRANGE
    let json = r##"{
      "subject": "Test Event",
      "start": { "dateTime": "2021-04-08T20:00:00.0000000", "timeZone": "UTC" },
      "end": { "dateTime": "2021-04-08T21:30:00.0000000", "timeZone": "UTC" },
      "location": {
        "displayName": "https://zoom.us/j/12345",
        "locationType": "default",
        "uniqueId": "https://zoom.us/j/12345",
        "uniqueIdType": "private"
      }
    }"##;

    let expected = CalendarViewResponse {
      subject: "Test Event".into(),
      start: DateWrapper {
        date_time: "2021-04-08T20:00:00.0000000".into(),
        time_zone: "UTC".into(),
      },
      end: DateWrapper {
        date_time: "2021-04-08T21:30:00.0000000".into(),
        time_zone: "UTC".into(),
      },
      location: Location {
        display_name: "https://zoom.us/j/12345".into(),
      },
    };

    // ACT
    let parsed = serde_json::from_str::<CalendarViewResponse>(&json).expect("should deserialize");

    // ASSERT
    if parsed != expected { panic!("expected {:#?}, got {:#?}", expected, parsed) }
  }
}
