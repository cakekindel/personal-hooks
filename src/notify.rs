pub struct Pushbullet;
pub struct Email;
pub struct Slack;

pub trait Notifier {
  fn notify(&self, msg: &str) -> Result<(), crate::AnyError>;
}
