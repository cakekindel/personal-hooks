pub trait NormalizeResult<T> {
  fn norm(self) -> Result<T, crate::AnyError>;
}

pub trait Open<A>
  where Self: Sized
{
  fn open(self) -> A;
}

impl<T, E: std::error::Error + Send + Sync + 'static> NormalizeResult<T>
  for Result<T, E>
{
  fn norm(self) -> Result<T, crate::AnyError> {
    self.map_err(|e| Box::new(e) as crate::AnyError)
  }
}

impl<A> Open<A> for Result<A, A> {
  fn open(self) -> A {
    self.unwrap_or_else(|e| e)
  }
}
