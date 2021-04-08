use std::fmt;

use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq)]
pub struct Event {
  pub cat: Cat,
  pub title: String,
  pub time_start: DateTime<Utc>,
  pub time_end: DateTime<Utc>,
  pub location: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Cat {
  Work,
  Personal(Personal),
}

impl fmt::Display for Cat {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      | Cat::Work => write!(f, "Work"),
      | Cat::Personal(p) => write!(f, "Personal: {}", p.to_string()),
    }
  }
}

#[derive(Debug, PartialEq)]
pub enum Personal {
  Chore,
  Habit,
  Plan,
  Med,
}

impl fmt::Display for Personal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", match self {
      | Self::Chore => "Chore",
      | Self::Habit => "Habit",
      | Self::Plan => "Plan",
      | Self::Med => "Medical",
    })
  }
}
