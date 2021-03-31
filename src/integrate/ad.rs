pub enum Auth {
  None { client_id: String },
  WaitForDeviceCode { client_id: String },
  Authed { client_id: String },
}

impl Auth {
  pub fn new(client_id: String) -> Auth {
    Auth::None { client_id }
  }
  pub fn client_id(&self) -> &str {
    match self {
      | Self::None { client_id, .. } => client_id,
      | Self::WaitForDeviceCode { client_id, .. } => client_id,
      | Self::Authed { client_id, .. } => client_id,
    }
  }
}
