pub struct Outlook {
  token: String,
}

impl Outlook {
  pub fn new(cfg: Config) -> Self {
    Outlook { token: "".to_string(), }
  }
}

pub struct Config {
  pub deviceCode: Option<String>,
}

pub enum AuthState {
  None {},
  WaitForDeviceCode {},
  Authed {},
}
