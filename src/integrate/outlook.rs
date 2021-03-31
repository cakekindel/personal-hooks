pub struct Outlook(super::ad::Auth);

impl Outlook {
  pub fn new(auth: super::ad::Auth) -> Self {
    Outlook(auth)
  }
}
