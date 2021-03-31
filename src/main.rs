use netlify_lambda::{lambda, Context};
use serde_json::Value;

mod calendar;
mod event;
mod lamb;

pub struct Config {
  pub outlook: calendar::outlook::Config,
}

type AnyError = Box<dyn std::error::Error + Send + Sync>;

#[lambda]
#[tokio::main]
async fn main(event_raw: Value, _: Context) -> Result<Value, AnyError> {
  let event = lamb::Event::from_value(event_raw);
  match event {
    | lamb::Event::Http(req) => handle_http(req),
    | _ => handle_sched(),
  }
}

fn handle_sched() -> Result<Value, AnyError> {
  println!("when the scheduled execution hits");
  serde_json::to_value(()).map_err(|e| Box::new(e) as AnyError)
}

fn handle_http(req: lamb::HttpRequest) -> Result<Value, AnyError> {
  let mut headers = std::collections::HashMap::<String, String>::new();

  headers.insert("X-Message".into(), "Echo!".into());

  let res = lamb::HttpResponse { status:  200,
                                 body:    req.body,
                                 headers: Some(headers), };

  serde_json::to_value(res).map_err(|err| Box::new(err) as AnyError)
}
