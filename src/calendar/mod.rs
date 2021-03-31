use chrono::{DateTime, Utc};

use crate::event::*;

trait Calendar {
  type Error;

  fn get_events(after: DateTime<Utc>,
                before: DateTime<Utc>)
                -> Result<Vec<Event>, Self::Error>;
}
